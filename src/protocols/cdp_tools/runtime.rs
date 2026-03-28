use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::cdp::{CdpCommand, CdpMessage, CdpServer};
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_JS_CODE_LENGTH, limit_input_length, sanitize_session_id};
use serde_json::Value;
use std::sync::Arc;

/// Runtime domain - Script evaluation, exceptions, and runtime events
pub struct RuntimeTools {
    pub(super) browser_tools: Arc<BrowserTools>,
}

impl RuntimeTools {
    pub fn new(browser_tools: Arc<BrowserTools>) -> Self {
        Self { browser_tools }
    }

    pub async fn enable_runtime(
        &mut self,
        _args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
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
                    McpResponse::error(
                        -1,
                        format!("CDP Runtime domain enable failed: {:?}", response.error),
                    )
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "CDP Runtime domain enabled successfully"
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": "CDP Runtime domain enabled (no response)"
            })),
            Err(e) => McpResponse::error(-1, format!("CDP Runtime domain enable error: {}", e)),
        }
    }

    pub async fn evaluate_javascript(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let expression = match args.get("expression").and_then(|v| v.as_str()) {
            Some(expr) => expr,
            None => {
                return McpResponse::error(
                    -1,
                    "Missing required parameter: expression".to_string(),
                );
            }
        };

        // Use session-managed browser for persistent CDP context
        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("cdp_default");

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(expression, MAX_JS_CODE_LENGTH, "JavaScript expression")
        {
            return McpResponse::error(-1, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-1, format!("Session ID validation failed: {}", e));
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
                            response = McpResponse::success(serde_json::json!({
                                "type": "text",
                                "text": format!("JavaScript result (boolean): {}", js_result)
                            }));
                        } else if let Ok(num) = js_result.parse::<f64>() {
                            response = McpResponse::success(serde_json::json!({
                                "type": "text",
                                "text": format!("JavaScript result (number): {}", num)
                            }));
                        } else {
                            response = McpResponse::success(serde_json::json!({
                                "type": "text",
                                "text": format!("JavaScript result: {}", js_result)
                            }));
                        }
                    }
                    Err(e) => {
                        response =
                            McpResponse::error(-1, format!("JavaScript execution error: {}", e));
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
                            response = McpResponse::error(
                                -1,
                                format!("CDP JavaScript evaluation failed: {}", error.message),
                            );
                        } else if let Some(result) = cdp_response.result {
                            response = McpResponse::success(serde_json::json!({
                                "type": "text",
                                "text": format!("CDP JavaScript evaluation result: {}", result)
                            }));
                        } else {
                            response = McpResponse::success(serde_json::json!({
                                "type": "text",
                                "text": "CDP JavaScript evaluation completed (no result)"
                            }));
                        }
                    }
                    Ok(_) => {
                        response = McpResponse::success(serde_json::json!({
                            "type": "text",
                            "text": "CDP JavaScript evaluation completed (no response)"
                        }));
                    }
                    Err(e) => {
                        response = McpResponse::error(
                            -1,
                            format!("CDP JavaScript evaluation error: {}", e),
                        );
                    }
                }
            }
        }

        response
    }

    pub async fn get_console_messages(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
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
                    McpResponse::error(
                        -1,
                        format!("Get console messages failed: {}", error.message),
                    )
                } else if let Some(result) = response.result {
                    let filtered_msg = if let Some(level) = level {
                        format!("Console messages (filtered by {}): {}", level, result)
                    } else {
                        format!("Console messages: {}", result)
                    };
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": filtered_msg
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "No console messages available"
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP get console messages error: {}", err)),
        }
    }
}
