use serde_json::{Value, json};
use std::collections::HashMap;

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{
    MAX_FORM_VALUE_LENGTH, MAX_SELECTOR_LENGTH, MAX_TEXT_INPUT_LENGTH, limit_input_length,
    sanitize_session_id,
};

impl BrowserTools {
    pub async fn handle_click_element(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "Selector is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(selector, MAX_SELECTOR_LENGTH, "CSS selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
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
                            eprintln!(
                                "🔍 DEBUG: Click on submit button for new window form detected"
                            );
                            eprintln!(
                                "🔍 DEBUG: Form target: {}, action: {}",
                                form_info.target, form_info.action
                            );

                            // Create predictive session for the new window
                            if let Some(ref predicted_url) = form_info.predicted_url {
                                let predictive_session_id = format!(
                                    "predictive_{}_{}",
                                    session_id,
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                );

                                eprintln!(
                                    "🔍 DEBUG: Creating predictive session: {} for URL: {}",
                                    predictive_session_id, predicted_url
                                );

                                // Create the predictive session (persistent=false for temporary use)
                                let _predictive_browser =
                                    self.get_or_create_session(&predictive_session_id, false);

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
                        Ok(resp) => {
                            // Add potential new window info to response
                            if let Some(new_window_info) = potential_new_window_info {
                                let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                                if let Some(obj) = resp_json.as_object_mut() {
                                    obj.insert("potential_new_window".to_string(), new_window_info);
                                }
                                response = McpResponse::success(resp_json);
                            } else {
                                response = McpResponse::success(
                                    serde_json::to_value(resp).unwrap_or_default(),
                                );
                            }
                        }
                        Err(e) => {
                            response =
                                McpResponse::error(-1, format!("Failed to click element: {}", e))
                        }
                    }
                }
                Err(_) => {}
            }
        }
        response
    }

    pub async fn handle_type_text(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let text = params["text"].as_str().unwrap_or("");
        let clear_first = params
            .get("clear_first")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "Selector is required".to_string());
        }

        if text.is_empty() {
            return McpResponse::error(-1, "Text is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(selector, MAX_SELECTOR_LENGTH, "CSS selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = limit_input_length(text, MAX_TEXT_INPUT_LENGTH, "Text input") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());
        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    // Use the browser's text input functionality
                    match browser_guard
                        .type_text_into_element(selector, text, clear_first)
                        .await
                    {
                        Ok(resp) => {
                            response =
                                McpResponse::success(serde_json::to_value(resp).unwrap_or_default())
                        }
                        Err(e) => {
                            response = McpResponse::error(-1, format!("Failed to type text: {}", e))
                        }
                    }
                }
                Err(_) => {}
            }
        }
        response
    }

    pub async fn handle_fill_form(&self, params: Value) -> McpResponse {
        let form_data = params["form_data"].as_object();
        let form_selector = params
            .get("form_selector")
            .and_then(|v| v.as_str())
            .unwrap_or("form");
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if form_data.is_none() {
            return McpResponse::error(-1, "Form data is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(form_selector, MAX_SELECTOR_LENGTH, "Form selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let form_data = form_data.unwrap();
        let mut form_map = HashMap::new();

        for (key, value) in form_data {
            if let Some(string_value) = value.as_str() {
                // SECURITY: Validate form field values
                if let Err(e) =
                    limit_input_length(string_value, MAX_FORM_VALUE_LENGTH, "Form field value")
                {
                    return McpResponse::error(
                        -32602,
                        format!("Input validation failed for field '{}': {}", key, e),
                    );
                }
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
                        Ok(resp) => {
                            // Add potential new window info to response
                            let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                            if let Some(new_window_info) = potential_new_window_info {
                                if let Some(obj) = resp_json.as_object_mut() {
                                    obj.insert("potential_new_window".to_string(), new_window_info);
                                }
                            }
                            response = McpResponse::success(resp_json);
                        }
                        Err(e) => {
                            response =
                                McpResponse::error(-1, format!("Failed to submit form: {}", e))
                        }
                    }
                }
                Err(_) => {}
            }
        }
        response
    }

    pub async fn handle_wait_for_element(&self, params: Value) -> McpResponse {
        let selector = params["selector"].as_str().unwrap_or("");
        let timeout = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(10000);
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "Selector is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(selector, MAX_SELECTOR_LENGTH, "CSS selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = self.get_or_create_session(session_id, false);
        let mut response = McpResponse::error(-1, "Failed to acquire browser lock".to_string());

        {
            let lock_res = browser.lock();
            match lock_res {
                Ok(mut browser_guard) => {
                    match browser_guard.wait_for_element(selector, timeout).await {
                        Ok(found) => {
                            response = McpResponse::success(json!({
                                "found": found,
                                "selector": selector,
                                "timeout_ms": timeout,
                                "message": if found {
                                    format!("Element found: {}", selector)
                                } else {
                                    format!("Element not found after {}ms: {}", timeout, selector)
                                }
                            }))
                        }
                        Err(e) => {
                            response =
                                McpResponse::error(-1, format!("Failed to wait for element: {}", e))
                        }
                    }
                }
                Err(_) => {}
            }
        }
        response
    }
}
