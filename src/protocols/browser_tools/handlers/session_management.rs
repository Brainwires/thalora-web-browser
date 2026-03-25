use rand;
use serde_json::{Value, json};

use crate::protocols::browser_tools::core::BrowserTools;
use crate::protocols::mcp::McpResponse;

impl BrowserTools {
    pub async fn handle_session_management(&self, params: Value) -> McpResponse {
        let action = params["action"].as_str().unwrap_or("");

        match action {
            "create" => {
                let persistent = params
                    .get("persistent")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                // Generate a unique session ID using timestamp and random component
                let session_id = format!(
                    "session_{}_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
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
                let session_id = params.get("session_id").and_then(|v| v.as_str());
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
                let session_id = params.get("session_id").and_then(|v| v.as_str());
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
                let max_age = params
                    .get("max_age_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600); // Default 1 hour
                self.cleanup_expired_sessions(max_age);
                McpResponse::success(json!({"cleaned_up": true}))
            }
            _ => McpResponse::error(-1, format!("Unknown action: {}", action)),
        }
    }

    pub async fn handle_validate_session(&self, params: Value) -> McpResponse {
        let session_id = params["session_id"].as_str().unwrap_or("");
        let expected_url_pattern = params.get("expected_url_pattern").and_then(|v| v.as_str());
        let expected_content = params.get("expected_content").and_then(|v| v.as_str());
        let timeout = params
            .get("timeout")
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
                McpResponse::error(
                    -1,
                    "Session exists but browser instance not found".to_string(),
                )
            }
        } else {
            McpResponse::success(json!({
                "session_exists": false,
                "validation_successful": false,
                "message": format!("Session '{}' not found", session_id)
            }))
        }
    }
}
