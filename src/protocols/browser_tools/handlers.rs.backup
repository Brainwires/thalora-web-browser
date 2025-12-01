use serde_json::{json, Value};
use std::collections::HashMap;
use url::Url;
use rand;

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
        let mut potential_new_window_info = None;

        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    // Check if this is a submit button for a form that opens new windows
                    if let Some(form_info) = browser_guard.find_form_by_submit_button(selector) {
                        if form_info.opens_new_window {
                            eprintln!("🔍 DEBUG: Click on submit button for new window form detected");
                            eprintln!("🔍 DEBUG: Form target: {}, action: {}", form_info.target, form_info.action);

                            // Create predictive session for the new window
                            if let Some(ref predicted_url) = form_info.predicted_url {
                                let predictive_session_id = format!("predictive_{}_{}",
                                    session_id,
                                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                                );

                                eprintln!("🔍 DEBUG: Creating predictive session: {} for URL: {}", predictive_session_id, predicted_url);

                                // Create the predictive session (persistent=false for temporary use)
                                let _predictive_browser = self.get_or_create_session(&predictive_session_id, false);

                                potential_new_window_info = Some(json!({
                                    "will_open_new_window": true,
                                    "predicted_url": predicted_url,
                                    "predictive_session_id": predictive_session_id,
                                    "form_target": form_info.target,
                                    "form_action": form_info.action,
                                    "form_method": form_info.method
                                }));
                            }
                        }
                    }

                    match browser_guard.click_element(selector).await {
                        Ok(mut resp) => {
                            // Add potential new window info to response
                            if let Some(new_window_info) = potential_new_window_info {
                                let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                                if let Some(obj) = resp_json.as_object_mut() {
                                    obj.insert("potential_new_window".to_string(), new_window_info);
                                }
                                response = McpResponse::success(resp_json);
                            } else {
                                response = McpResponse::success(serde_json::to_value(resp).unwrap_or_default());
                            }
                        }
                        Err(e) => response = McpResponse::error(-1, format!("Failed to click element: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_type_text(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let text = params["text"].as_str().unwrap_or("");
        let clear_first = params.get("clear_first").and_then(|v| v.as_bool()).unwrap_or(true);
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "Selector is required".to_string());
        }

        if text.is_empty() {
            return McpResponse::error(-1, "Text is required".to_string());
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    // Use the browser's text input functionality
                    match browser_guard.type_text_into_element(selector, text, clear_first).await {
                        Ok(resp) => response = McpResponse::success(serde_json::to_value(resp).unwrap_or_default()),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to type text: {}", e)),
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
        let mut potential_new_window_info = None;

        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    // Check if this form would open new windows when submitted via click
                    let matching_form = browser_guard.get_analyzed_forms().iter().find(|form| {
                        form.selector == form_selector || form.selector.contains(form_selector)
                    });

                    if let Some(form_info) = matching_form {
                        if form_info.opens_new_window {
                            potential_new_window_info = Some(json!({
                                "form_would_open_new_window": true,
                                "predicted_url": form_info.predicted_url,
                                "form_target": form_info.target,
                                "note": "Form has target='_blank' but programmatic submission bypasses this behavior"
                            }));
                        }
                    }

                    match browser_guard.submit_form(form_selector, form_map).await {
                        Ok(mut resp) => {
                            // Add potential new window info to response
                            let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                            if let Some(new_window_info) = potential_new_window_info {
                                if let Some(obj) = resp_json.as_object_mut() {
                                    obj.insert("potential_new_window".to_string(), new_window_info);
                                }
                            }
                            response = McpResponse::success(resp_json);
                        }
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

    pub async fn handle_session_management(&self, params: Value) -> McpResponse {
        let action = params["action"].as_str().unwrap_or("");

        match action {
            "create" => {
                let persistent = params.get("persistent").and_then(|v| v.as_bool()).unwrap_or(false);
                // Generate a unique session ID using timestamp and random component
                let session_id = format!("session_{}_{}",
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    rand::random::<u32>()
                );
                let _browser = self.get_or_create_session(&session_id, persistent);
                McpResponse::success(json!({
                    "session_id": session_id,
                    "created": true,
                    "persistent": persistent
                }))
            }
            "info" => {
                let session_id = params.get("session_id")
                    .and_then(|v| v.as_str());
                if let Some(session_id) = session_id {
                    if let Some(session) = self.get_session_info(session_id) {
                        McpResponse::success(serde_json::to_value(session).unwrap_or_default())
                    } else {
                        McpResponse::error(-1, "Session not found".to_string())
                    }
                } else {
                    McpResponse::error(-1, "Session ID is required for info action".to_string())
                }
            }
            "list" => {
                let sessions = self.list_sessions();
                McpResponse::success(json!({"sessions": sessions}))
            }
            "close" => {
                let session_id = params.get("session_id")
                    .and_then(|v| v.as_str());
                if let Some(session_id) = session_id {
                    let closed = self.close_session(session_id);
                    McpResponse::success(json!({
                        "session_id": session_id,
                        "closed": closed
                    }))
                } else {
                    McpResponse::error(-1, "Session ID is required for close action".to_string())
                }
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

    pub async fn handle_prepare_form_submission(&self, params: Value) -> McpResponse {
        let form_selector = params["form_selector"].as_str().unwrap_or("");
        let submit_button_selector = params.get("submit_button_selector")
            .and_then(|v| v.as_str());
        let session_id = params.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if form_selector.is_empty() {
            return McpResponse::error(-1, "Form selector is required".to_string());
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());

        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(browser_guard) => {
                    // Find forms that match the selector and open new windows
                    let new_window_forms: Vec<_> = browser_guard.get_new_window_forms().into_iter().cloned().collect();

                    let matching_form = new_window_forms.iter().find(|form| {
                        // Check if the form selector matches
                        form.selector == form_selector ||
                        form.selector.contains(form_selector) ||
                        // If submit button selector provided, check if it matches
                        submit_button_selector.map_or(false, |btn_sel| {
                            form.submit_buttons.iter().any(|btn| btn == btn_sel || btn.contains(btn_sel))
                        })
                    });

                    if let Some(form_info) = matching_form {
                        if let Some(ref predicted_url) = form_info.predicted_url {
                            // Create predictive session for the form submission
                            let predictive_session_id = format!("predictive_{}_{}",
                                session_id,
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                            );

                            eprintln!("🔍 DEBUG: Creating predictive session for form preparation: {}", predictive_session_id);

                            // Create the predictive session
                            let _predictive_browser = self.get_or_create_session(&predictive_session_id, false);

                            response = McpResponse::success(json!({
                                "success": true,
                                "message": format!("Predictive session created for form that opens new window"),
                                "form_info": {
                                    "selector": form_info.selector,
                                    "action": form_info.action,
                                    "target": form_info.target,
                                    "method": form_info.method,
                                    "predicted_url": predicted_url,
                                    "submit_buttons": form_info.submit_buttons
                                },
                                "predictive_session_id": predictive_session_id,
                                "ready_for_submission": true
                            }));
                        } else {
                            response = McpResponse::error(-1, "Form found but no predicted URL available".to_string());
                        }
                    } else {
                        // Check if any form matches the selector but doesn't open new windows
                        let all_forms = browser_guard.get_analyzed_forms();
                        let form_exists = all_forms.iter().any(|form| {
                            form.selector == form_selector || form.selector.contains(form_selector)
                        });

                        if form_exists {
                            response = McpResponse::success(json!({
                                "success": true,
                                "message": "Form found but does not open new windows",
                                "predictive_session_needed": false,
                                "form_opens_new_window": false
                            }));
                        } else {
                            response = McpResponse::error(-1, format!("No form found matching selector: {}", form_selector));
                        }
                    }
                }
                Err(_) => { }
            }
        }
        response
    }

    pub async fn handle_validate_session(&self, params: Value) -> McpResponse {
        let session_id = params["session_id"].as_str().unwrap_or("");
        let expected_url_pattern = params.get("expected_url_pattern")
            .and_then(|v| v.as_str());
        let expected_content = params.get("expected_content")
            .and_then(|v| v.as_str());
        let timeout = params.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        if session_id.is_empty() {
            return McpResponse::error(-1, "Session ID is required".to_string());
        }

        // Check if session exists
        if let Some(session_info) = self.get_session_info(session_id) {
            // Try to get browser content if session exists
            let sessions = self.sessions.lock().unwrap();
            if let Some((browser, _)) = sessions.get(session_id) {
                let mut validation_result = json!({
                    "session_exists": true,
                    "session_info": {
                        "session_id": session_info.session_id,
                        "created_at": session_info.created_timestamp,
                        "last_accessed": session_info.last_accessed_timestamp,
                        "persistent": session_info.persistent
                    }
                });

                if let Ok(browser_guard) = browser.try_lock() {
                    let current_url = browser_guard.get_current_url();
                    let current_content = browser_guard.get_current_content();

                    validation_result["current_url"] = json!(current_url);
                    validation_result["content_length"] = json!(current_content.len());
                    validation_result["has_content"] = json!(!current_content.is_empty());

                    // Check URL pattern if provided
                    if let Some(url_pattern) = expected_url_pattern {
                        if let Some(ref url) = current_url {
                            let url_matches = if let Ok(regex) = regex::Regex::new(url_pattern) {
                                regex.is_match(url)
                            } else {
                                url.contains(url_pattern)
                            };
                            validation_result["url_matches_pattern"] = json!(url_matches);
                            validation_result["expected_url_pattern"] = json!(url_pattern);
                        } else {
                            validation_result["url_matches_pattern"] = json!(false);
                            validation_result["error"] = json!("No current URL in session");
                        }
                    }

                    // Check expected content if provided
                    if let Some(content_check) = expected_content {
                        let content_matches = current_content.contains(content_check);
                        validation_result["content_matches"] = json!(content_matches);
                        validation_result["expected_content"] = json!(content_check);
                    }

                    validation_result["validation_successful"] = json!(true);
                } else {
                    validation_result["validation_successful"] = json!(false);
                    validation_result["error"] = json!("Could not acquire browser lock");
                }

                McpResponse::success(validation_result)
            } else {
                McpResponse::error(-1, "Session exists but browser instance not found".to_string())
            }
        } else {
            McpResponse::success(json!({
                "session_exists": false,
                "validation_successful": false,
                "message": format!("Session '{}' not found", session_id)
            }))
        }
    }

    pub async fn handle_wait_for_element(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let timeout = params.get("timeout").and_then(|v| v.as_u64()).unwrap_or(10000);
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
                    match browser_guard.wait_for_element(selector, timeout).await {
                        Ok(found) => response = McpResponse::success(json!({
                            "found": found,
                            "selector": selector,
                            "timeout_ms": timeout,
                            "message": if found {
                                format!("Element found: {}", selector)
                            } else {
                                format!("Element not found after {}ms: {}", timeout, selector)
                            }
                        })),
                        Err(e) => response = McpResponse::error(-1, format!("Failed to wait for element: {}", e)),
                    }
                }
                Err(_) => { }
            }
        }
        response
    }
}