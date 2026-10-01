use futures::FutureExt;
use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_SELECTOR_LENGTH, limit_input_length, sanitize_session_id};

impl BrowserTools {
    pub async fn handle_prepare_form_submission(&self, params: Value) -> McpResponse {
        let form_selector = params["form_selector"].as_str().unwrap_or("");
        let submit_button_selector = params
            .get("submit_button_selector")
            .and_then(|v| v.as_str());
        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if form_selector.is_empty() {
            return McpResponse::error(-1, "Form selector is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Err(e) = limit_input_length(form_selector, MAX_SELECTOR_LENGTH, "Form selector") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Some(btn_sel) = submit_button_selector
            && let Err(e) =
                limit_input_length(btn_sel, MAX_SELECTOR_LENGTH, "Submit button selector")
        {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }
        if let Err(e) = sanitize_session_id(session_id) {
            return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
        }

        let browser = match self.get_session(session_id) {
            Ok(browser) => browser,
            Err(e) => return McpResponse::error(-32602, e),
        };
        // Read the analysed forms on the browser thread, decide here
        let forms = browser
            .call(|browser| {
                async move {
                    browser.lock().ok().map(|guard| {
                        let new_window: Vec<_> =
                            guard.get_new_window_forms().into_iter().cloned().collect();
                        let all = guard.get_analyzed_forms().to_vec();
                        (new_window, all)
                    })
                }
                .boxed_local()
            })
            .await;
        let (new_window_forms, all_forms) = match forms {
            Ok(Some(forms)) => forms,
            Ok(None) => {
                return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
            }
            Err(e) => return McpResponse::error(-1, format!("Browser session failed: {}", e)),
        };
        let response;

        {
            {
                // Find forms that match the selector and open new windows
                let matching_form = new_window_forms.iter().find(|form| {
                        // Check if the form selector matches
                        form.selector == form_selector ||
                        form.selector.contains(form_selector) ||
                        // If submit button selector provided, check if it matches
                        submit_button_selector.is_some_and(|btn_sel| {
                            form.submit_buttons.iter().any(|btn| btn == btn_sel || btn.contains(btn_sel))
                        })
                    });

                if let Some(form_info) = matching_form {
                    if let Some(ref predicted_url) = form_info.predicted_url {
                        // Create predictive session for the form submission
                        let predictive_session_id = format!(
                            "predictive_{}_{}",
                            session_id,
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        );

                        tracing::debug!(
                            "Creating predictive session for form preparation: {}",
                            predictive_session_id
                        );

                        // Create the predictive session
                        if let Err(e) = self.get_or_create_session(&predictive_session_id, false) {
                            return McpResponse::error(-1, e);
                        }

                        response = McpResponse::success(json!({
                            "success": true,
                            "message": "Predictive session created for form that opens new window".to_string(),
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
                        response = McpResponse::error(
                            -1,
                            "Form found but no predicted URL available".to_string(),
                        );
                    }
                } else {
                    // Check if any form matches the selector but doesn't open new windows
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
                        response = McpResponse::error(
                            -1,
                            format!("No form found matching selector: {}", form_selector),
                        );
                    }
                }
            }
        }
        response
    }
}
