use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

use crate::engine::browser::HeadlessWebBrowser;
use crate::protocols::mcp::McpResponse;
use crate::protocols::browser_tools::core::BrowserTools;

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

        // Validate URL
        if let Err(_) = Url::parse(url) {
            return McpResponse::error(-1, "Invalid URL format".to_string());
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.navigate_to(url).await {
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

    pub async fn handle_click_element(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "Selector is required".to_string());
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.click_link(selector).await {
                        Ok(resp) => response = McpResponse::success(serde_json::to_value(resp).unwrap_or_default()),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to click element: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_fill_form(&self, params: Value) -> McpResponse {
        let form_data = params["form_data"].as_object();
        let form_selector = params.get("form_selector")
            .and_then(|v| v.as_str())
            .unwrap_or("form");
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if form_data.is_none() {
            return McpResponse::error(-1, "Form data is required".to_string());
        }

        let form_data = form_data.unwrap();
        let mut form_map = HashMap::new();

        for (key, value) in form_data {
            if let Some(string_value) = value.as_str() {
                form_map.insert(key.clone(), string_value.to_string());
            }
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.submit_form(form_selector, form_map).await {
                        Ok(resp) => response = McpResponse::success(serde_json::to_value(resp).unwrap_or_default()),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to submit form: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_get_page_content(&self, params: Value) -> McpResponse {
        let session_id = params.get("session_id")
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

    pub async fn handle_session_management(&self, params: Value) -> McpResponse {
        let action = params["action"].as_str().unwrap_or("");
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        match action {
            "create" => {
                let persistent = params.get("persistent").and_then(|v| v.as_bool()).unwrap_or(false);
                let _browser = self.get_or_create_session(session_id, persistent);
                McpResponse::success(json!({
                    "session_id": session_id,
                    "created": true,
                    "persistent": persistent
                }))
            }
            "info" => {
                if let Some(session) = self.get_session_info(session_id) {
                    McpResponse::success(serde_json::to_value(session).unwrap_or_default())
                } else {
                    McpResponse::error(-1, "Session not found".to_string())
                }
            }
            "list" => {
                let sessions = self.list_sessions();
                McpResponse::success(json!({"sessions": sessions}))
            }
            "close" => {
                let closed = self.close_session(session_id);
                McpResponse::success(json!({
                    "session_id": session_id,
                    "closed": closed
                }))
            }
            "cleanup" => {
                let max_age = params.get("max_age_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600); // Default 1 hour
                self.cleanup_expired_sessions(max_age);
                McpResponse::success(json!({"cleaned_up": true}))
            }
            _ => McpResponse::error(-1, format!("Unknown action: {}", action))
        }
    }
}