use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use vfs::VfsInstance;

use crate::protocols::mcp::{McpRequest, McpResponse, InitializeResult};
use crate::engine::browser::HeadlessWebBrowser;
use crate::apis::websocket::WebSocketManager;
// DOM is now natively handled by Boa engine
use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::cdp::CdpServer;
use crate::protocols::memory_tools::MemoryTools;
use crate::protocols::cdp_tools::CdpTools;
use crate::protocols::browser_tools::BrowserTools;

pub struct McpServer {
    pub(super) browser: Arc<Mutex<HeadlessWebBrowser>>,
    pub(super) websocket_manager: WebSocketManager,
    pub(super) ai_memory: AiMemoryHeap,
    pub(super) cdp_server: CdpServer,
    pub(super) memory_tools: MemoryTools,
    pub(super) cdp_tools: CdpTools,
    pub(super) browser_tools: BrowserTools,
    /// Optional session-scoped persistent VFS instances. Keyed by session_id.
    pub(super) session_vfs: Arc<Mutex<HashMap<String, Arc<VfsInstance>>>>,
}

impl McpServer {
    pub fn new() -> Self {
        let ai_memory = AiMemoryHeap::new_default().unwrap_or_else(|_| {
            tracing::warn!("Failed to load AI memory heap, creating new one");
            AiMemoryHeap::new("/tmp/thalora_ai_memory.json").expect("Failed to create AI memory heap")
        });

        let browser = HeadlessWebBrowser::new();
        let cdp_tools = CdpTools::with_browser(browser.clone());

        Self {
            browser,
            websocket_manager: WebSocketManager::new(),
            ai_memory,
            cdp_server: CdpServer::new(),
            memory_tools: MemoryTools::new(),
            cdp_tools,
            browser_tools: BrowserTools::new(),
            session_vfs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a session-scoped VFS. If `backing_dir` is provided, the backing file will be created there.
    pub fn get_or_create_session_vfs(&self, session_id: &str, backing_dir: Option<&std::path::Path>) -> Arc<VfsInstance> {
        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.get(session_id) {
            return v.clone();
        }

        // Build a backing path: backing_dir/vfs-<session_id>.bin
        let file = if let Some(dir) = backing_dir {
            dir.join(format!("vfs-session-{}.bin", session_id))
        } else {
            // fallback to temp dir
            std::env::temp_dir().join(format!("vfs-session-{}.bin", session_id))
        };

        // Try to open existing or create new
        let v = match VfsInstance::open_file_backed(&file) {
            Ok(inst) => Arc::new(inst),
            Err(_) => Arc::new(VfsInstance::new_temp_in_dir(std::env::temp_dir()).expect("create temp vfs")),
        };
        guard.insert(session_id.to_string(), v.clone());
        v
    }

    /// Remove and optionally delete the backing file for a session VFS.
    pub fn remove_session_vfs(&self, session_id: &str, delete_backing: bool) {
        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.remove(session_id) {
            if delete_backing {
                drop(v.delete_backing_file());
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<McpRequest>(line) {
                        Ok(request) => {
                            let response = self.handle_request(request).await;
                            let response_json = serde_json::to_string(&response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Err(e) => {
                            error!("Failed to parse request: {}", e);

                            // Send a JSON-RPC error response to stdout for invalid methods
                            let error_response = McpResponse::Error {
                                error: format!("Invalid method or malformed request: {}", e),
                            };

                            let response_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;

                            // Also log to stderr for debugging
                            let mut stderr = tokio::io::stderr();
                            let error_msg = format!("Parse error: {}\n", e);
                            stderr.write_all(error_msg.as_bytes()).await?;
                            stderr.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }

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
                McpResponse::success(serde_json::json!({
                    "tools": self.get_tool_definitions()
                }))
            }
            McpRequest::CallTool { params } => {
                let tool_name = params.name;
                let arguments = params.arguments;
                self.call_tool(tool_name.to_string(), arguments).await
            }
        }
    }
}