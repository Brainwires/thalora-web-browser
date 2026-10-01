use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;

/// Default cap on returned text/HTML, in bytes (about 5k tokens).
const DEFAULT_MAX_LENGTH: usize = 20_000;

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
        let include_text = params
            .get("include_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let include_html = params
            .get("include_html")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_length = params
            .get("max_length")
            .and_then(|v| v.as_u64())
            .map_or(DEFAULT_MAX_LENGTH, |n| n.clamp(500, 2_000_000) as usize);

        match crate::protocols::browser_tools::core::page_state(&browser).await {
            Ok((url, html)) => {
                let mut truncated = false;
                let mut limit = |text: String| {
                    if text.len() > max_length {
                        truncated = true;
                        crate::protocols::mcp_server::core::McpServer::truncate_at_boundary(
                            &text, max_length,
                        )
                    } else {
                        text
                    }
                };
                let mut body = json!({"url": url, "session_id": session_id});
                if include_text {
                    body["text"] = json!(limit(crate::engine::browser::page_text::visible_text(
                        &html
                    )));
                }
                if include_html {
                    body["html"] = json!(limit(html));
                }
                body["truncated"] = json!(truncated);
                McpResponse::page_content(url.as_deref(), body)
            }
            Err(e) => McpResponse::error(-1, e),
        }
    }
}
