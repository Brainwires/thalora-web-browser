use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::cdp::{CdpServer, CdpCommand, CdpMessage};
use crate::engine::browser::HeadlessWebBrowser;
use std::sync::{Arc, Mutex};

pub struct CdpTools {
    browser: Option<Arc<Mutex<HeadlessWebBrowser>>>,
}

impl CdpTools {
    pub fn new() -> Self {
        Self {
            browser: None,
        }
    }

    pub fn with_browser(browser: Arc<Mutex<HeadlessWebBrowser>>) -> Self {
        Self {
            browser: Some(browser),
        }
    }

    pub async fn enable_runtime(&mut self, _args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        // Use the actual CDP server to enable the runtime domain
        let command = CdpCommand {
            id: 1,
            method: "Runtime.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Runtime domain enable failed: {:?}", response.error)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP Runtime domain enabled successfully"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP Runtime domain enabled (no response)"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Runtime domain enable error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn evaluate_javascript(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let expression = match args.get("expression").and_then(|v| v.as_str()) {
            Some(expr) => expr,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: expression"
                    })],
                    is_error: true,
                };
            }
        };

        // Browser-based JavaScript execution is not implemented via the HeadlessWebBrowser
        // at this time. Fall back to using the CDP server evaluation below.

        // Fallback to CDP server evaluation
        let command = CdpCommand {
            id: 2,
            method: "Runtime.evaluate".to_string(),
            params: Some(serde_json::json!({
                "expression": expression,
                "returnByValue": true
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP JavaScript evaluation failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP JavaScript evaluation result: {}", result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP JavaScript evaluation completed (no result)"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP JavaScript evaluation completed (no response)"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP JavaScript evaluation error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn enable_debugger(&mut self, _args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let command = CdpCommand {
            id: 3,
            method: "Debugger.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Debugger domain enable failed: {:?}", response.error)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP Debugger domain enabled successfully"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP Debugger domain enabled"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Debugger domain enable error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn set_breakpoint(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let line_number = match args.get("line_number").and_then(|v| v.as_i64()) {
            Some(line) => line,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: line_number"
                    })],
                    is_error: true,
                };
            }
        };

        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");

        let command = CdpCommand {
            id: 4,
            method: "Debugger.setBreakpointByUrl".to_string(),
            params: Some(serde_json::json!({
                "lineNumber": line_number,
                "url": url
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Breakpoint set failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Breakpoint set successfully at line {} in {}: {}", line_number, url, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Breakpoint set at line {} in {}", line_number, url)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Breakpoint set at line {} in {}", line_number, url)
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Breakpoint set error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn enable_dom(&mut self, _args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let command = CdpCommand {
            id: 5,
            method: "DOM.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM domain enable failed: {:?}", response.error)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP DOM domain enabled successfully"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM domain enabled"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP DOM domain enable error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn get_document(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let depth = args.get("depth").and_then(|v| v.as_i64()).unwrap_or(1);

        let command = CdpCommand {
            id: 6,
            method: "DOM.getDocument".to_string(),
            params: Some(serde_json::json!({
                "depth": depth
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM document retrieval failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM document retrieved: {}", result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP DOM document retrieved"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM document retrieved"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP DOM document retrieval error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn enable_network(&mut self, _args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let command = CdpCommand {
            id: 7,
            method: "Network.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Network domain enable failed: {:?}", response.error)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP Network domain enabled successfully"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP Network domain enabled"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Network domain enable error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn get_response_body(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let request_id = match args.get("request_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: request_id"
                    })],
                    is_error: true,
                };
            }
        };

        let command = CdpCommand {
            id: 8,
            method: "Network.getResponseBody".to_string(),
            params: Some(serde_json::json!({
                "requestId": request_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Response body retrieval failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Response body for request {}: {}", request_id, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP Response body for request {} retrieved", request_id)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Response body for request {} retrieved", request_id)
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Response body retrieval error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }
}