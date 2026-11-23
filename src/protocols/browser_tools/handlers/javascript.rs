use serde_json::Value;

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_JS_CODE_LENGTH, limit_input_length, sanitize_session_id};

impl BrowserTools {
    /// Evaluate a JavaScript expression in a session's page
    /// (`cdp_runtime_evaluate` / `browser_eval`).
    ///
    /// There is no JavaScript engine at the moment (Boa was removed, V8 is
    /// not wired in yet): after validating its arguments this always returns
    /// the [`JS_UNAVAILABLE`](crate::engine::JS_UNAVAILABLE) error.
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

        McpResponse::error(-1, crate::engine::JS_UNAVAILABLE.to_string())
    }
}
