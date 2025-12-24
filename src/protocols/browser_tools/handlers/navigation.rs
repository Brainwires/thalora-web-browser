use serde_json::{json, Value};
use url::Url;

use crate::protocols::mcp::McpResponse;
use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::security::validate_url_for_navigation;

impl BrowserTools {
    pub async fn handle_scrape_url(&self, params: Value) -> McpResponse {
        let url = params["url"].as_str().unwrap_or("");
        let wait_for_js = params["wait_for_js"].as_bool().unwrap_or(false);
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if url.is_empty() {
            return McpResponse::error(-1, "URL is required".to_string());
        }

        // Validate URL format
        if let Err(_) = Url::parse(url) {
            return McpResponse::error(-1, "Invalid URL format".to_string());
        }

        // SECURITY: Validate URL to prevent SSRF attacks (CWE-918)
        if let Err(e) = validate_url_for_navigation(url) {
            return McpResponse::error(-32602, format!("URL blocked for security: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.navigate_to_with_options(url, wait_for_js).await {
                        Ok(_) => {
                            match browser_guard.scrape_current_page().await {
                                Ok(scraped_data) => {
                                    response = McpResponse::success(serde_json::to_value(scraped_data).unwrap_or_default());
                                }
                                Err(e) => response = McpResponse::error(-1, format!("Failed to scrape page: {}", e)),
                            }
                        }
                        Err(e) => response = McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
                    }
                }
                Err(_) => { /* keep default response */ }
            }
        }
        response
    }

    pub async fn handle_navigate_to(&self, params: Value) -> McpResponse {
        let url = params["url"].as_str().unwrap_or("");
        let wait_for_load = params.get("wait_for_load").and_then(|v| v.as_bool()).unwrap_or(true);
        let wait_for_js = params.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(false);
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if url.is_empty() {
            return McpResponse::error(-1, "URL is required".to_string());
        }

        // Validate URL format
        if let Err(_) = Url::parse(url) {
            return McpResponse::error(-1, "Invalid URL format".to_string());
        }

        // SECURITY: Validate URL to prevent SSRF attacks (CWE-918)
        if let Err(e) = validate_url_for_navigation(url) {
            return McpResponse::error(-32602, format!("URL blocked for security: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.navigate_to_with_js_option(url, wait_for_load, wait_for_js).await {
                        Ok(content) => response = McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": browser_guard.get_current_url(),
                            "message": format!("Successfully navigated to {}", url)
                        })),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_navigate_back(&self, params: Value) -> McpResponse {
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.go_back().await {
                        Ok(Some(content)) => response = McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": browser_guard.get_current_url()
                        })),
                        Ok(None) => response = McpResponse::success(json!({
                            "success": false,
                            "message": "Cannot go back further"
                        })),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to navigate back: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_navigate_forward(&self, params: Value) -> McpResponse {
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.go_forward().await {
                        Ok(Some(content)) => response = McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": browser_guard.get_current_url()
                        })),
                        Ok(None) => response = McpResponse::success(json!({
                            "success": false,
                            "message": "Cannot go forward further"
                        })),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to navigate forward: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_refresh_page(&self, params: Value) -> McpResponse {
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.reload().await {
                        Ok(content) => response = McpResponse::success(json!({
                            "success": true,
                            "content": content,
                            "url": browser_guard.get_current_url(),
                            "message": "Page refreshed successfully"
                        })),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to refresh page: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }
}
