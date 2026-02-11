use anyhow::Result;
use tracing::info;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use vfs::VfsInstance;

use crate::protocols::mcp::{McpRequest, McpNotification, McpResponse, InitializeResult};
// websocket API is now natively implemented in Boa engine
// DOM is now natively handled by Boa engine
use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::cdp::CdpServer;
use crate::protocols::memory_tools::MemoryTools;
use crate::protocols::cdp_tools::CdpTools;
use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::session_manager::SessionManager;
use crate::protocols::security::sanitize_session_id;
use crate::protocols::rate_limiter::RateLimiter;
use crate::engine::EngineConfig;

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
}

impl McpServer {
    pub fn new() -> Self {
        // Default to Boa engine for backward compatibility
        Self::new_with_engine(EngineConfig::new())
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
            tracing::info!("AI memory disabled. Set THALORA_ENABLE_AI_MEMORY=1 to enable persistent storage");
            AiMemoryHeap::new("/dev/null")
                .unwrap_or_else(|_| {
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
        }
    }

    /// Get or create a session-scoped VFS. If `backing_dir` is provided, the backing file will be created there.
    ///
    /// # Security
    /// - The session_id is validated to prevent path traversal attacks (CWE-22).
    /// - Session data is encrypted at rest using ChaCha20-Poly1305.
    /// - Only alphanumeric characters, hyphens, and underscores are allowed.
    pub fn get_or_create_session_vfs(&self, session_id: &str, backing_dir: Option<&std::path::Path>) -> Result<Arc<VfsInstance>> {
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
                Arc::new(VfsInstance::new_temp_in_dir(std::env::temp_dir()).expect("create temp vfs"))
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

    /// Run the MCP server using stdio transport (default, backward-compatible).
    pub async fn run(&mut self) -> Result<()> {
        self.run_stdio().await
    }

    /// Run the MCP server using stdio transport.
    /// Reads JSON-RPC from stdin, writes responses to stdout.
    pub async fn run_stdio(&mut self) -> Result<()> {
        super::transport::stdio::run_stdio(self).await
    }

    /// Run the MCP server using HTTP transport.
    /// Consumes self because it gets wrapped in `Arc<tokio::sync::Mutex<McpServer>>`.
    pub async fn run_http(self, host: &str, port: u16) -> Result<()> {
        super::transport::http::run_http(self, host, port).await
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
                        tools: self.get_tool_definitions()
                    }
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