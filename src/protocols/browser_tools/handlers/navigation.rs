use serde_json::{Value, json};
use url::Url;

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{
    MAX_URL_LENGTH, limit_input_length, sanitize_session_id, validate_url_for_navigation,
};

impl BrowserTools {
    pub async fn handle_navigate_to(&self, params: Value) -> McpResponse {
        let url = params["url"].as_str().unwrap_or("");
        let wait_for_load = params
            .get("wait_for_load")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let wait_for_js = params
            .get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if url.is_empty() {
            return McpResponse::error(-1, "URL is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(url, MAX_URL_LENGTH, "URL") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        // Validate URL format
        if Url::parse(url).is_err() {
            return McpResponse::error(-1, "Invalid URL format".to_string());
        }

        // SECURITY: Validate URL to prevent SSRF attacks (CWE-918)
        if let Err(e) = validate_url_for_navigation(url) {
            return McpResponse::error(-32602, format!("URL blocked for security: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        let url_owned = url.to_string();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.navigate_to_with_js_option(
                    &url_owned,
                    wait_for_load,
                    wait_for_js,
                )) {
                    Ok(content) => {
                        let current_url = guard.get_current_url();
                        McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": current_url,
                            "message": format!("Successfully navigated to {}", url_owned)
                        }))
                    }
                    Err(e) => McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }

    pub async fn handle_navigate_back(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // SECURITY: Validate session ID
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.go_back()) {
                    Ok(Some(content)) => {
                        let current_url = guard.get_current_url();
                        McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": current_url
                        }))
                    }
                    Ok(None) => McpResponse::success(json!({
                        "success": false,
                        "message": "Cannot go back further"
                    })),
                    Err(e) => McpResponse::error(-1, format!("Failed to navigate back: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }

    pub async fn handle_navigate_forward(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // SECURITY: Validate session ID
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.go_forward()) {
                    Ok(Some(content)) => {
                        let current_url = guard.get_current_url();
                        McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": current_url
                        }))
                    }
                    Ok(None) => McpResponse::success(json!({
                        "success": false,
                        "message": "Cannot go forward further"
                    })),
                    Err(e) => McpResponse::error(-1, format!("Failed to navigate forward: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }

    pub async fn handle_refresh_page(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // SECURITY: Validate session ID
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.reload()) {
                    Ok(content) => {
                        let current_url = guard.get_current_url();
                        McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": current_url,
                            "message": "Page refreshed successfully"
                        }))
                    }
                    Err(e) => McpResponse::error(-1, format!("Failed to refresh page: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }
}
