use futures::future::LocalBoxFuture;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;
use tracing::{debug, info};

use crate::engine::browser::BrowserThread;
use crate::engine::engine_trait::EngineType;
use crate::protocols::browser_tools::session::BrowserSession;
use crate::protocols::mcp::McpResponse;

/// A session's browser, living on its own thread (see [`BrowserThread`]).
pub type SessionBrowser = BrowserThread;

/// The browser as seen by jobs running on its thread.
pub use crate::engine::browser::session_thread::BrowserHandle;

/// Map of session ID to (browser thread, session metadata).
pub(super) type SessionMap = HashMap<String, (SessionBrowser, BrowserSession)>;

/// Run `job` with a session's browser on its thread. A failed or panicked
/// job becomes an MCP error response.
pub(crate) async fn run_in<F>(browser: &SessionBrowser, job: F) -> McpResponse
where
    F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, McpResponse> + Send + 'static,
{
    browser
        .call(job)
        .await
        .unwrap_or_else(|e| McpResponse::error(-1, format!("Browser session failed: {}", e)))
}

/// Current URL and HTML content of a session's page.
pub(crate) async fn page_state(
    browser: &SessionBrowser,
) -> Result<(Option<String>, String), String> {
    use futures::FutureExt;
    browser
        .call(|browser| {
            async move {
                match browser.lock() {
                    Ok(guard) => Ok((guard.get_current_url(), guard.get_current_content())),
                    Err(_) => Err("Failed to acquire browser lock".to_string()),
                }
            }
            .boxed_local()
        })
        .await
        .map_err(|e| format!("Browser session failed: {}", e))?
}

#[allow(dead_code)]
pub struct BrowserTools {
    pub(super) sessions: Rc<Mutex<SessionMap>>,
    pub(super) persistent_session_path: Option<PathBuf>,
}

impl BrowserTools {
    pub fn new() -> Self {
        Self {
            sessions: Rc::new(Mutex::new(HashMap::new())),
            persistent_session_path: None,
        }
    }

    /// Get a session's browser, starting a new browser thread if needed.
    pub fn get_or_create_session(
        &self,
        session_id: &str,
        persistent: bool,
    ) -> Result<SessionBrowser, String> {
        let mut sessions = self.sessions.lock().unwrap();

        if let Some((browser, session)) = sessions.get_mut(session_id) {
            debug!(session_id, "Found existing session");
            session.update_last_accessed();
            return Ok(browser.clone());
        }

        debug!(session_id, "Creating new session");
        let browser = BrowserThread::spawn(session_id, EngineType::Boa).map_err(|e| {
            format!(
                "Failed to start browser for session '{}': {}",
                session_id, e
            )
        })?;
        let session = BrowserSession::new(session_id.to_string(), persistent);
        sessions.insert(session_id.to_string(), (browser.clone(), session));
        drop(sessions);

        if persistent {
            drop(self.save_session(session_id));
        }

        Ok(browser)
    }

    /// Look up a session for tools that act on an existing page.
    ///
    /// Unlike [`get_or_create_session`](Self::get_or_create_session) this does
    /// not silently create a blank browser for a mistyped ID: unknown IDs are
    /// an error listing the sessions that do exist. The `"default"` session is
    /// always available and is created on first use.
    pub fn get_session(&self, session_id: &str) -> Result<SessionBrowser, String> {
        if session_id == "default" {
            return self.get_or_create_session(session_id, false);
        }
        if let Some(browser) = self.get_session_browser(session_id) {
            return Ok(browser);
        }
        let mut known: Vec<String> = self.sessions.lock().unwrap().keys().cloned().collect();
        known.sort();
        Err(format!(
            "Unknown session '{}'. Navigate with this session_id or create it via \
             browser_session_management first. Existing sessions: [{}]",
            session_id,
            known.join(", ")
        ))
    }

    pub fn get_session_info(&self, session_id: &str) -> Option<BrowserSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|(_, session)| session.clone())
    }

    /// Get browser from an existing session without creating a new one
    /// Returns None if the session doesn't exist
    pub fn get_session_browser(&self, session_id: &str) -> Option<SessionBrowser> {
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
        sessions
            .values()
            .map(|(_, session)| session.clone())
            .collect()
    }

    pub fn close_session(&self, session_id: &str) -> bool {
        let removed = self.sessions.lock().unwrap().remove(session_id);
        if let Some((browser, session)) = removed {
            info!(session_id, "Closing session");
            // Stop the browser thread; the browser is dropped on that thread
            browser.shutdown_blocking();

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
        info!("BrowserTools shutting down, closing all sessions");
        let mut sessions = self.sessions.lock().unwrap();
        let session_ids: Vec<String> = sessions.keys().cloned().collect();
        info!(count = session_ids.len(), "Closing active sessions");

        // Dropping the last handle stops each browser thread, which drops
        // its browser on its own thread.
        for session_id in session_ids {
            sessions.remove(&session_id);
        }
        sessions.clear();
    }
}
