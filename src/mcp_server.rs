use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};

use crate::mcp::{McpRequest, McpResponse};
use crate::browser::HeadlessWebBrowser;
use crate::websocket::WebSocketManager;
use crate::enhanced_dom::DomManager;
use crate::ai_memory::AiMemoryHeap;
use crate::cdp::CdpServer;
use crate::memory_tools::MemoryTools;
use crate::cdp_tools::CdpTools;

pub struct McpServer {
    pub browser: HeadlessWebBrowser,
    pub websocket_manager: WebSocketManager,
    pub dom_manager: Option<DomManager>,
    pub ai_memory: AiMemoryHeap,
    pub cdp_server: CdpServer,
    pub memory_tools: MemoryTools,
    pub cdp_tools: CdpTools,
}

impl McpServer {
    pub fn new() -> Self {
        let ai_memory = AiMemoryHeap::new_default().unwrap_or_else(|_| {
            tracing::warn!("Failed to load AI memory heap, creating new one");
            AiMemoryHeap::new("/tmp/synaptic_ai_memory.json").expect("Failed to create AI memory heap")
        });
        
        Self {
            browser: HeadlessWebBrowser::new(),
            websocket_manager: WebSocketManager::new(),
            dom_manager: None,
            ai_memory,
            cdp_server: CdpServer::new(),
            memory_tools: MemoryTools::new(),
            cdp_tools: CdpTools::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        let mut line = String::new();

        info!("MCP Web Scraper starting...");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(request) = serde_json::from_str::<McpRequest>(&line.trim()) {
                        let response = self.handle_request(request).await;
                        let response_json = serde_json::to_string(&response)?;
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        match request {
            McpRequest::ListTools => self.list_tools(),
            McpRequest::CallTool { params } => {
                self.call_tool(params.name, params.arguments).await
            }
            McpRequest::Initialize { .. } => McpResponse::Initialize {
                protocol_version: "2024-11-05".to_string(),
                capabilities: serde_json::json!({
                    "tools": {}
                }),
                server_info: serde_json::json!({
                    "name": "brainwires-scraper",
                    "version": "0.1.0"
                }),
            },
        }
    }

    async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        match name.as_str() {
            // Memory tools - delegate to memory_tools module
            "memory_store_research" => self.memory_tools.store_research(arguments, &mut self.ai_memory).await,
            "memory_store_credentials" => self.memory_tools.store_credentials(arguments, &mut self.ai_memory).await,
            "memory_get_credentials" => self.memory_tools.get_credentials(arguments, &mut self.ai_memory).await,
            "memory_store_bookmark" => self.memory_tools.store_bookmark(arguments, &mut self.ai_memory).await,
            "memory_store_note" => self.memory_tools.store_note(arguments, &mut self.ai_memory).await,
            "memory_search" => self.memory_tools.search(arguments, &mut self.ai_memory).await,
            "memory_start_session" => self.memory_tools.start_session(arguments, &mut self.ai_memory).await,
            "memory_update_session" => self.memory_tools.update_session(arguments, &mut self.ai_memory).await,
            "memory_get_statistics" => self.memory_tools.get_statistics(arguments, &mut self.ai_memory).await,

            // CDP tools - delegate to cdp_tools module  
            "cdp_enable_runtime" => self.cdp_tools.enable_runtime(arguments, &mut self.cdp_server).await,
            "cdp_evaluate_javascript" => self.cdp_tools.evaluate_javascript(arguments, &mut self.cdp_server).await,
            "cdp_enable_debugger" => self.cdp_tools.enable_debugger(arguments, &mut self.cdp_server).await,
            "cdp_set_breakpoint" => self.cdp_tools.set_breakpoint(arguments, &mut self.cdp_server).await,
            "cdp_enable_dom" => self.cdp_tools.enable_dom(arguments, &mut self.cdp_server).await,
            "cdp_get_document" => self.cdp_tools.get_document(arguments, &mut self.cdp_server).await,
            "cdp_enable_network" => self.cdp_tools.enable_network(arguments, &mut self.cdp_server).await,
            "cdp_get_response_body" => self.cdp_tools.get_response_body(arguments, &mut self.cdp_server).await,
            
            _ => McpResponse::Error {
                error: format!("Unknown tool: {}", name),
            },
        }
    }

    fn list_tools(&self) -> McpResponse {
        McpResponse::ListTools {
            tools: vec![
                // AI Memory Heap capabilities
                serde_json::json!({
                    "name": "memory_store_research",
                    "description": "Store research findings in AI memory heap for persistent access",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Unique identifier for the research entry"
                            },
                            "topic": {
                                "type": "string",
                                "description": "Research topic or subject"
                            },
                            "summary": {
                                "type": "string",
                                "description": "Brief summary of findings"
                            },
                            "findings": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Key findings and discoveries"
                            },
                            "sources": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Source URLs and references"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags for categorization"
                            },
                            "confidence_score": {
                                "type": "number",
                                "description": "Confidence in findings (0.0-1.0)",
                                "minimum": 0.0,
                                "maximum": 1.0
                            },
                            "related_topics": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Related research topics"
                            }
                        },
                        "required": ["key", "topic", "summary"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_store_credentials",
                    "description": "Securely store credentials in AI memory heap with encryption",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Unique identifier for the credential"
                            },
                            "service": {
                                "type": "string",
                                "description": "Service name or description"
                            },
                            "username": {
                                "type": "string",
                                "description": "Username or email"
                            },
                            "password": {
                                "type": "string",
                                "description": "Password or token (will be encrypted)"
                            },
                            "additional_data": {
                                "type": "object",
                                "description": "Additional metadata as key-value pairs"
                            }
                        },
                        "required": ["key", "service", "username", "password"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_get_credentials",
                    "description": "Retrieve stored credentials from AI memory heap",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Credential identifier"
                            }
                        },
                        "required": ["key"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_store_bookmark",
                    "description": "Store a bookmark in AI memory heap for future reference",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Unique identifier for the bookmark"
                            },
                            "url": {
                                "type": "string",
                                "description": "URL to bookmark"
                            },
                            "title": {
                                "type": "string",
                                "description": "Page title or custom name"
                            },
                            "description": {
                                "type": "string",
                                "description": "Description of the bookmarked resource"
                            },
                            "content_preview": {
                                "type": "string",
                                "description": "Brief content preview or snippet"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags for categorization"
                            }
                        },
                        "required": ["key", "url", "title"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_store_note",
                    "description": "Store a note in AI memory heap with categorization and priority",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Unique identifier for the note"
                            },
                            "title": {
                                "type": "string",
                                "description": "Note title"
                            },
                            "content": {
                                "type": "string",
                                "description": "Note content"
                            },
                            "category": {
                                "type": "string",
                                "description": "Category for organization"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags for categorization"
                            },
                            "priority": {
                                "type": "string",
                                "enum": ["Low", "Medium", "High", "Critical"],
                                "description": "Priority level"
                            }
                        },
                        "required": ["key", "title", "content"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_search",
                    "description": "Search through AI memory heap entries with various criteria",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query text"
                            },
                            "category": {
                                "type": "string",
                                "enum": ["research", "credentials", "bookmarks", "notes", "sessions"],
                                "description": "Category to search in"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags to match"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum results to return"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "memory_start_session",
                    "description": "Start a new development session in AI memory heap",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Unique session identifier"
                            },
                            "description": {
                                "type": "string",
                                "description": "Session description"
                            },
                            "objectives": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Session objectives and tasks"
                            }
                        },
                        "required": ["session_id", "description"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_update_session",
                    "description": "Update progress and metadata for an active session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Session identifier"
                            },
                            "progress_key": {
                                "type": "string",
                                "description": "Progress field to update"
                            },
                            "progress_value": {
                                "description": "New progress value (any type)"
                            }
                        },
                        "required": ["session_id"]
                    }
                }),
                serde_json::json!({
                    "name": "memory_get_statistics",
                    "description": "Get AI memory heap statistics and usage information",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                // Chrome DevTools Protocol (CDP) debugging capabilities
                serde_json::json!({
                    "name": "cdp_enable_runtime",
                    "description": "Enable CDP Runtime domain for JavaScript execution and debugging",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "cdp_evaluate_javascript",
                    "description": "Evaluate JavaScript code using CDP Runtime.evaluate",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "expression": {
                                "type": "string",
                                "description": "JavaScript expression to evaluate"
                            },
                            "return_by_value": {
                                "type": "boolean",
                                "description": "Whether to return result by value",
                                "default": true
                            },
                            "generate_preview": {
                                "type": "boolean",
                                "description": "Whether to generate object preview",
                                "default": false
                            }
                        },
                        "required": ["expression"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_enable_debugger",
                    "description": "Enable CDP Debugger domain for breakpoint management",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "cdp_set_breakpoint",
                    "description": "Set a breakpoint at specified line using CDP Debugger",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "line_number": {
                                "type": "integer",
                                "description": "Line number to set breakpoint on"
                            },
                            "url": {
                                "type": "string",
                                "description": "Optional URL/script ID to set breakpoint in"
                            },
                            "condition": {
                                "type": "string",
                                "description": "Optional breakpoint condition expression"
                            }
                        },
                        "required": ["line_number"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_enable_dom",
                    "description": "Enable CDP DOM domain for document inspection",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "cdp_get_document",
                    "description": "Get the root DOM node using CDP DOM.getDocument",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "depth": {
                                "type": "integer",
                                "description": "Maximum depth to retrieve",
                                "default": 1
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "cdp_enable_network",
                    "description": "Enable CDP Network domain for request/response monitoring",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "cdp_get_response_body",
                    "description": "Get response body for a network request using CDP",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "request_id": {
                                "type": "string",
                                "description": "Network request ID"
                            }
                        },
                        "required": ["request_id"]
                    }
                })
            ]
        }
    }
}