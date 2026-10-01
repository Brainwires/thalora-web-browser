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

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let selector_owned = selector.to_string();
        let self_ref = self;
        let session_id_owned = session_id.to_string();

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                let mut potential_new_window_info = None;

                // Check if this is a submit button for a form that opens new windows
                if let Some(form_info) = guard.find_form_by_submit_button(&selector_owned)
                    && form_info.opens_new_window
                {
                    eprintln!("🔍 DEBUG: Click on submit button for new window form detected");
                    eprintln!(
                        "🔍 DEBUG: Form target: {}, action: {}",
                        form_info.target, form_info.action
                    );

                    if let Some(ref predicted_url) = form_info.predicted_url {
                        let predictive_session_id = format!(
                            "predictive_{}_{}",
                            session_id_owned,
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        );

                        eprintln!(
                            "🔍 DEBUG: Creating predictive session: {} for URL: {}",
                            predictive_session_id, predicted_url
                        );

                        let _predictive_browser =
                            self_ref.get_or_create_session(&predictive_session_id, false);

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

                match rt.block_on(guard.click_element(&selector_owned)) {
                    Ok(resp) => {
                        if let Some(new_window_info) = potential_new_window_info {
                            let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                            if let Some(obj) = resp_json.as_object_mut() {
                                obj.insert("potential_new_window".to_string(), new_window_info);
                            }
                            McpResponse::success(resp_json)
                        } else {
                            McpResponse::success(serde_json::to_value(resp).unwrap_or_default())
                        }
                    }
                    Err(e) => McpResponse::error(-1, format!("Failed to click element: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
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

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let selector_owned = selector.to_string();
        let text_owned = text.to_string();

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.type_text_into_element(
                    &selector_owned,
                    &text_owned,
                    clear_first,
                )) {
                    Ok(resp) => {
                        McpResponse::success(serde_json::to_value(resp).unwrap_or_default())
                    }
                    Err(e) => McpResponse::error(-1, format!("Failed to type text: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
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

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let form_selector_owned = form_selector.to_string();
        let submit = params
            .get("submit")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !submit {
            // Fill only: type each value into `<form_selector> [name="…"]`.
            return tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                let mut fields = serde_json::Map::new();
                for (name, value) in &form_map {
                    let field_selector = format!(
                        "{} [name=\"{}\"]",
                        form_selector_owned,
                        name.replace('\\', "\\\\").replace('"', "\\\"")
                    );
                    match rt.block_on(guard.type_text_into_element(&field_selector, value, true)) {
                        Ok(resp) => {
                            fields.insert(
                                name.clone(),
                                json!({"filled": resp.success, "message": resp.message}),
                            );
                        }
                        Err(e) => {
                            return McpResponse::error(
                                -1,
                                format!("Failed to fill field '{}': {}", name, e),
                            );
                        }
                    }
                }
                let all_filled = fields.values().all(|f| f["filled"] == true);
                McpResponse::success(json!({
                    "success": all_filled,
                    "submitted": false,
                    "fields": fields,
                }))
            });
        }

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                let mut potential_new_window_info = None;

                // Check if this form would open new windows when submitted
                let matching_form = guard.get_analyzed_forms().iter().find(|form| {
                    form.selector == form_selector_owned
                        || form.selector.contains(&form_selector_owned)
                });

                if let Some(form_info) = matching_form
                    && form_info.opens_new_window
                {
                    potential_new_window_info = Some(json!({
                        "form_would_open_new_window": true,
                        "predicted_url": form_info.predicted_url,
                        "form_target": form_info.target,
                        "note": "Form has target='_blank' but programmatic submission bypasses this behavior"
                    }));
                }

                match rt.block_on(guard.submit_form(&form_selector_owned, form_map)) {
                    Ok(resp) => {
                        let mut resp_json = serde_json::to_value(&resp).unwrap_or_default();
                        if let Some(new_window_info) = potential_new_window_info
                            && let Some(obj) = resp_json.as_object_mut()
                        {
                            obj.insert("potential_new_window".to_string(), new_window_info);
                        }
                        McpResponse::success(resp_json)
                    }
                    Err(e) => McpResponse::error(-1, format!("Failed to submit form: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }

    /// `browser_fill`: set the value of a single field, optionally submitting
    /// its form afterwards (with every other field of the form).
    pub async fn handle_fill_field(&self, params: Value) -> McpResponse {
        let Some(selector) = params.get("selector").and_then(|v| v.as_str()) else {
            return McpResponse::error(-32602, "Missing required parameter: selector".to_string());
        };
        let Some(value) = params.get("value").and_then(|v| v.as_str()) else {
            return McpResponse::error(-32602, "Missing required parameter: value".to_string());
        };
        let submit = params
            .get("submit")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(selector, MAX_SELECTOR_LENGTH, "CSS selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = limit_input_length(value, MAX_FORM_VALUE_LENGTH, "Field value") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            let Ok(mut guard) = browser.lock() else {
                return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
            };
            let filled = match rt.block_on(guard.type_text_into_element(selector, value, true)) {
                Ok(resp) => resp,
                Err(e) => return McpResponse::error(-1, format!("Failed to fill field: {}", e)),
            };
            if !filled.success || !submit {
                return McpResponse::success(json!({
                    "success": filled.success,
                    "submitted": false,
                    "message": filled.message,
                }));
            }

            // Submit the form that contains the field
            match rt.block_on(guard.submit_form_containing(selector)) {
                Ok(resp) => McpResponse::success(json!({
                    "success": resp.success,
                    "submitted": true,
                    "message": resp.message,
                    "url": guard.get_current_url(),
                })),
                Err(e) => McpResponse::error(
                    -1,
                    format!("Field filled, but submitting its form failed: {}", e),
                ),
            }
        })
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

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let selector_owned = selector.to_string();

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            if let Ok(mut guard) = browser.lock() {
                match rt.block_on(guard.wait_for_element(&selector_owned, timeout)) {
                    Ok(found) => McpResponse::success(json!({
                        "found": found,
                        "selector": selector_owned,
                        "timeout_ms": timeout,
                        "message": if found {
                            format!("Element found: {}", selector_owned)
                        } else {
                            format!("Element not found after {}ms: {}", timeout, selector_owned)
                        }
                    })),
                    Err(e) => McpResponse::error(-1, format!("Failed to wait for element: {}", e)),
                }
            } else {
                McpResponse::error(-1, "Failed to acquire browser lock".to_string())
            }
        })
    }
}
