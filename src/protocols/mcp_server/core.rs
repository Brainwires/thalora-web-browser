use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};
use std::sync::{Arc, Mutex};

use crate::protocols::mcp::{McpRequest, McpResponse, McpMessage, McpMessageContent, InitializeResult};
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
}

impl McpServer {
    pub fn new() -> Self {
        let ai_memory = AiMemoryHeap::new_default().unwrap_or_else(|_| {
            tracing::warn!("Failed to load AI memory heap, creating new one");
            AiMemoryHeap::new("/tmp/thalora_ai_memory.json").expect("Failed to create AI memory heap")
        });

        Self {
            browser: HeadlessWebBrowser::new(),
            websocket_manager: WebSocketManager::new(),
            ai_memory,
            cdp_server: CdpServer::new(),
            memory_tools: MemoryTools::new(),
            cdp_tools: CdpTools::new(),
            browser_tools: BrowserTools::new(),
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
                            let error_response = McpResponse::error(-1, format!("Parse error: {}", e));
                            let response_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
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
                McpResponse::success(serde_json::to_value(result).unwrap())
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