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

        // Text and HTML are computed from the live document on the browser
        // thread; only the strings come back.
        let page = crate::protocols::browser_tools::core::with_page(&browser, move |page| {
            let text = include_text
                .then(|| page.with_doc(crate::engine::browser::page_text::visible_text));
            let html = include_html.then(|| page.to_html());
            (page.url().map(str::to_string), text, html)
        })
        .await;
        match page {
            Ok((url, text, html)) => {
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
                if let Some(text) = text {
                    body["text"] = json!(limit(text));
                }
                if let Some(html) = html {
                    body["html"] = json!(limit(html));
                }
                body["truncated"] = json!(truncated);
                McpResponse::page_content(url.as_deref(), body)
            }
            Err(e) => McpResponse::error(-1, e),
        }
    }
}
