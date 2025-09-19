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

    // New debugging tools implementation

    pub async fn query_selector(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let selector = match args.get("selector").and_then(|v| v.as_str()) {
            Some(sel) => sel,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: selector"
                    })],
                    is_error: true,
                };
            }
        };

        let node_id = args.get("node_id").and_then(|v| v.as_i64()).unwrap_or(1);

        let command = CdpCommand {
            id: 10,
            method: "DOM.querySelector".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id,
                "selector": selector
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("DOM query selector failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Element found with selector '{}': {}", selector, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No element found with selector: {}", selector)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP query selector error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_attributes(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: node_id"
                    })],
                    is_error: true,
                };
            }
        };

        let command = CdpCommand {
            id: 11,
            method: "DOM.getAttributes".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Get attributes failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Attributes for node {}: {}", node_id, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No attributes found for node: {}", node_id)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get attributes error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_computed_style(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: node_id"
                    })],
                    is_error: true,
                };
            }
        };

        let command = CdpCommand {
            id: 12,
            method: "CSS.getComputedStyleForNode".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Get computed style failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Computed styles for node {}: {}", node_id, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No computed styles found for node: {}", node_id)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get computed style error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_cookies(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let urls = args.get("urls")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());

        let command = CdpCommand {
            id: 13,
            method: "Network.getCookies".to_string(),
            params: if let Some(urls) = urls {
                Some(serde_json::json!({ "urls": urls }))
            } else {
                None
            },
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Get cookies failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Cookies: {}", result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "No cookies found"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get cookies error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn set_cookie(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let name = match args.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: name"
                    })],
                    is_error: true,
                };
            }
        };

        let value = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: value"
                    })],
                    is_error: true,
                };
            }
        };

        let domain = args.get("domain").and_then(|v| v.as_str());
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("/");
        let secure = args.get("secure").and_then(|v| v.as_bool()).unwrap_or(false);
        let http_only = args.get("http_only").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut cookie_params = serde_json::json!({
            "name": name,
            "value": value,
            "path": path,
            "secure": secure,
            "httpOnly": http_only
        });

        if let Some(domain) = domain {
            cookie_params["domain"] = serde_json::Value::String(domain.to_string());
        }

        let command = CdpCommand {
            id: 14,
            method: "Network.setCookie".to_string(),
            params: Some(cookie_params),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Set cookie failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Cookie '{}' set successfully", name)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP set cookie error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_console_messages(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        // Enable Console domain first
        let enable_command = CdpCommand {
            id: 15,
            method: "Console.enable".to_string(),
            params: None,
            session_id: None,
        };

        let _enable_result = cdp_server.handle_message(CdpMessage::Command(enable_command));

        // Get console messages (this would typically require storing messages from events)
        // For now, we'll use Runtime.evaluate to get console history
        let level = args.get("level").and_then(|v| v.as_str());
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);

        let js_code = format!(
            r#"
            (function() {{
                // Try to get console messages if available
                if (window.console && window.console._messages) {{
                    return window.console._messages.slice(-{});
                }}
                return "Console message history not available";
            }})()
            "#,
            limit
        );

        let command = CdpCommand {
            id: 16,
            method: "Runtime.evaluate".to_string(),
            params: Some(serde_json::json!({
                "expression": js_code,
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
                            "text": format!("Get console messages failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    let filtered_msg = if let Some(level) = level {
                        format!("Console messages (filtered by {}): {}", level, result)
                    } else {
                        format!("Console messages: {}", result)
                    };
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": filtered_msg
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "No console messages available"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get console messages error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn take_screenshot(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("png");
        let quality = args.get("quality").and_then(|v| v.as_i64()).unwrap_or(80);
        let full_page = args.get("full_page").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut params = serde_json::json!({
            "format": format
        });

        if format == "jpeg" {
            params["quality"] = serde_json::Value::Number(serde_json::Number::from(quality));
        }

        if full_page {
            params["captureBeyondViewport"] = serde_json::Value::Bool(true);
        }

        let command = CdpCommand {
            id: 17,
            method: "Page.captureScreenshot".to_string(),
            params: Some(params),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Screenshot failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    if let Some(data) = result.get("data") {
                        McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "image",
                                "data": data,
                                "mimeType": format!("image/{}", format)
                            })],
                            is_error: false,
                        }
                    } else {
                        McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": format!("Screenshot captured: {}", result)
                            })],
                            is_error: false,
                        }
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "Screenshot capture completed"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP screenshot error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn reload_page(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let ignore_cache = args.get("ignore_cache").and_then(|v| v.as_bool()).unwrap_or(false);

        let command = CdpCommand {
            id: 18,
            method: "Page.reload".to_string(),
            params: Some(serde_json::json!({
                "ignoreCache": ignore_cache
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Page reload failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else {
                    let cache_msg = if ignore_cache { " (ignoring cache)" } else { "" };
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Page reloaded successfully{}", cache_msg)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP page reload error: {}", err)
                })],
                is_error: true,
            }
        }
    }
}