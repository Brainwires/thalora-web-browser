use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::cdp::{CdpServer, CdpCommand, CdpMessage};

/// Network domain - Network monitoring, cookies, and request/response inspection
pub struct NetworkTools;

impl NetworkTools {
    pub fn new() -> Self {
        Self
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
}
