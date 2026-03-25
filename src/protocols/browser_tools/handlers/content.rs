use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;

impl BrowserTools {
    pub async fn handle_get_page_content(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(browser_guard) => {
                    let content = browser_guard.get_current_content();
                    let url = browser_guard.get_current_url();
                    response = McpResponse::success(json!({
                        "content": content,
                        "url": url,
                        "session_id": session_id
                    }));
                }
                Err(_) => {}
            }
        }
        response
    }
}
