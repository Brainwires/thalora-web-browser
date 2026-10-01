//! `browser_fill_credential`: fill a stored credential into a page without
//! the secret ever entering the model's context.

// Browser jobs run one at a time on the session's own BrowserThread
// (single-threaded), so holding the browser's MutexGuard across .await
// cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use futures::FutureExt;
use serde_json::{Value, json};

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use crate::protocols::memory_tools::credentials::normalize_origin;
use crate::protocols::security::sanitize_session_id;

impl McpServer {
    pub(crate) async fn handle_fill_credential(&mut self, arguments: Value) -> McpResponse {
        let Some(service) = arguments
            .get("service")
            .or_else(|| arguments.get("key"))
            .and_then(|v| v.as_str())
        else {
            return McpResponse::error(-32602, "Missing required parameter: service".to_string());
        };
        let session_id = arguments
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let (username, password, additional_data) = match self.ai_memory.get_credentials(service) {
            Ok(Some((_service, username, password, additional_data))) => {
                (username, password, additional_data)
            }
            Ok(None) => {
                return McpResponse::error(
                    -32602,
                    format!("No stored credential named '{}'", service),
                );
            }
            Err(e) => {
                return McpResponse::error(-1, format!("Failed to read credential: {}", e));
            }
        };
        let Some(credential_origin) = additional_data.get("origin").cloned() else {
            return McpResponse::error(
                -32602,
                format!(
                    "Credential '{}' has no origin, so it can't be filled safely. Store it again \
                     with ai_memory_store_credentials and an `origin` (e.g. https://example.com).",
                    service
                ),
            );
        };

        let browser = match self.browser_tools.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let service = service.to_string();

        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
async move {
            let Ok(mut guard) = browser.lock() else {
                return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
            };

            // SECURITY: only fill into pages of the credential's own origin
            let page_origin = guard.get_current_url().and_then(|u| normalize_origin(&u));
            if page_origin.as_deref() != Some(credential_origin.as_str()) {
                return McpResponse::error(
                    -32602,
                    format!(
                        "Refusing to fill: the page origin {} does not match the credential's \
                         origin {}",
                        page_origin.as_deref().unwrap_or("(none)"),
                        credential_origin
                    ),
                );
            }

            let target =
                |ref_key: &str, selector_key: &str| -> Result<Option<String>, McpResponse> {
                    if let Some(r) = arguments.get(ref_key).and_then(|v| v.as_str()) {
                        return guard
                            .resolve_ref(r)
                            .map(Some)
                            .map_err(|e| McpResponse::error(-32602, e.to_string()));
                    }
                    Ok(arguments
                        .get(selector_key)
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(str::to_string))
                };
            let password_field = match target("password_ref", "password_selector") {
                Ok(Some(selector)) => selector,
                Ok(None) => {
                    return McpResponse::error(
                        -32602,
                        "Give password_ref or password_selector".to_string(),
                    );
                }
                Err(resp) => return resp,
            };
            let username_field = match target("username_ref", "username_selector") {
                Ok(field) => field,
                Err(resp) => return resp,
            };

            let mut filled = Vec::new();
            if let Some(field) = &username_field {
                if let Err(e) = guard.fill_secret(field, &username).await {
                    return McpResponse::error(-1, format!("Failed to fill username: {}", e));
                }
                filled.push("username");
            }
            if let Err(e) = guard.fill_secret(&password_field, &password).await {
                return McpResponse::error(-1, format!("Failed to fill password: {}", e));
            }
            filled.push("password");

            McpResponse::success(json!({
                "success": true,
                "filled": filled,
                "service": service,
                "origin": credential_origin,
                "message": "Credential filled; the secret was not returned. Submit the form to log in."
            }))
        }
.boxed_local()
})
.await
    }
}
