use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::engine::browser::HeadlessWebBrowser;
use crate::protocols::browser_tools::session::BrowserSession;

#[allow(dead_code)]
pub struct BrowserTools {
    pub(super) sessions: Arc<Mutex<HashMap<String, (Arc<Mutex<HeadlessWebBrowser>>, BrowserSession)>>>,
    pub(super) persistent_session_path: Option<PathBuf>,
}

impl BrowserTools {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            persistent_session_path: None,
        }
    }

    pub fn get_or_create_session(&self, session_id: &str, persistent: bool) -> Arc<Mutex<HeadlessWebBrowser>> {
        let mut sessions = self.sessions.lock().unwrap();

        if let Some((browser, session)) = sessions.get_mut(session_id) {
            eprintln!("🔍 DEBUG: get_or_create_session - FOUND existing session: {}", session_id);
            session.update_last_accessed();
            // Debug browser state
            if let Ok(browser_guard) = browser.try_lock() {
                eprintln!("🔍 DEBUG: get_or_create_session - existing browser content length: {}", browser_guard.get_current_content().len());
                eprintln!("🔍 DEBUG: get_or_create_session - existing browser URL: {:?}", browser_guard.get_current_url());
            }
            // Return existing browser with preserved state
            browser.clone()
        } else {
            eprintln!("🔍 DEBUG: get_or_create_session - CREATING new session: {}", session_id);
            let browser = HeadlessWebBrowser::new();
            let session = BrowserSession::new(session_id.to_string(), persistent);

            // Set persistent data path for session storage
            if persistent {
                if let Ok(mut browser_guard) = browser.lock() {
                    browser_guard.get_storage_mut().session_storage.insert(
                        "_session_id".to_string(),
                        session_id.to_string()
                    );
                }
            }

            sessions.insert(session_id.to_string(), (browser.clone(), session));

            if persistent {
                drop(self.save_session(session_id));
            }

            browser
        }
    }

    pub fn get_session_info(&self, session_id: &str) -> Option<BrowserSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|(_, session)| session.clone())
    }

    /// Get browser from an existing session without creating a new one
    /// Returns None if the session doesn't exist
    pub fn get_session_browser(&self, session_id: &str) -> Option<Arc<Mutex<HeadlessWebBrowser>>> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some((browser, session)) = sessions.get_mut(session_id) {
            session.update_last_accessed();
            Some(browser.clone())
        } else {
            None
        }
    }

    pub fn list_sessions(&self) -> Vec<BrowserSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().map(|(_, session)| session.clone()).collect()
    }

    pub fn close_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some((browser, session)) = sessions.remove(session_id) {
            eprintln!("🧹 Closing session: {}", session_id);
            // Explicitly drop browser to trigger cleanup
            drop(browser);

            if session.persistent {
                drop(self.remove_persistent_session(session_id));
            }
            true
        } else {
            false
        }
    }

    fn save_session(&self, _session_id: &str) -> Result<(), std::io::Error> {
        // Implementation for saving persistent sessions
        // For now, just return Ok
        Ok(())
    }

    fn remove_persistent_session(&self, _session_id: &str) -> Result<(), std::io::Error> {
        // Implementation for removing persistent sessions
        // For now, just return Ok
        Ok(())
    }

    pub fn cleanup_expired_sessions(&self, max_age_seconds: u64) {
        let mut sessions = self.sessions.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expired_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, (_, session))| {
                !session.persistent && (now - session.last_accessed_timestamp) > max_age_seconds
            })
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            sessions.remove(&session_id);
        }
    }
}

impl Default for BrowserTools {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BrowserTools {
    fn drop(&mut self) {
        eprintln!("🧹 BrowserTools shutting down, closing all sessions");
        if let Ok(mut sessions) = self.sessions.lock() {
            let session_ids: Vec<String> = sessions.keys().cloned().collect();
            eprintln!("🧹 Closing {} active session(s)", session_ids.len());

            for session_id in session_ids {
                if let Some((browser, _)) = sessions.remove(&session_id) {
                    drop(browser);
                }
            }
            sessions.clear();
        }
    }
}