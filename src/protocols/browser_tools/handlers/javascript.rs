// Browser jobs run one at a time on the session's own BrowserThread
// (single-threaded), so holding the browser's MutexGuard across .await
// cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use futures::FutureExt;
use serde_json::Value;

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_JS_CODE_LENGTH, limit_input_length, sanitize_session_id};

impl BrowserTools {
    /// Evaluate a JavaScript expression in a session's page
    /// (`cdp_runtime_evaluate` / `browser_eval`).
    pub async fn handle_evaluate_javascript(&self, args: Value) -> McpResponse {
        let Some(expression) = args.get("expression").and_then(|v| v.as_str()) else {
            return McpResponse::error(-1, "Missing required parameter: expression".to_string());
        };
        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(expression, MAX_JS_CODE_LENGTH, "JavaScript expression")
        {
            return McpResponse::error(-1, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-1, format!("Session ID validation failed: {}", e));
        }

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };

        let expression = expression.to_string();
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut browser_guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                match browser_guard.execute_javascript(&expression).await {
                    Ok(js_result) => {
                        let text = if js_result == "true" || js_result == "false" {
                            format!("JavaScript result (boolean): {}", js_result)
                        } else if let Ok(num) = js_result.parse::<f64>() {
                            format!("JavaScript result (number): {}", num)
                        } else {
                            format!("JavaScript result: {}", js_result)
                        };
                        // Values come from page state, so they're untrusted
                        let page_url = browser_guard.get_current_url();
                        McpResponse::page_content(
                            page_url.as_deref(),
                            serde_json::Value::String(text),
                        )
                    }
                    Err(e) => McpResponse::error(-1, format!("JavaScript execution error: {}", e)),
                }
            }
            .boxed_local()
        })
        .await
    }
}
