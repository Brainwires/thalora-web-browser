use anyhow::anyhow;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use url::Url;

use crate::engine::browser::HeadlessWebBrowser;
use crate::protocols::mcp::McpResponse;

/// Session identifier for managing browser sessions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserSession {
    pub session_id: String,
    #[serde(skip)]
    pub created_at: std::time::Instant,
    #[serde(skip)]
    pub last_accessed: std::time::Instant,
    pub current_url: Option<String>,
    pub persistent: bool,
    // Store creation time as Unix timestamp for persistence
    pub created_timestamp: u64,
    pub last_accessed_timestamp: u64,
}

impl BrowserSession {
    pub fn new(session_id: String, persistent: bool) -> Self {
        let now = std::time::Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            session_id,
            created_at: now,
            last_accessed: now,
            current_url: None,
            persistent,
            created_timestamp: timestamp,
            last_accessed_timestamp: timestamp,
        }
    }

    pub fn update_last_accessed(&mut self) {
        self.last_accessed = std::time::Instant::now();
        self.last_accessed_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Browser interaction tools for MCP
pub struct BrowserTools {
    sessions: Arc<Mutex<HashMap<String, BrowserSession>>>,
    session_file: PathBuf,
}

impl BrowserTools {
    pub fn new() -> Self {
        let session_file = Self::get_session_file_path();
        let mut browser_tools = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_file,
        };

        // Load existing sessions on startup
        if let Err(e) = browser_tools.load_sessions() {
            tracing::warn!("Failed to load sessions: {}", e);
        }

        browser_tools
    }

    fn get_session_file_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("thalora_browser_sessions.json");
        path
    }

    fn save_sessions(&self) {
        if let Ok(sessions) = self.sessions.lock() {
            // Only save persistent sessions
            let persistent_sessions: HashMap<String, BrowserSession> = sessions
                .iter()
                .filter(|(_, session)| session.persistent)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            if let Ok(json) = serde_json::to_string_pretty(&persistent_sessions) {
                if let Err(e) = fs::write(&self.session_file, json) {
                    tracing::warn!("Failed to save sessions: {}", e);
                }
            }
        }
    }

    fn load_sessions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.session_file.exists() {
            return Ok(());
        }

        let json = fs::read_to_string(&self.session_file)?;
        let saved_sessions: HashMap<String, BrowserSession> = serde_json::from_str(&json)?;

        if let Ok(mut sessions) = self.sessions.lock() {
            for (id, mut session) in saved_sessions {
                // Restore runtime fields with current time
                session.created_at = std::time::Instant::now();
                session.last_accessed = std::time::Instant::now();
                sessions.insert(id, session);
            }
        }

        Ok(())
    }

    /// Create a new browser session
    pub async fn create_session(&self, arguments: Value, _browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let session_id = arguments.get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("session_{}", uuid::Uuid::new_v4()))
            .to_string();

        let persistent = arguments.get("persistent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let description = arguments.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("New browser session");

        let session = BrowserSession::new(session_id.clone(), persistent);

        match self.sessions.lock() {
            Ok(mut sessions) => {
                sessions.insert(session_id.clone(), session);
                drop(sessions); // Release lock before saving

                // Save sessions if this one is persistent
                if persistent {
                    self.save_sessions();
                }

                McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Created browser session '{}': {}", session_id, description)
                    })],
                    is_error: false,
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to create session: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Get current session information
    pub async fn get_session(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let session_id = match arguments.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: session_id"
                    })],
                    is_error: true,
                };
            }
        };

        match self.sessions.lock() {
            Ok(mut sessions) => {
                if let Some(session) = sessions.get_mut(session_id) {
                    session.update_last_accessed();

                    // Get browser state
                    let (storage_state, cookies_info) = match browser.lock() {
                        Ok(browser) => {
                            let storage = browser.get_storage_state().unwrap_or_default();
                            let cookies = if let Some(ref url) = session.current_url {
                                browser.get_cookies(url).unwrap_or_default()
                            } else {
                                HashMap::new()
                            };
                            (storage, cookies)
                        }
                        Err(_) => (Default::default(), HashMap::new())
                    };

                    McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&json!({
                                "session_id": session.session_id,
                                "created_at": session.created_at.elapsed().as_secs(),
                                "last_accessed": session.last_accessed.elapsed().as_secs(),
                                "current_url": session.current_url,
                                "persistent": session.persistent,
                                "storage": storage_state,
                                "cookies_count": cookies_info.len()
                            })).unwrap_or_else(|_| "Failed to serialize session data".to_string())
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Session '{}' not found", session_id)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access sessions: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Close a browser session
    pub async fn close_session(&self, arguments: Value, _browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let session_id = match arguments.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: session_id"
                    })],
                    is_error: true,
                };
            }
        };

        match self.sessions.lock() {
            Ok(mut sessions) => {
                if let Some(removed_session) = sessions.remove(session_id) {
                    let was_persistent = removed_session.persistent;
                    drop(sessions); // Release lock before saving

                    // Save sessions if a persistent session was removed
                    if was_persistent {
                        self.save_sessions();
                    }

                    McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Closed browser session '{}'", session_id)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Session '{}' not found", session_id)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access sessions: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Navigate to a URL within a session
    pub async fn navigate(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let session_id = arguments.get("session_id").and_then(|v| v.as_str());
        let url = match arguments.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: url"
                    })],
                    is_error: true,
                };
            }
        };

        let wait_for_js = arguments.get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Update session if provided
        if let Some(session_id) = session_id {
            if let Ok(mut sessions) = self.sessions.lock() {
                if let Some(session) = sessions.get_mut(session_id) {
                    session.current_url = Some(url.to_string());
                    session.last_accessed = std::time::Instant::now();
                }
            }
        }

        match browser.lock() {
            Ok(mut browser) => {
                match browser.scrape(url, wait_for_js, None, false, false).await {
                    Ok(scraped_data) => {
                        McpResponse::ToolResult {
                            content: vec![json!({
                                "type": "text",
                                "text": serde_json::to_string_pretty(&json!({
                                    "url": scraped_data.url,
                                    "title": scraped_data.title,
                                    "content_length": scraped_data.content.len(),
                                    "navigation_successful": true
                                })).unwrap_or_else(|_| "Failed to serialize navigation result".to_string())
                            })],
                            is_error: false,
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Navigation failed: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Find elements on the current page
    pub async fn find_elements(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let selector = match arguments.get("selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: selector"
                    })],
                    is_error: true,
                };
            }
        };

        let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("about:blank");

        match browser.lock() {
            Ok(mut browser) => {
                match browser.scrape(url, true, Some(selector), false, false).await {
                    Ok(scraped_data) => {
                        McpResponse::ToolResult {
                            content: vec![json!({
                                "type": "text",
                                "text": serde_json::to_string_pretty(&json!({
                                    "selector": selector,
                                    "found_content": scraped_data.content,
                                    "content_length": scraped_data.content.len(),
                                    "elements_found": !scraped_data.content.trim().is_empty()
                                })).unwrap_or_else(|_| "Failed to serialize element search result".to_string())
                            })],
                            is_error: false,
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Element search failed: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Get page state including forms and content
    pub async fn get_page_state(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("about:blank");
        let include_forms = arguments.get("include_forms")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match browser.lock() {
            Ok(mut browser) => {
                match browser.scrape(url, true, None, true, true).await {
                    Ok(scraped_data) => {
                        let mut result = json!({
                            "url": scraped_data.url,
                            "title": scraped_data.title,
                            "content": scraped_data.content,
                            "links": scraped_data.links,
                            "images": scraped_data.images,
                            "metadata": scraped_data.metadata
                        });

                        if include_forms {
                            if let Ok(base_url) = Url::parse(url) {
                                match browser.extract_forms(&scraped_data.content, &base_url) {
                                    Ok(forms) => {
                                        result["forms"] = json!(forms);
                                    }
                                    Err(e) => {
                                        result["forms_error"] = json!(format!("Failed to extract forms: {}", e));
                                    }
                                }
                            }
                        }

                        McpResponse::ToolResult {
                            content: vec![json!({
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Failed to serialize page state".to_string())
                            })],
                            is_error: false,
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Failed to get page state: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Fill form fields
    pub async fn fill_form(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let form_data_value = match arguments.get("form_data") {
            Some(data) => data,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: form_data"
                    })],
                    is_error: true,
                };
            }
        };

        let form_data: HashMap<String, String> = match serde_json::from_value(form_data_value.clone()) {
            Ok(data) => data,
            Err(e) => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Invalid form_data format: {}", e)
                    })],
                    is_error: true,
                };
            }
        };

        let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("about:blank");
        let _form_selector = arguments.get("form_selector").and_then(|v| v.as_str()).unwrap_or("form");

        match browser.lock() {
            Ok(mut browser) => {
                // First, get the current page to extract forms
                match browser.scrape(url, true, None, false, false).await {
                    Ok(scraped_data) => {
                        if let Ok(base_url) = Url::parse(url) {
                            match browser.extract_forms(&scraped_data.content, &base_url) {
                                Ok(forms) => {
                                    if let Some(form) = forms.first() {
                                        match browser.submit_form(form, form_data.clone(), true).await {
                                            Ok(response) => {
                                                McpResponse::ToolResult {
                                                    content: vec![json!({
                                                        "type": "text",
                                                        "text": serde_json::to_string_pretty(&json!({
                                                            "form_submitted": true,
                                                            "final_url": response.url,
                                                            "status_code": response.status_code,
                                                            "form_data": form_data,
                                                            "response_length": response.content.len()
                                                        })).unwrap_or_else(|_| "Failed to serialize form submission result".to_string())
                                                    })],
                                                    is_error: false,
                                                }
                                            }
                                            Err(e) => McpResponse::ToolResult {
                                                content: vec![json!({
                                                    "type": "text",
                                                    "text": format!("Form submission failed: {}", e)
                                                })],
                                                is_error: true,
                                            }
                                        }
                                    } else {
                                        McpResponse::ToolResult {
                                            content: vec![json!({
                                                "type": "text",
                                                "text": "No forms found on the page"
                                            })],
                                            is_error: true,
                                        }
                                    }
                                }
                                Err(e) => McpResponse::ToolResult {
                                    content: vec![json!({
                                        "type": "text",
                                        "text": format!("Failed to extract forms: {}", e)
                                    })],
                                    is_error: true,
                                }
                            }
                        } else {
                            McpResponse::ToolResult {
                                content: vec![json!({
                                    "type": "text",
                                    "text": "Invalid URL format"
                                })],
                                is_error: true,
                            }
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Failed to load page: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Click an element (link or button)
    pub async fn click_element(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let selector = match arguments.get("selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: selector"
                    })],
                    is_error: true,
                };
            }
        };

        let url = match arguments.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: url"
                    })],
                    is_error: true,
                };
            }
        };

        let wait_for_js = arguments.get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        match browser.lock() {
            Ok(mut browser) => {
                match browser.click_link(url, selector, wait_for_js).await {
                    Ok(response) => {
                        McpResponse::ToolResult {
                            content: vec![json!({
                                "type": "text",
                                "text": serde_json::to_string_pretty(&json!({
                                    "click_successful": true,
                                    "final_url": response.url,
                                    "status_code": response.status_code,
                                    "selector": selector,
                                    "response_length": response.content.len()
                                })).unwrap_or_else(|_| "Failed to serialize click result".to_string())
                            })],
                            is_error: false,
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("Click failed: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Execute JavaScript code
    pub async fn execute_javascript(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let code = match arguments.get("code").and_then(|v| v.as_str()) {
            Some(code) => code,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: code"
                    })],
                    is_error: true,
                };
            }
        };

        match browser.lock() {
            Ok(mut browser) => {
                match browser.execute_javascript(code).await {
                    Ok(result) => {
                        McpResponse::ToolResult {
                            content: vec![json!({
                                "type": "text",
                                "text": serde_json::to_string_pretty(&json!({
                                    "execution_successful": true,
                                    "code": code,
                                    "result": format!("{:?}", result)
                                })).unwrap_or_else(|_| "Failed to serialize JavaScript execution result".to_string())
                            })],
                            is_error: false,
                        }
                    }
                    Err(e) => McpResponse::ToolResult {
                        content: vec![json!({
                            "type": "text",
                            "text": format!("JavaScript execution failed: {}", e)
                        })],
                        is_error: true,
                    }
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access browser: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// List all active browser sessions
    pub async fn list_sessions(&self, _arguments: Value, _browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        match self.sessions.lock() {
            Ok(sessions) => {
                let session_list: Vec<_> = sessions.iter().map(|(id, session)| {
                    json!({
                        "session_id": id,
                        "created_at": session.created_at.elapsed().as_secs(),
                        "last_accessed": session.last_accessed.elapsed().as_secs(),
                        "current_url": session.current_url,
                        "persistent": session.persistent
                    })
                }).collect();

                McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&json!({
                            "active_sessions": session_list,
                            "total_sessions": sessions.len()
                        })).unwrap_or_else(|_| "Failed to serialize session list".to_string())
                    })],
                    is_error: false,
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access sessions: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Close all active browser sessions
    pub async fn close_all_sessions(&self, _arguments: Value, _browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        match self.sessions.lock() {
            Ok(mut sessions) => {
                let session_count = sessions.len();
                sessions.clear();

                McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": format!("Closed {} browser sessions", session_count)
                    })],
                    is_error: false,
                }
            }
            Err(e) => McpResponse::ToolResult {
                content: vec![json!({
                    "type": "text",
                    "text": format!("Failed to access sessions: {}", e)
                })],
                is_error: true,
            }
        }
    }

    /// Wait for an element to appear or disappear
    pub async fn wait_for_element(&self, arguments: Value, browser: &Arc<Mutex<HeadlessWebBrowser>>) -> McpResponse {
        let selector = match arguments.get("selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => {
                return McpResponse::ToolResult {
                    content: vec![json!({
                        "type": "text",
                        "text": "Missing required parameter: selector"
                    })],
                    is_error: true,
                };
            }
        };

        let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("about:blank");
        let timeout_ms = arguments.get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);
        let expect_visible = arguments.get("expect_visible")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        while start_time.elapsed() < timeout_duration {
            match browser.lock() {
                Ok(mut browser) => {
                    match browser.scrape(url, true, Some(selector), false, false).await {
                        Ok(scraped_data) => {
                            let element_found = !scraped_data.content.trim().is_empty();

                            if element_found == expect_visible {
                                return McpResponse::ToolResult {
                                    content: vec![json!({
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&json!({
                                            "wait_successful": true,
                                            "selector": selector,
                                            "element_found": element_found,
                                            "waited_ms": start_time.elapsed().as_millis(),
                                            "content": scraped_data.content
                                        })).unwrap_or_else(|_| "Failed to serialize wait result".to_string())
                                    })],
                                    is_error: false,
                                };
                            }
                        }
                        Err(_) => {
                            // Continue waiting on errors
                        }
                    }
                }
                Err(_) => {
                    // Continue waiting on browser access errors
                }
            }

            // Wait a bit before retrying
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        McpResponse::ToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Wait timeout: Element '{}' not found within {}ms", selector, timeout_ms)
            })],
            is_error: true,
        }
    }
}