use serde_json::{Value, json};

use crate::engine::browser::snapshot::SnapshotOptions;
use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::sanitize_session_id;

impl BrowserTools {
    /// `browser_snapshot`: compact outline of the current page with element refs.
    pub async fn handle_snapshot(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let defaults = SnapshotOptions::default();
        let options = SnapshotOptions {
            max_tokens: params
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .map_or(defaults.max_tokens, |t| t.clamp(200, 50_000) as usize),
            interactive_only: params
                .get("interactive_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        let focus_ref = params.get("focus_ref").and_then(|v| v.as_str());

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let Ok(mut guard) = browser.lock() else {
            return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        };
        match guard.snapshot(options, focus_ref) {
            Ok(snapshot) => {
                let url = guard.get_current_url();
                let header = format!(
                    "url: {}\nrefs: {}{}\n",
                    url.as_deref().unwrap_or("about:blank"),
                    snapshot.ref_count,
                    if snapshot.truncated {
                        " (truncated)"
                    } else {
                        ""
                    }
                );
                McpResponse::page_content(
                    url.as_deref(),
                    Value::String(format!("{header}{}", snapshot.text)),
                )
            }
            Err(e) => {
                let message = e.to_string();
                let code = if message.starts_with("stale_ref") {
                    -32602
                } else {
                    -1
                };
                McpResponse::error(code, message)
            }
        }
    }

    /// If `params` has a `ref` from `browser_snapshot`, resolve it to a CSS
    /// selector and store it as `params["selector"]` (a ref takes precedence
    /// over a selector). Stale or unknown refs are reported as errors.
    pub(crate) fn resolve_ref_param(&self, mut params: Value) -> Result<Value, McpResponse> {
        let Some(element_ref) = params
            .get("ref")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            return Ok(params);
        };
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let browser = self
            .get_session(&session_id)
            .map_err(|e| McpResponse::error(-32602, e))?;
        let selector = {
            let guard = browser.lock().map_err(|_| {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            })?;
            guard
                .resolve_ref(&element_ref)
                .map_err(|e| McpResponse::error(-32602, e.to_string()))?
        };
        params["selector"] = json!(selector);
        Ok(params)
    }
}
