use futures::FutureExt;
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
        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
        let selector = params["selector"].as_str().unwrap_or("");
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if selector.is_empty() {
            return McpResponse::error(-1, "selector or ref is required".to_string());
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
        let session_id_owned = session_id.to_string();

        // The job returns the response plus a predictive session to create
        // (sessions live in BrowserTools, which stays on this thread).
        let outcome = browser
            .call(move |browser| {
                async move {
                    let mut predictive_session = None;
                    if let Ok(mut guard) = browser.lock() {
                        let mut potential_new_window_info = None;

                        // Check if this is a submit button for a form that opens new windows
                        if let Some(form_info) = guard.find_form_by_submit_button(&selector_owned)
                            && form_info.opens_new_window
                        {
                            tracing::debug!("Click on submit button for new window form detected");
                            tracing::debug!(
                                "Form target: {}, action: {}",
                                form_info.target,
                                form_info.action
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

                                tracing::debug!(
                                    "Creating predictive session: {} for URL: {}",
                                    predictive_session_id,
                                    predicted_url
                                );

                                predictive_session = Some(predictive_session_id.clone());

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

                        let response = match guard.click_element(&selector_owned).await {
                            Ok(resp) => {
                                if let Some(new_window_info) = potential_new_window_info {
                                    let mut resp_json =
                                        serde_json::to_value(&resp).unwrap_or_default();
                                    if let Some(obj) = resp_json.as_object_mut() {
                                        obj.insert(
                                            "potential_new_window".to_string(),
                                            new_window_info,
                                        );
                                    }
                                    McpResponse::success(resp_json)
                                } else {
                                    McpResponse::success(
                                        serde_json::to_value(resp).unwrap_or_default(),
                                    )
                                }
                            }
                            Err(e) => {
                                McpResponse::error(-1, format!("Failed to click element: {}", e))
                            }
                        };
                        (response, predictive_session)
                    } else {
                        (
                            McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                            None,
                        )
                    }
                }
                .boxed_local()
            })
            .await;

        match outcome {
            Ok((response, predictive_session)) => {
                if let Some(predictive_session_id) = predictive_session
                    && let Err(e) = self.get_or_create_session(&predictive_session_id, false)
                {
                    eprintln!("⚠️ Could not create predictive session: {}", e);
                }
                response
            }
            Err(e) => McpResponse::error(-1, format!("Browser session failed: {}", e)),
        }
    }

    pub async fn handle_type_text(&self, params: Value) -> McpResponse {
        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
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
            return McpResponse::error(-1, "selector or ref is required".to_string());
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

        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                if let Ok(mut guard) = browser.lock() {
                    match guard
                        .type_text_into_element(&selector_owned, &text_owned, clear_first)
                        .await
                    {
                        Ok(resp) => {
                            McpResponse::success(serde_json::to_value(resp).unwrap_or_default())
                        }
                        Err(e) => McpResponse::error(-1, format!("Failed to type text: {}", e)),
                    }
                } else {
                    McpResponse::error(-1, "Failed to acquire browser lock".to_string())
                }
            }
            .boxed_local()
        })
        .await
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
            return crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
                async move {
                    let Ok(mut guard) = browser.lock() else {
                        return McpResponse::error(
                            -1,
                            "Failed to acquire browser lock".to_string(),
                        );
                    };
                    let mut fields = serde_json::Map::new();
                    for (name, value) in &form_map {
                        let field_selector = format!(
                            "{} [name=\"{}\"]",
                            form_selector_owned,
                            name.replace('\\', "\\\\").replace('"', "\\\"")
                        );
                        match guard
                            .type_text_into_element(&field_selector, value, true)
                            .await
                        {
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
                }
                .boxed_local()
            })
            .await;
        }

        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
async move {
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

                match guard.submit_form(&form_selector_owned, form_map).await {
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
        }
.boxed_local()
})
.await
    }

    /// `browser_fill`: set the value of a single field, optionally submitting
    /// its form afterwards (with every other field of the form).
    pub async fn handle_fill_field(&self, params: Value) -> McpResponse {
        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
        let Some(selector) = params.get("selector").and_then(|v| v.as_str()) else {
            return McpResponse::error(
                -32602,
                "Missing required parameter: selector or ref".to_string(),
            );
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
        let selector = selector.to_string();
        let value = value.to_string();

        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                let filled = match guard.type_text_into_element(&selector, &value, true).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        return McpResponse::error(-1, format!("Failed to fill field: {}", e));
                    }
                };
                if !filled.success || !submit {
                    return McpResponse::success(json!({
                        "success": filled.success,
                        "submitted": false,
                        "message": filled.message,
                    }));
                }

                // Submit the form that contains the field
                match guard.submit_form_containing(&selector).await {
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
            }
            .boxed_local()
        })
        .await
    }

    /// Element actions: `browser_select_option`, `browser_check`,
    /// `browser_press_key`, `browser_hover`, `browser_scroll`.
    pub async fn handle_element_action(&self, tool: &str, params: Value) -> McpResponse {
        use crate::engine::browser::types::ElementAction;

        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        if let Err(e) = sanitize_session_id(&session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let selector = params
            .get("selector")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        if let Some(selector) = &selector
            && let Err(e) = limit_input_length(selector, MAX_SELECTOR_LENGTH, "CSS selector")
        {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        let string_param = |key: &str| {
            params
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| {
                    McpResponse::error(-32602, format!("Missing required parameter: {key}"))
                })
        };

        let action = match tool {
            "browser_select_option" => match string_param("value") {
                Ok(value) => ElementAction::SelectOption(value),
                Err(resp) => return resp,
            },
            "browser_check" => ElementAction::SetChecked(
                params
                    .get("checked")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            ),
            "browser_press_key" => match string_param("key") {
                Ok(key) if key.chars().count() <= 32 => ElementAction::PressKey(key),
                Ok(_) => return McpResponse::error(-32602, "key is too long".to_string()),
                Err(resp) => return resp,
            },
            "browser_hover" => ElementAction::Hover,
            "browser_scroll" => ElementAction::Scroll(
                params
                    .get("delta_y")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(600)
                    .clamp(-100_000, 100_000),
            ),
            other => return McpResponse::error(-32601, format!("Tool not found: {other}")),
        };

        let browser = match self.get_session(&session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        let tool = tool.to_string();
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                match guard.perform_action(selector.as_deref(), &action).await {
                    Ok(resp) => {
                        let mut out = serde_json::to_value(&resp).unwrap_or_default();
                        // The new page content is available via page tools
                        if let Some(obj) = out.as_object_mut() {
                            obj.remove("new_content");
                            obj.insert("url".to_string(), json!(guard.get_current_url()));
                        }
                        McpResponse::success(out)
                    }
                    Err(e) => McpResponse::error(-1, format!("{tool} failed: {e}")),
                }
            }
            .boxed_local()
        })
        .await
    }

    /// `browser_console_messages`: console output logged by the page.
    pub async fn handle_console_messages(&self, params: Value) -> McpResponse {
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let clear = params
            .get("clear")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                let messages: Vec<Value> = guard
                    .console_messages(clear)
                    .into_iter()
                    .map(|m| json!({"level": m.level, "text": m.text}))
                    .collect();
                let url = guard.get_current_url();
                McpResponse::page_content(
                    url.as_deref(),
                    json!({"count": messages.len(), "messages": messages}),
                )
            }
            .boxed_local()
        })
        .await
    }

    /// `browser_wait`: wait for a selector/ref, text, URL fragment or network idle.
    pub async fn handle_wait(&self, params: Value) -> McpResponse {
        use crate::engine::browser::types::WaitCondition;

        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        if let Err(e) = sanitize_session_id(&session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }
        let timeout_ms = params
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000)
            .min(60_000);

        let str_param = |key: &str| {
            params
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        let mut conditions = Vec::new();
        if let Some(selector) = str_param("selector") {
            if let Err(e) = limit_input_length(&selector, MAX_SELECTOR_LENGTH, "CSS selector") {
                return McpResponse::error(-32602, format!("Input validation failed: {}", e));
            }
            conditions.push(WaitCondition::Selector(selector));
        }
        if let Some(text) = str_param("text") {
            if let Err(e) = limit_input_length(&text, MAX_TEXT_INPUT_LENGTH, "Text") {
                return McpResponse::error(-32602, format!("Input validation failed: {}", e));
            }
            conditions.push(WaitCondition::Text(text));
        }
        if let Some(url) = str_param("url_contains") {
            conditions.push(WaitCondition::UrlContains(url));
        }
        if params.get("network_idle").and_then(|v| v.as_bool()) == Some(true) {
            conditions.push(WaitCondition::NetworkIdle);
        }
        if conditions.len() != 1 {
            return McpResponse::error(
                -32602,
                "Give exactly one of: ref, selector, text, url_contains, network_idle".to_string(),
            );
        }
        let condition = conditions.remove(0);

        let browser = match self.get_session(&session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                let Ok(mut guard) = browser.lock() else {
                    return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                };
                let started = std::time::Instant::now();
                match guard.wait_for_condition(&condition, timeout_ms).await {
                    Ok(met) => McpResponse::success(json!({
                        "met": met,
                        "condition": format!("{:?}", condition),
                        "waited_ms": started.elapsed().as_millis() as u64,
                        "url": guard.get_current_url(),
                    })),
                    Err(e) => McpResponse::error(-1, format!("Wait failed: {}", e)),
                }
            }
            .boxed_local()
        })
        .await
    }

    pub async fn handle_wait_for_element(&self, params: Value) -> McpResponse {
        let params = match self.resolve_ref_param(params).await {
            Ok(params) => params,
            Err(resp) => return resp,
        };
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
            return McpResponse::error(-1, "selector or ref is required".to_string());
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

        crate::protocols::browser_tools::core::run_in(&browser, move |browser| {
            async move {
                if let Ok(mut guard) = browser.lock() {
                    match guard.wait_for_element(&selector_owned, timeout).await {
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
                        Err(e) => {
                            McpResponse::error(-1, format!("Failed to wait for element: {}", e))
                        }
                    }
                } else {
                    McpResponse::error(-1, "Failed to acquire browser lock".to_string())
                }
            }
            .boxed_local()
        })
        .await
    }
}
