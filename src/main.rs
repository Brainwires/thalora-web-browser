use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};

mod mcp;
mod browser;
mod renderer;
mod react_processor;
mod websocket;
mod enhanced_dom;

use mcp::{McpRequest, McpResponse};
use browser::HeadlessWebBrowser;
use websocket::WebSocketManager;
use enhanced_dom::DomManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    tracing::info!("🧠 Synaptic v0.1.0 - Pure Rust headless browser for AI models");
    tracing::info!("🔗 Neural connections between AI and the web");
    
    let mut server = McpServer::new();
    server.run().await
}

struct McpServer {
    browser: HeadlessWebBrowser,
    websocket_manager: WebSocketManager,
    dom_manager: Option<DomManager>,
}

impl McpServer {
    fn new() -> Self {
        Self {
            browser: HeadlessWebBrowser::new(),
            websocket_manager: WebSocketManager::new(),
            dom_manager: None,
        }
    }

    async fn run(&mut self) -> Result<()> {
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

    fn list_tools(&self) -> McpResponse {
        McpResponse::ListTools {
            tools: vec![
                // Core scraping capabilities
                serde_json::json!({
                    "name": "scrape_url",
                    "description": "Scrape content from a URL with advanced browser simulation and stealth features",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to scrape"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to execute JavaScript and wait for dynamic content",
                                "default": true
                            },
                            "selector": {
                                "type": "string",
                                "description": "Optional CSS selector to extract specific elements"
                            },
                            "extract_links": {
                                "type": "boolean", 
                                "description": "Whether to extract all links from the page",
                                "default": false
                            },
                            "extract_images": {
                                "type": "boolean",
                                "description": "Whether to extract all images from the page", 
                                "default": false
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "extract_data",
                    "description": "Extract structured data from HTML using CSS selectors",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "html": {
                                "type": "string",
                                "description": "The HTML content to parse"
                            },
                            "selectors": {
                                "type": "object",
                                "description": "Object mapping field names to CSS selectors"
                            }
                        },
                        "required": ["html", "selectors"]
                    }
                }),
                // Form interaction capabilities
                serde_json::json!({
                    "name": "extract_forms",
                    "description": "Extract all forms from a webpage for analysis or interaction",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to extract forms from"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "submit_form",
                    "description": "Submit a form with specified data",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "form_action": {
                                "type": "string",
                                "description": "The form action URL"
                            },
                            "method": {
                                "type": "string",
                                "description": "HTTP method (GET or POST)",
                                "enum": ["GET", "POST"]
                            },
                            "form_data": {
                                "type": "object",
                                "description": "Form field data as key-value pairs"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to wait for JavaScript execution after submission",
                                "default": true
                            }
                        },
                        "required": ["form_action", "method", "form_data"]
                    }
                }),
                serde_json::json!({
                    "name": "click_link",
                    "description": "Click a link on a webpage and follow the navigation",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "base_url": {
                                "type": "string",
                                "description": "The current page URL"
                            },
                            "link_selector": {
                                "type": "string",
                                "description": "CSS selector for the link to click"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to wait for JavaScript execution after clicking",
                                "default": false
                            }
                        },
                        "required": ["base_url", "link_selector"]
                    }
                }),
                // Browser state and storage management
                serde_json::json!({
                    "name": "manage_cookies",
                    "description": "Get or manage cookies for a domain",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to get cookies for"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "manage_storage",
                    "description": "Interact with browser localStorage and sessionStorage",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "description": "Action to perform",
                                "enum": ["get_local", "set_local", "get_session", "set_session", "clear_session"]
                            },
                            "key": {
                                "type": "string",
                                "description": "Storage key (required for get/set operations)"
                            },
                            "value": {
                                "type": "string",
                                "description": "Value to set (required for set operations)"
                            }
                        },
                        "required": ["action"]
                    }
                }),
                // Authentication and headers
                serde_json::json!({
                    "name": "manage_auth",
                    "description": "Manage authentication tokens and custom headers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "description": "Authentication action",
                                "enum": ["set_bearer", "get_bearer", "set_csrf", "get_csrf", "set_header", "get_headers"]
                            },
                            "token": {
                                "type": "string",
                                "description": "Token value (for set operations)"
                            },
                            "header_name": {
                                "type": "string",
                                "description": "Header name (for set_header)"
                            },
                            "header_value": {
                                "type": "string",
                                "description": "Header value (for set_header)"
                            }
                        },
                        "required": ["action"]
                    }
                }),
                // WebSocket capabilities
                serde_json::json!({
                    "name": "websocket_connect",
                    "description": "Create a WebSocket connection for real-time communication",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "WebSocket URL to connect to"
                            },
                            "protocols": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Optional WebSocket sub-protocols"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "websocket_send",
                    "description": "Send a message through an active WebSocket connection",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "connection_id": {
                                "type": "string",
                                "description": "WebSocket connection ID"
                            },
                            "message": {
                                "type": "string",
                                "description": "Message to send"
                            },
                            "binary": {
                                "type": "boolean",
                                "description": "Whether to send as binary data",
                                "default": false
                            }
                        },
                        "required": ["connection_id", "message"]
                    }
                }),
                serde_json::json!({
                    "name": "websocket_simulate",
                    "description": "Simulate real-time events on a WebSocket connection",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "connection_id": {
                                "type": "string",
                                "description": "WebSocket connection ID"
                            },
                            "events": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Event types to simulate",
                                "default": ["heartbeat", "user_joined", "message"]
                            }
                        },
                        "required": ["connection_id"]
                    }
                }),
                // Advanced browser capabilities
                serde_json::json!({
                    "name": "browser_status",
                    "description": "Get comprehensive browser status and capabilities",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "include_websockets": {
                                "type": "boolean",
                                "description": "Include active WebSocket connections",
                                "default": true
                            },
                            "include_storage": {
                                "type": "boolean",
                                "description": "Include storage state",
                                "default": true
                            }
                        }
                    }
                })
            ]
        }
    }

    async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        match name.as_str() {
            // Core scraping
            "scrape_url" => self.scrape_url(arguments).await,
            "extract_data" => self.extract_data(arguments).await,
            
            // Form interaction
            "extract_forms" => self.extract_forms(arguments).await,
            "submit_form" => self.submit_form(arguments).await,
            "click_link" => self.click_link(arguments).await,
            
            // Browser state management
            "manage_cookies" => self.manage_cookies(arguments).await,
            "manage_storage" => self.manage_storage(arguments).await,
            "manage_auth" => self.manage_auth(arguments).await,
            
            // WebSocket capabilities
            "websocket_connect" => self.websocket_connect(arguments).await,
            "websocket_send" => self.websocket_send(arguments).await,
            "websocket_simulate" => self.websocket_simulate(arguments).await,
            
            // Advanced browser status
            "browser_status" => self.browser_status(arguments).await,
            
            _ => McpResponse::Error {
                error: format!("Unknown tool: {}", name),
            },
        }
    }

    async fn scrape_url(&mut self, args: Value) -> McpResponse {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: url".to_string(),
            },
        };

        let wait_for_js = args.get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let selector = args.get("selector")
            .and_then(|v| v.as_str());

        let extract_links = args.get("extract_links")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let extract_images = args.get("extract_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match self.browser.scrape(url, wait_for_js, selector, extract_links, extract_images).await {
            Ok(result) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Scraping failed: {}", e),
            },
        }
    }

    async fn extract_data(&mut self, args: Value) -> McpResponse {
        let html = match args.get("html").and_then(|v| v.as_str()) {
            Some(html) => html,
            None => return McpResponse::Error {
                error: "Missing required parameter: html".to_string(),
            },
        };

        let selectors = match args.get("selectors").and_then(|v| v.as_object()) {
            Some(selectors) => selectors,
            None => return McpResponse::Error {
                error: "Missing required parameter: selectors".to_string(),
            },
        };

        match self.browser.extract_data(html, selectors).await {
            Ok(result) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text", 
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Data extraction failed: {}", e),
            },
        }
    }

    // Form interaction methods
    async fn extract_forms(&mut self, args: Value) -> McpResponse {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: url".to_string(),
            },
        };

        // First scrape the page to get HTML
        match self.browser.scrape(url, false, None, false, false).await {
            Ok(scraped) => {
                match url::Url::parse(url) {
                    Ok(base_url) => {
                        match self.browser.extract_forms(&scraped.content, &base_url) {
                            Ok(forms) => McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&forms).unwrap_or_default()
                                })],
                                is_error: false,
                            },
                            Err(e) => McpResponse::Error {
                                error: format!("Form extraction failed: {}", e),
                            },
                        }
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Invalid URL: {}", e),
                    },
                }
            },
            Err(e) => McpResponse::Error {
                error: format!("Failed to scrape URL: {}", e),
            },
        }
    }

    async fn submit_form(&mut self, args: Value) -> McpResponse {
        let form_action = match args.get("form_action").and_then(|v| v.as_str()) {
            Some(action) => action.to_string(),
            None => return McpResponse::Error {
                error: "Missing required parameter: form_action".to_string(),
            },
        };

        let method = match args.get("method").and_then(|v| v.as_str()) {
            Some(method) => method.to_string(),
            None => return McpResponse::Error {
                error: "Missing required parameter: method".to_string(),
            },
        };

        let form_data = match args.get("form_data").and_then(|v| v.as_object()) {
            Some(data) => {
                let mut form_map = std::collections::HashMap::new();
                for (key, value) in data {
                    if let Some(val_str) = value.as_str() {
                        form_map.insert(key.clone(), val_str.to_string());
                    }
                }
                form_map
            },
            None => return McpResponse::Error {
                error: "Missing required parameter: form_data".to_string(),
            },
        };

        let wait_for_js = args.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(true);

        // Create a form object
        let form = browser::Form {
            action: form_action,
            method,
            fields: Vec::new(), // Simplified for MCP
            submit_buttons: vec!["Submit".to_string()],
        };

        match self.browser.submit_form(&form, form_data, wait_for_js).await {
            Ok(response) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&response).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Form submission failed: {}", e),
            },
        }
    }

    async fn click_link(&mut self, args: Value) -> McpResponse {
        let base_url = match args.get("base_url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: base_url".to_string(),
            },
        };

        let link_selector = match args.get("link_selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => return McpResponse::Error {
                error: "Missing required parameter: link_selector".to_string(),
            },
        };

        let wait_for_js = args.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(false);

        match self.browser.click_link(base_url, link_selector, wait_for_js).await {
            Ok(response) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&response).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Link clicking failed: {}", e),
            },
        }
    }

    // Browser state management methods
    async fn manage_cookies(&mut self, args: Value) -> McpResponse {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: url".to_string(),
            },
        };

        match self.browser.get_cookies(url) {
            Ok(cookies) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&cookies).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Cookie management failed: {}", e),
            },
        }
    }

    async fn manage_storage(&mut self, args: Value) -> McpResponse {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(action) => action,
            None => return McpResponse::Error {
                error: "Missing required parameter: action".to_string(),
            },
        };

        match action {
            "get_local" => {
                let key = match args.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: key".to_string(),
                    },
                };
                match self.browser.get_local_storage(key) {
                    Ok(value) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&value).unwrap_or_default()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to get localStorage: {}", e),
                    },
                }
            },
            "set_local" => {
                let key = match args.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: key".to_string(),
                    },
                };
                let value = match args.get("value").and_then(|v| v.as_str()) {
                    Some(value) => value,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: value".to_string(),
                    },
                };
                match self.browser.set_local_storage(key, value) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "localStorage item set successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to set localStorage: {}", e),
                    },
                }
            },
            "get_session" => {
                let key = match args.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: key".to_string(),
                    },
                };
                match self.browser.get_session_storage(key) {
                    Ok(value) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&value).unwrap_or_default()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to get sessionStorage: {}", e),
                    },
                }
            },
            "set_session" => {
                let key = match args.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: key".to_string(),
                    },
                };
                let value = match args.get("value").and_then(|v| v.as_str()) {
                    Some(value) => value,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: value".to_string(),
                    },
                };
                match self.browser.set_session_storage(key, value) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "sessionStorage item set successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to set sessionStorage: {}", e),
                    },
                }
            },
            "clear_session" => {
                match self.browser.clear_session_storage() {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "sessionStorage cleared successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to clear sessionStorage: {}", e),
                    },
                }
            },
            _ => McpResponse::Error {
                error: format!("Unknown storage action: {}", action),
            },
        }
    }

    async fn manage_auth(&mut self, args: Value) -> McpResponse {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(action) => action,
            None => return McpResponse::Error {
                error: "Missing required parameter: action".to_string(),
            },
        };

        match action {
            "set_bearer" => {
                let token = match args.get("token").and_then(|v| v.as_str()) {
                    Some(token) => token,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: token".to_string(),
                    },
                };
                match self.browser.set_bearer_token(token) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "Bearer token set successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to set bearer token: {}", e),
                    },
                }
            },
            "get_bearer" => {
                match self.browser.get_bearer_token() {
                    Ok(token) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&token).unwrap_or_default()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to get bearer token: {}", e),
                    },
                }
            },
            "set_csrf" => {
                let token = match args.get("token").and_then(|v| v.as_str()) {
                    Some(token) => token,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: token".to_string(),
                    },
                };
                match self.browser.set_csrf_token(token) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CSRF token set successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to set CSRF token: {}", e),
                    },
                }
            },
            "get_csrf" => {
                match self.browser.get_csrf_token() {
                    Ok(token) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&token).unwrap_or_default()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to get CSRF token: {}", e),
                    },
                }
            },
            "set_header" => {
                let name = match args.get("header_name").and_then(|v| v.as_str()) {
                    Some(name) => name,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: header_name".to_string(),
                    },
                };
                let value = match args.get("header_value").and_then(|v| v.as_str()) {
                    Some(value) => value,
                    None => return McpResponse::Error {
                        error: "Missing required parameter: header_value".to_string(),
                    },
                };
                match self.browser.set_custom_header(name, value) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "Custom header set successfully"
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to set custom header: {}", e),
                    },
                }
            },
            "get_headers" => {
                match self.browser.get_custom_headers() {
                    Ok(headers) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&headers).unwrap_or_default()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Failed to get custom headers: {}", e),
                    },
                }
            },
            _ => McpResponse::Error {
                error: format!("Unknown auth action: {}", action),
            },
        }
    }

    // WebSocket methods
    async fn websocket_connect(&mut self, args: Value) -> McpResponse {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: url".to_string(),
            },
        };

        let protocols = args.get("protocols")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

        match self.websocket_manager.connect(url, protocols).await {
            Ok(connection_id) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::json!({
                        "connection_id": connection_id,
                        "url": url,
                        "status": "connected"
                    }).to_string()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("WebSocket connection failed: {}", e),
            },
        }
    }

    async fn websocket_send(&mut self, args: Value) -> McpResponse {
        let connection_id = match args.get("connection_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return McpResponse::Error {
                error: "Missing required parameter: connection_id".to_string(),
            },
        };

        let message = match args.get("message").and_then(|v| v.as_str()) {
            Some(msg) => msg,
            None => return McpResponse::Error {
                error: "Missing required parameter: message".to_string(),
            },
        };

        let binary = args.get("binary").and_then(|v| v.as_bool()).unwrap_or(false);

        match self.websocket_manager.send_message(connection_id, message, binary).await {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "Message sent successfully"
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Failed to send WebSocket message: {}", e),
            },
        }
    }

    async fn websocket_simulate(&mut self, args: Value) -> McpResponse {
        let connection_id = match args.get("connection_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return McpResponse::Error {
                error: "Missing required parameter: connection_id".to_string(),
            },
        };

        let events = args.get("events")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_else(|| vec!["heartbeat", "user_joined", "message"]);

        match self.websocket_manager.simulate_realtime_events(connection_id, events).await {
            Ok(_) => {
                // Get message history to show what was simulated
                match self.websocket_manager.get_message_history(connection_id) {
                    Ok((sent, received)) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": serde_json::json!({
                                "events_simulated": received.len(),
                                "recent_messages": received.iter().take(5).collect::<Vec<_>>()
                            }).to_string()
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::Error {
                        error: format!("Events simulated but failed to get history: {}", e),
                    },
                }
            },
            Err(e) => McpResponse::Error {
                error: format!("Failed to simulate WebSocket events: {}", e),
            },
        }
    }

    // Browser status method
    async fn browser_status(&mut self, args: Value) -> McpResponse {
        let include_websockets = args.get("include_websockets").and_then(|v| v.as_bool()).unwrap_or(true);
        let include_storage = args.get("include_storage").and_then(|v| v.as_bool()).unwrap_or(true);

        let mut status = serde_json::json!({
            "browser": {
                "name": "Synaptic",
                "version": "0.1.0",
                "type": "headless",
                "capabilities": [
                    "javascript_execution",
                    "form_submission",
                    "cookie_management",
                    "local_storage",
                    "session_storage",
                    "websocket_simulation",
                    "stealth_features",
                    "anti_bot_evasion"
                ]
            }
        });

        if include_websockets {
            let active_connections = self.websocket_manager.get_active_connections();
            status["websockets"] = serde_json::json!({
                "active_connections": active_connections.len(),
                "connection_ids": active_connections
            });
        }

        if include_storage {
            match self.browser.get_storage_state() {
                Ok(storage_state) => {
                    // Get auth status
                    let has_bearer = self.browser.get_bearer_token().unwrap_or(None).is_some();
                    let has_csrf = self.browser.get_csrf_token().unwrap_or(None).is_some();
                    let custom_headers = self.browser.get_custom_headers().unwrap_or_default();
                    
                    status["storage"] = serde_json::json!({
                        "local_storage_keys": storage_state.local_storage.len(),
                        "session_storage_keys": storage_state.session_storage.len(),
                        "has_bearer_token": has_bearer,
                        "has_csrf_token": has_csrf,
                        "custom_headers": custom_headers.len()
                    });
                },
                Err(_) => {
                    status["storage"] = serde_json::json!({
                        "status": "unavailable"
                    });
                }
            }
        }

        McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&status).unwrap_or_default()
            })],
            is_error: false,
        }
    }
}
