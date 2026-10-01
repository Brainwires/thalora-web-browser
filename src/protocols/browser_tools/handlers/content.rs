use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;

impl BrowserTools {
    pub async fn handle_get_page_content(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        match crate::protocols::browser_tools::core::page_state(&browser).await {
            Ok((url, content)) => McpResponse::page_content(
                url.as_deref(),
                json!({
                    "content": content,
                    "url": url,
                    "session_id": session_id
                }),
            ),
            Err(e) => McpResponse::error(-1, e),
        }
    }
}
