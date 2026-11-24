// Browser jobs run one at a time on the session's own BrowserThread
// (single-threaded), so holding the browser's MutexGuard across .await
// cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use futures::FutureExt;
use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_JS_CODE_LENGTH, limit_input_length, sanitize_session_id};

impl BrowserTools {
    /// Evaluate a JavaScript expression in a session's page
    /// (`cdp_runtime_evaluate` / `browser_eval`).
    ///
    /// The script runs in the page's global scope, on the session's browser
    /// thread, under the script timeout. The result is the completion value
    /// as JSON (`undefined` is `null`, a DOM node is its markup, a value
    /// JSON cannot express is its string form). A promise is not awaited
    /// (TODO(a3): the pinned page runtime has no asynchronous evaluation
    /// yet). An uncaught exception, a timeout and a static page (JavaScript
    /// disabled or not started) are errors.
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
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                let url_before = guard.get_current_url();
                match guard.evaluate_javascript(&expression).await {
                    Ok(value) => {
                        let url = guard.get_current_url();
                        let mut out = json!({ "result": value, "url": url });
                        if url != url_before {
                            // The script navigated; the result is from the
                            // page it ran in
                            out["navigated"] = json!(true);
                        }
                        // What a page computes is page content: data, not
                        // instructions
                        McpResponse::page_content(out["url"].as_str(), out.clone())
                    }
                    Err(e) => McpResponse::error(-1, format!("Evaluation failed: {}", e)),
                }
            }
            .boxed_local()
        })
        .await
    }
}
