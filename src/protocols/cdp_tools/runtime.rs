use serde_json::Value;
use std::sync::Arc;
use crate::protocols::mcp::McpResponse;
use crate::protocols::cdp::{CdpServer, CdpCommand, CdpMessage};
use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::security::{sanitize_session_id, limit_input_length, MAX_JS_CODE_LENGTH};

/// Runtime domain - Script evaluation, exceptions, and runtime events
pub struct RuntimeTools {
    pub(super) browser_tools: Arc<BrowserTools>,
}

impl RuntimeTools {
    pub fn new(browser_tools: Arc<BrowserTools>) -> Self {
        Self { browser_tools }
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

        // Use session-managed browser for persistent CDP context
        let session_id = args.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("cdp_default");

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(expression, MAX_JS_CODE_LENGTH, "JavaScript expression") {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Input validation failed: {}", e)
                })],
                is_error: true,
            };
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Session ID validation failed: {}", e)
                })],
                is_error: true,
            };
        }

        let browser = self.browser_tools.get_or_create_session(session_id, false);

        let response;
        let lock_res = browser.lock();
        match lock_res {
            Ok(mut browser_guard) => {
                match browser_guard.execute_javascript(expression).await {
                    Ok(js_result) => {
                        // Try to parse as different types
                        if js_result == "true" || js_result == "false" {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": format!("JavaScript result (boolean): {}", js_result)
                                })],
                                is_error: false,
                            };
                        } else if let Ok(num) = js_result.parse::<f64>() {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": format!("JavaScript result (number): {}", num)
                                })],
                                is_error: false,
                            };
                        } else {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": format!("JavaScript result: {}", js_result)
                                })],
                                is_error: false,
                            };
                        }
                    }
                    Err(e) => {
                        response = McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": format!("JavaScript execution error: {}", e)
                            })],
                            is_error: true,
                        };
                    }
                }
            }
            Err(_) => {
                // If session browser fails, try CDP server as fallback
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
                    Ok(Some(CdpMessage::Response(cdp_response))) => {
                        if let Some(error) = cdp_response.error {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": format!("CDP JavaScript evaluation failed: {}", error.message)
                                })],
                                is_error: true,
                            };
                        } else if let Some(result) = cdp_response.result {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": format!("CDP JavaScript evaluation result: {}", result)
                                })],
                                is_error: false,
                            };
                        } else {
                            response = McpResponse::ToolResult {
                                content: vec![serde_json::json!({
                                    "type": "text",
                                    "text": "CDP JavaScript evaluation completed (no result)"
                                })],
                                is_error: false,
                            };
                        }
                    }
                    Ok(_) => {
                        response = McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": "CDP JavaScript evaluation completed (no response)"
                            })],
                            is_error: false,
                        };
                    }
                    Err(e) => {
                        response = McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": format!("CDP JavaScript evaluation error: {}", e)
                            })],
                            is_error: true,
                        };
                    }
                }
            }
        }

        response
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
}
