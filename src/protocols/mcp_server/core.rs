use anyhow::Result;
use futures::FutureExt;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info, trace};
use vfs::VfsInstance;

use crate::engine::browser::HeadlessWebBrowser;
use crate::protocols::mcp::{
    InitializeResult, McpMessage, McpMessageContent, McpNotification, McpRequest, McpResponse,
};
// websocket API is now natively implemented in Boa engine
// DOM is now natively handled by Boa engine
use crate::engine::EngineConfig;
use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::cdp::CdpServer;
use crate::protocols::cdp_tools::CdpTools;
use crate::protocols::memory_tools::MemoryTools;
use crate::protocols::rate_limiter::RateLimiter;
use crate::protocols::security::sanitize_session_id;
use crate::protocols::session_manager::SessionManager;
#[cfg(feature = "wasm-debug")]
use crate::protocols::wasm_debug_tools::WasmDebugTools;

#[allow(dead_code)]
pub struct McpServer {
    // websocket API is now natively implemented in Boa engine
    pub(super) ai_memory: AiMemoryHeap,
    pub(super) cdp_server: CdpServer,
    pub(super) memory_tools: MemoryTools,
    pub(super) cdp_tools: CdpTools,
    /// Shared browser tools - wrapped in Arc to share with CdpTools
    pub(super) browser_tools: Arc<BrowserTools>,
    pub(super) session_manager: SessionManager,
    /// Optional session-scoped persistent VFS instances. Keyed by session_id.
    pub(super) session_vfs: Arc<Mutex<HashMap<String, Arc<VfsInstance>>>>,
    /// Engine configuration for JavaScript execution
    pub(super) engine_config: EngineConfig,
    /// Rate limiter for DoS prevention
    pub(super) rate_limiter: RateLimiter,
    /// WASM debug tools (optional, requires wasm-debug feature + env var)
    #[cfg(feature = "wasm-debug")]
    pub(super) wasm_debug_tools: Option<WasmDebugTools>,
}

impl McpServer {
    pub fn new() -> Self {
        // Default to Boa engine for backward compatibility
        Self::new_with_engine(EngineConfig::new(false).unwrap_or(EngineConfig {
            engine_type: crate::engine::EngineType::Boa,
        }))
    }

    pub fn new_with_engine(engine_config: EngineConfig) -> Self {
        // Only enable AI memory if THALORA_ENABLE_AI_MEMORY is set to a truthy value
        // Accepts: "1", "true", "yes", "on" (case-insensitive)
        // Rejects: "0", "false", "no", "off", "" (empty), or unset
        let ai_memory_enabled = std::env::var("THALORA_ENABLE_AI_MEMORY")
            .map(|v| {
                let val = v.trim().to_lowercase();
                !val.is_empty() && val != "0" && val != "false" && val != "no" && val != "off"
            })
            .unwrap_or(false);

        let ai_memory = if ai_memory_enabled {
            tracing::info!("AI memory enabled via THALORA_ENABLE_AI_MEMORY");
            AiMemoryHeap::new_default().unwrap_or_else(|e| {
                tracing::warn!("Failed to load AI memory heap: {}, creating new one", e);
                AiMemoryHeap::new("/tmp/thalora_ai_memory.json")
                    .expect("Failed to create AI memory heap")
            })
        } else {
            // Create disabled AI memory (in-memory only, no persistence)
            tracing::info!(
                "AI memory disabled. Set THALORA_ENABLE_AI_MEMORY=1 to enable persistent storage"
            );
            AiMemoryHeap::new("/dev/null").unwrap_or_else(|_| {
                // Fallback to temp file if /dev/null fails (Windows)
                AiMemoryHeap::new(std::env::temp_dir().join("thalora_disabled.json"))
                    .expect("Failed to create disabled AI memory")
            })
        };

        let session_manager = SessionManager::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create session manager: {}", e);
            panic!("Could not initialize session manager: {}", e);
        });

        // Create shared BrowserTools instance
        let browser_tools = Arc::new(BrowserTools::new());

        #[cfg(feature = "wasm-debug")]
        let wasm_debug_tools = {
            use crate::protocols::mcp_server::tools::features::is_wasm_debug_enabled;
            if is_wasm_debug_enabled() {
                match WasmDebugTools::new() {
                    Ok(tools) => {
                        tracing::info!("WASM debug tools enabled via THALORA_ENABLE_WASM_DEBUG");
                        Some(tools)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize WASM debug tools: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };

        Self {
            // websocket API is now natively implemented in Boa engine
            ai_memory,
            cdp_server: CdpServer::new(),
            memory_tools: MemoryTools::new(),
            // Share BrowserTools instance with CdpTools to avoid duplicate Drop calls
            cdp_tools: CdpTools::with_browser_tools(browser_tools.clone()),
            browser_tools,
            session_manager,
            session_vfs: Arc::new(Mutex::new(HashMap::new())),
            engine_config,
            rate_limiter: RateLimiter::new(),
            #[cfg(feature = "wasm-debug")]
            wasm_debug_tools,
        }
    }

    /// Get or create a session-scoped VFS. If `backing_dir` is provided, the backing file will be created there.
    ///
    /// # Security
    /// - The session_id is validated to prevent path traversal attacks (CWE-22).
    /// - Session data is encrypted at rest using ChaCha20-Poly1305.
    /// - Only alphanumeric characters, hyphens, and underscores are allowed.
    pub fn get_or_create_session_vfs(
        &self,
        session_id: &str,
        backing_dir: Option<&std::path::Path>,
    ) -> Result<Arc<VfsInstance>> {
        // SECURITY: Validate session_id to prevent path traversal attacks
        let safe_session_id = sanitize_session_id(session_id)?;

        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.get(&safe_session_id) {
            return Ok(v.clone());
        }

        // Build a backing path: backing_dir/vfs-session-<session_id>.bin.enc
        // SECURITY: Use .bin.enc extension for encrypted session files
        let file = if let Some(dir) = backing_dir {
            dir.join(format!("vfs-session-{}.bin.enc", safe_session_id))
        } else {
            // fallback to temp dir
            std::env::temp_dir().join(format!("vfs-session-{}.bin.enc", safe_session_id))
        };

        // SECURITY: Derive encryption key from session_id and secret
        let key = vfs::derive_session_key(&safe_session_id);

        // Try to open existing encrypted file or create new
        let v = match VfsInstance::open_file_backed_encrypted(&file, &*key) {
            Ok(inst) => Arc::new(inst),
            Err(e) => {
                // Log decryption failure but create fresh VFS
                tracing::warn!("Failed to open encrypted session VFS: {}. Creating new.", e);
                Arc::new(
                    VfsInstance::new_temp_in_dir(std::env::temp_dir()).expect("create temp vfs"),
                )
            }
        };
        guard.insert(safe_session_id, v.clone());
        Ok(v)
    }

    /// Remove and optionally delete the backing file for a session VFS.
    ///
    /// # Security
    /// The session_id is validated to prevent path traversal attacks (CWE-22).
    pub fn remove_session_vfs(&self, session_id: &str, delete_backing: bool) -> Result<()> {
        // SECURITY: Validate session_id to prevent path traversal attacks
        let safe_session_id = sanitize_session_id(session_id)?;

        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.remove(&safe_session_id) {
            if delete_backing {
                drop(v.delete_backing_file());
            }
        }
        Ok(())
    }

    /// Cleanup all sessions and resources
    pub async fn cleanup(&self) {
        self.session_manager.shutdown().await;
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();

        // Configure idle timeout - if no input received for this duration, exit gracefully
        // Default: 5 minutes, can be overridden with THALORA_IDLE_TIMEOUT_SECS env var
        let idle_timeout_secs = std::env::var("THALORA_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300); // 5 minutes default

        let idle_timeout = std::time::Duration::from_secs(idle_timeout_secs);

        trace!(
            "MCP Server starting stdio loop (idle timeout: {}s)",
            idle_timeout_secs
        );

        loop {
            trace!("Waiting for input...");
            let mut line = String::new();

            // Use timeout on the read operation to prevent hanging forever
            let read_result = tokio::time::timeout(idle_timeout, reader.read_line(&mut line)).await;

            match read_result {
                Ok(Ok(0)) => {
                    trace!("EOF received, shutting down");
                    break;
                }
                Ok(Err(e)) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout occurred - no input received for idle_timeout duration
                    trace!(
                        "Idle timeout reached ({}s with no input), shutting down",
                        idle_timeout_secs
                    );
                    eprintln!(
                        "⏱️ MCP Server idle timeout reached ({}s), shutting down gracefully",
                        idle_timeout_secs
                    );
                    break;
                }
                Ok(Ok(n)) => {
                    trace!("Read {} bytes from stdin", n);
                    let line = line.trim();
                    if line.is_empty() {
                        trace!("Empty line, continuing");
                        continue;
                    }

                    trace!("Parsing JSON: {}", line);

                    // First, check if this is a notification (no 'id' field) or a request (has 'id' field)
                    let parsed: serde_json::Value = match serde_json::from_str(line) {
                        Ok(v) => {
                            trace!("JSON parsed successfully");
                            v
                        }
                        Err(e) => {
                            error!("Failed to parse JSON: {}", e);
                            continue;
                        }
                    };

                    // Check if this is a request (has non-null id) or notification (no id or null id)
                    let request_id = parsed.get("id").filter(|id| !id.is_null());

                    if let Some(request_id) = request_id {
                        trace!("Handling request with id: {}", request_id);
                        // This is a request - parse as McpRequest and send response
                        match serde_json::from_value::<McpRequest>(parsed.clone()) {
                            Ok(request) => {
                                trace!("Request parsed, calling handler");
                                let response = AssertUnwindSafe(self.handle_request(request))
                                    .catch_unwind()
                                    .await
                                    .unwrap_or_else(|payload| {
                                        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                                            s.to_string()
                                        } else if let Some(s) = payload.downcast_ref::<String>() {
                                            s.clone()
                                        } else {
                                            "unknown panic".to_string()
                                        };
                                        eprintln!("PANIC caught in request handler: {}", msg);
                                        McpResponse::error(
                                            -32603,
                                            format!("Internal server error: {}", msg),
                                        )
                                    });
                                trace!("Handler returned, preparing response");

                                // Wrap response in proper JSON-RPC 2.0 format
                                let message = McpMessage {
                                    jsonrpc: "2.0".to_string(),
                                    id: Some(request_id.clone()),
                                    content: McpMessageContent::Response(response),
                                };

                                trace!("Serializing response");
                                let response_json = serde_json::to_string(&message)?;
                                trace!("Writing response to stdout: {} bytes", response_json.len());
                                stdout.write_all(response_json.as_bytes()).await?;
                                stdout.write_all(b"\n").await?;
                                trace!("Flushing stdout");
                                stdout.flush().await?;
                                trace!("Response sent successfully");
                            }
                            Err(e) => {
                                error!("Failed to parse request: {}", e);

                                // Send a JSON-RPC error response to stdout for invalid methods
                                let error_response = McpResponse::Error {
                                    error: format!("Invalid method or malformed request: {}", e),
                                };

                                // Wrap error in proper JSON-RPC 2.0 format
                                let message = McpMessage {
                                    jsonrpc: "2.0".to_string(),
                                    id: Some(request_id.clone()),
                                    content: McpMessageContent::Response(error_response),
                                };

                                let response_json = serde_json::to_string(&message)?;
                                stdout.write_all(response_json.as_bytes()).await?;
                                stdout.write_all(b"\n").await?;
                                stdout.flush().await?;
                            }
                        }
                    } else {
                        // This is a notification - parse as McpNotification and handle without response
                        match serde_json::from_value::<McpNotification>(parsed) {
                            Ok(notification) => {
                                self.handle_notification(notification).await;
                                // Notifications don't require responses
                            }
                            Err(e) => {
                                error!("Failed to parse notification: {}", e);
                                // For notifications, we don't send error responses
                            }
                        }
                    }
                }
            }
        }

        // Cleanup all sessions before shutting down
        self.cleanup().await;
        Ok(())
    }

    pub(super) async fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        info!("Handling request: {:?}", request);

        match request {
            McpRequest::Initialize { .. } => {
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: serde_json::json!({
                        "tools": {}
                    }),
                    server_info: serde_json::json!({
                        "name": "thalora-mcp-server",
                        "version": "1.0.0"
                    }),
                };
                McpResponse::Initialize { result }
            }
            McpRequest::ListTools => {
                use crate::protocols::mcp::ListToolsResult;
                McpResponse::ListTools {
                    result: ListToolsResult {
                        tools: self.get_tool_definitions(),
                    },
                }
            }
            McpRequest::CallTool { params } => {
                let tool_name = params.name;
                let arguments = params.arguments;
                self.call_tool(tool_name.to_string(), arguments).await
            }
        }
    }

    pub(super) async fn handle_notification(&mut self, notification: McpNotification) {
        info!("Handling notification: {:?}", notification);

        match notification {
            McpNotification::Cancelled { .. } => {
                // Handle cancellation notification
                // For now, we just log it - in a more complex implementation
                // we might track and cancel ongoing operations
                info!("Received cancellation notification");
            }
            McpNotification::Initialized { .. } => {
                // Handle initialization complete notification
                info!("Received initialization notification");
            }
        }
    }
}
