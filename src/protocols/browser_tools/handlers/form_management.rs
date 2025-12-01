use serde_json::{json, Value};

use crate::protocols::mcp::McpResponse;
use crate::protocols::browser_tools::core::BrowserTools;

impl BrowserTools {
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
}
