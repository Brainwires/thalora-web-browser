use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};

use crate::protocols::mcp::{McpRequest, McpResponse, McpMessage, McpMessageContent, InitializeResult};
use crate::engine::browser::HeadlessWebBrowser;
use std::sync::{Arc, Mutex};
use crate::apis::websocket::WebSocketManager;
use crate::engine::dom::EnhancedDom;
use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::cdp::CdpServer;
use crate::protocols::memory_tools::MemoryTools;
use crate::protocols::cdp_tools::CdpTools;
use crate::protocols::browser_tools::BrowserTools;

pub struct McpServer {
    pub browser: Arc<Mutex<HeadlessWebBrowser>>,
    pub websocket_manager: WebSocketManager,
    pub dom_manager: Option<EnhancedDom>,
    pub ai_memory: AiMemoryHeap,
    pub cdp_server: CdpServer,
    pub memory_tools: MemoryTools,
    pub cdp_tools: CdpTools,
    pub browser_tools: BrowserTools,
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
            dom_manager: None,
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
        let mut line = String::new();

        info!("MCP Web Scraper starting...");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(message) = serde_json::from_str::<McpMessage>(&line.trim()) {
                        let response = match message.content {
                            McpMessageContent::Request(request) => self.handle_request(request).await,
                            McpMessageContent::Response(_) => {
                                // This is a response, not a request - shouldn't happen in server context
                                continue;
                            }
                        };

                        let response_message = McpMessage {
                            jsonrpc: "2.0".to_string(),
                            id: message.id,
                            content: McpMessageContent::Response(response),
                        };

                        let response_json = serde_json::to_string(&response_message)?;
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
                result: InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: serde_json::json!({
                        "tools": {}
                    }),
                    server_info: serde_json::json!({
                        "name": "brainwires-scraper",
                        "version": "0.1.0"
                    }),
                }
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
            
            // Browser session management tools
            "browser_create_session" => self.browser_tools.create_session(arguments, &self.browser).await,
            "browser_get_session" => self.browser_tools.get_session(arguments, &self.browser).await,
            "browser_close_session" => self.browser_tools.close_session(arguments, &self.browser).await,
            "browser_list_sessions" => self.browser_tools.list_sessions(arguments, &self.browser).await,
            "browser_close_all_sessions" => self.browser_tools.close_all_sessions(arguments, &self.browser).await,
            "browser_navigate" => self.browser_tools.navigate(arguments, &self.browser).await,
            "browser_find_elements" => self.browser_tools.find_elements(arguments, &self.browser).await,
            "browser_get_page_state" => self.browser_tools.get_page_state(arguments, &self.browser).await,
            "browser_fill_form" => self.browser_tools.fill_form(arguments, &self.browser).await,
            "browser_click_element" => self.browser_tools.click_element(arguments, &self.browser).await,
            "browser_execute_javascript" => self.browser_tools.execute_javascript(arguments, &self.browser).await,
            "browser_wait_for_element" => self.browser_tools.wait_for_element(arguments, &self.browser).await,

            // Web scraping tools (legacy)
            "scrape_url" => self.scrape_url(arguments).await,
            "web_search" => self.google_search(arguments).await,

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
                }),
                // Browser session management tools
                serde_json::json!({
                    "name": "browser_create_session",
                    "description": "Create a new browser session for persistent interactions",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Unique identifier for the session (optional, auto-generated if not provided)"
                            },
                            "persistent": {
                                "type": "boolean",
                                "description": "Whether to maintain session state across interactions",
                                "default": false
                            },
                            "description": {
                                "type": "string",
                                "description": "Description of the session purpose"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "browser_get_session",
                    "description": "Get information about a browser session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Session identifier"
                            }
                        },
                        "required": ["session_id"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_close_session",
                    "description": "Close and cleanup a browser session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Session identifier"
                            }
                        },
                        "required": ["session_id"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_list_sessions",
                    "description": "List all active browser sessions with their details",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "browser_close_all_sessions",
                    "description": "Close and cleanup all active browser sessions",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }),
                serde_json::json!({
                    "name": "browser_navigate",
                    "description": "Navigate to a URL within a browser session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to navigate to"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Optional session identifier"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to wait for JavaScript execution",
                                "default": true
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_find_elements",
                    "description": "Find elements on the current page using CSS selectors",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector to find elements"
                            },
                            "url": {
                                "type": "string",
                                "description": "URL to search on (optional, uses current page if not provided)"
                            }
                        },
                        "required": ["selector"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_get_page_state",
                    "description": "Get comprehensive page state including forms, links, and content",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to analyze (optional, uses current page if not provided)"
                            },
                            "include_forms": {
                                "type": "boolean",
                                "description": "Whether to extract form data",
                                "default": true
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "browser_fill_form",
                    "description": "Fill and submit a form on the current page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "form_data": {
                                "type": "object",
                                "description": "Key-value pairs of form field names and values"
                            },
                            "url": {
                                "type": "string",
                                "description": "URL containing the form"
                            },
                            "form_selector": {
                                "type": "string",
                                "description": "CSS selector for the form (optional, uses first form if not provided)",
                                "default": "form"
                            }
                        },
                        "required": ["form_data", "url"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_click_element",
                    "description": "Click on an element (link, button, etc.)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector for the element to click"
                            },
                            "url": {
                                "type": "string",
                                "description": "URL containing the element"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to wait for JavaScript execution after click",
                                "default": true
                            }
                        },
                        "required": ["selector", "url"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_execute_javascript",
                    "description": "Execute JavaScript code in the browser context",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "code": {
                                "type": "string",
                                "description": "JavaScript code to execute"
                            }
                        },
                        "required": ["code"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_wait_for_element",
                    "description": "Wait for an element to appear or disappear on the page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector for the element to wait for"
                            },
                            "url": {
                                "type": "string",
                                "description": "URL to monitor (optional, uses current page if not provided)"
                            },
                            "timeout_ms": {
                                "type": "integer",
                                "description": "Timeout in milliseconds",
                                "default": 5000
                            },
                            "expect_visible": {
                                "type": "boolean",
                                "description": "Whether to wait for element to be visible (true) or hidden (false)",
                                "default": true
                            }
                        },
                        "required": ["selector"]
                    }
                }),
                // Web scraping tools (legacy)
                serde_json::json!({
                    "name": "scrape_url",
                    "description": "Scrape a web page and extract content, links, images, and metadata",
                    "inputSchema": {
                        "type": "object", 
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to scrape"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to execute JavaScript before scraping",
                                "default": true
                            },
                            "selector": {
                                "type": "string",
                                "description": "Optional CSS selector to focus on specific content"
                            },
                            "extract_links": {
                                "type": "boolean", 
                                "description": "Whether to extract links from the page",
                                "default": true
                            },
                            "extract_images": {
                                "type": "boolean",
                                "description": "Whether to extract images from the page", 
                                "default": true
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "web_search",
                    "description": "Perform a web search by submitting a query and returning search results",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query to submit"
                            },
                            "num_results": {
                                "type": "integer",
                                "description": "Maximum number of search results to return",
                                "default": 10,
                                "minimum": 1,
                                "maximum": 100
                            }
                        },
                        "required": ["query"]
                    }
                })
            ]
        }
    }

    async fn scrape_url(&mut self, arguments: Value) -> McpResponse {
        let url = match arguments.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: url"
                    })],
                    is_error: true,
                };
            }
        };

        let wait_for_js = arguments.get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let selector = arguments.get("selector").and_then(|v| v.as_str());

        let extract_links = arguments.get("extract_links")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let extract_images = arguments.get("extract_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Setup history API for JavaScript-enabled scraping
        if wait_for_js {
            if let Err(e) = HeadlessWebBrowser::setup_history_api(Arc::clone(&self.browser)) {
                tracing::warn!("Failed to setup history API: {}", e);
            }
        }

        match self.browser.lock().unwrap().scrape(url, wait_for_js, selector, extract_links, extract_images).await {
            Ok(scraped_data) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&scraped_data).unwrap_or_else(|_| "Failed to serialize scraped data".to_string())
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text", 
                        "text": format!("Failed to scrape URL {}: {}", url, e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    async fn google_search(&mut self, arguments: Value) -> McpResponse {
        let query = match arguments.get("query").and_then(|v| v.as_str()) {
            Some(query) => query,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: query"
                    })],
                    is_error: true,
                };
            }
        };

        let num_results = arguments.get("num_results")
            .and_then(|v| v.as_i64())
            .unwrap_or(10) as usize;

        match self.perform_google_search(query, num_results).await {
            Ok(results) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&results).unwrap_or_else(|_| "Failed to serialize search results".to_string())
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("Failed to perform Google search for '{}': {}", query, e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    async fn perform_google_search(&mut self, query: &str, num_results: usize) -> anyhow::Result<SearchResults> {
        use tokio::time::{sleep, Duration};

        // Simple direct search - same approach that works in our tests
        let search_url = format!(
            "https://www.google.com/search?q={}",
            query.replace(' ', "+")
        );

        // Use the same parameters that work in our tests
        let mut search_data = self.browser.lock().unwrap().scrape(&search_url, false, None, false, false).await?;

        // Debug: Log the response details to stderr (MCP protocol requirement)
        tracing::debug!("Google response length: {} chars", search_data.content.len());
        tracing::debug!("First 500 chars: {}", &search_data.content[..search_data.content.len().min(500)]);

        // Check if we got the JavaScript challenge page
        if search_data.content.contains("enablejs") && search_data.content.contains("httpservice/retry") {
            tracing::debug!("Got JavaScript challenge, following redirect...");

            // Extract the redirect URL from the meta refresh tag
            if let Some(start) = search_data.content.find("/httpservice/retry/enablejs") {
                if let Some(end) = search_data.content[start..].find("\"") {
                    let redirect_path = &search_data.content[start..start + end];
                    let redirect_url = format!("https://www.google.com{}", redirect_path);

                    tracing::debug!("Following redirect to: {}", redirect_url);

                    // Follow the redirect with JavaScript enabled to handle the challenge
                    let _challenge_response = self.browser.lock().unwrap().scrape(&redirect_url, true, None, false, false).await?;

                    // Wait a moment for any JavaScript to execute
                    sleep(Duration::from_millis(2000)).await;

                    // Now retry the original search - should get real results
                    tracing::debug!("Retrying original search after challenge...");
                    search_data = self.browser.lock().unwrap().scrape(&search_url, true, None, false, false).await?;

                    tracing::debug!("After challenge - response length: {} chars", search_data.content.len());
                    tracing::debug!("After challenge - first 500 chars: {}", &search_data.content[..search_data.content.len().min(500)]);
                }
            }
        }

        // Step 5: Parse the search results
        let mut results = self.parse_google_search_results(&search_data.content, num_results).await?;
        results.query = query.to_string();
        Ok(results)
    }

    async fn parse_google_search_results(&self, html: &str, num_results: usize) -> anyhow::Result<SearchResults> {
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
        
        // Google search result selectors
        let result_selector = Selector::parse("div.g, div.MjjYud").map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;
        let title_selector = Selector::parse("h3").map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;
        let link_selector = Selector::parse("a").map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;
        let snippet_selector = Selector::parse(".VwiC3b, .s3v9rd, .st").map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;

        let mut results = Vec::new();
        
        for result_elem in document.select(&result_selector).take(num_results) {
            // Extract title
            let title = result_elem.select(&title_selector)
                .next()
                .map(|elem| elem.text().collect::<String>())
                .unwrap_or_default();

            if title.is_empty() {
                continue; // Skip results without titles
            }

            // Extract URL - look for the first link in the result
            let url = result_elem.select(&link_selector)
                .next()
                .and_then(|elem| elem.value().attr("href"))
                .map(|href| {
                    if href.starts_with("/url?") {
                        // Google redirects - extract the actual URL
                        if let Ok(parsed) = url::Url::parse(&format!("https://google.com{}", href)) {
                            if let Some(actual_url) = parsed.query_pairs().find(|(key, _)| key == "url") {
                                return actual_url.1.to_string();
                            }
                        }
                        href.to_string()
                    } else if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://google.com{}", href)
                    }
                })
                .unwrap_or_default();

            // Extract snippet
            let snippet = result_elem.select(&snippet_selector)
                .next()
                .map(|elem| elem.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();

            if !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url,
                    snippet,
                });
            }
        }

        let total_results = results.len();
        Ok(SearchResults {
            query: "".to_string(), // We'll set this in the calling function
            results,
            total_results,
        })
    }
}

#[derive(Debug, serde::Serialize)]
struct SearchResults {
    query: String,
    results: Vec<SearchResult>,
    total_results: usize,
}

#[derive(Debug, serde::Serialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}