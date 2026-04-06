use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::protocols::security::sanitize_session_id;

/// Session information stored by the session manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub persistent: bool,
    pub socket_path: String,
}

/// Commands that can be sent to background browser processes
#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserCommand {
    /// Navigate to a URL
    Navigate { url: String },
    /// Navigate back in history
    NavigateBack,
    /// Navigate forward in history
    NavigateForward,
    /// Reload the current page
    Reload,
    /// Stop loading
    Stop,
    /// Execute JavaScript
    ExecuteJs { code: String },
    /// Get page content
    GetContent,
    /// Click an element
    Click { selector: String },
    /// Fill a form field
    Fill { selector: String, value: String },
    /// Take a screenshot
    Screenshot,
    /// Get cookies
    GetCookies,
    /// Set cookies
    SetCookies { cookies: Vec<String> },
    /// Close the session
    Close,
}

/// Response from background browser processes
#[derive(Debug, Serialize, Deserialize)]
pub enum BrowserResponse {
    Success { data: serde_json::Value },
    Error { message: String },
}

/// Manages browser sessions running in background processes
pub struct SessionManager {
    /// Active sessions and their process handles
    sessions: Arc<Mutex<HashMap<String, (SessionInfo, Child)>>>,
    /// Directory for Unix domain sockets
    socket_dir: std::path::PathBuf,
    /// Executable path for browser processes
    browser_executable: std::path::PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let socket_dir = std::env::temp_dir().join("thalora_sessions");
        std::fs::create_dir_all(&socket_dir).context("Failed to create socket directory")?;

        let browser_executable =
            std::env::current_exe().context("Failed to get current executable path")?;

        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            socket_dir,
            browser_executable,
        })
    }

    /// Get or create a browser session
    ///
    /// # Security
    /// The session_id is validated to prevent path traversal attacks (CWE-22).
    /// Only alphanumeric characters, hyphens, and underscores are allowed.
    pub async fn get_or_create_session(
        &self,
        session_id: &str,
        persistent: bool,
    ) -> Result<SessionInfo> {
        // SECURITY: Validate session_id to prevent path traversal attacks
        let safe_session_id =
            sanitize_session_id(session_id).context("Invalid session_id format")?;

        let mut sessions = self.sessions.lock().await;

        // Check if session already exists
        if let Some((session_info, _process)) = sessions.get_mut(&safe_session_id) {
            session_info.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Ok(session_info.clone());
        }

        // Create new session with sanitized ID
        let socket_path = self.socket_dir.join(format!("{}.sock", safe_session_id));
        let socket_path_str = socket_path.to_string_lossy().to_string();

        info!(
            "Spawning new browser process for session: {}",
            safe_session_id
        );

        // Spawn background browser process using session subcommand
        let mut cmd = Command::new(&self.browser_executable);
        cmd.arg("session")
            .arg("--session-id")
            .arg(&safe_session_id)
            .arg("--socket-path")
            .arg(&socket_path_str);

        // Add --persistent flag only if true (it's a boolean flag, not a value)
        if persistent {
            cmd.arg("--persistent");
        }

        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped()) // Capture output for debugging
            .stderr(Stdio::piped()) // Capture errors for debugging
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn browser process: {:?}",
                    self.browser_executable
                )
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session_info = SessionInfo {
            session_id: safe_session_id.clone(),
            created_at: now,
            last_accessed: now,
            persistent,
            socket_path: socket_path_str,
        };

        sessions.insert(safe_session_id.clone(), (session_info.clone(), child));

        // Poll for socket readiness with exponential backoff instead of fixed sleep
        {
            let socket_path = std::path::Path::new(&session_info.socket_path);
            let mut delay_ms = 10u64;
            let max_total_ms = 2000u64;
            let mut elapsed_ms = 0u64;

            while elapsed_ms < max_total_ms {
                if socket_path.exists() {
                    debug!(
                        session_id = %safe_session_id,
                        elapsed_ms,
                        "Socket file appeared"
                    );
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                elapsed_ms += delay_ms;
                delay_ms = (delay_ms * 2).min(200);
            }

            if !socket_path.exists() {
                warn!(
                    session_id = %safe_session_id,
                    timeout_ms = max_total_ms,
                    "Socket file not found after timeout"
                );
            }
        }

        info!("Created new browser session: {}", safe_session_id);
        Ok(session_info)
    }

    /// Send a command to a browser session
    ///
    /// # Security
    /// The session_id is validated to prevent path traversal attacks.
    pub async fn send_command(
        &self,
        session_id: &str,
        command: BrowserCommand,
    ) -> Result<BrowserResponse> {
        // SECURITY: Validate session_id
        let safe_session_id =
            sanitize_session_id(session_id).context("Invalid session_id format")?;

        let session_info = {
            let sessions = self.sessions.lock().await;
            sessions
                .get(&safe_session_id)
                .map(|(info, _)| info.clone())
                .context("Session not found")?
        };

        debug!(
            "Sending command to session {}: {:?}",
            safe_session_id, command
        );

        // Connect to the Unix socket
        let mut stream = UnixStream::connect(&session_info.socket_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to session socket: {}",
                    session_info.socket_path
                )
            })?;

        // Send command
        let command_json =
            serde_json::to_string(&command).context("Failed to serialize command")?;

        stream
            .write_all(command_json.as_bytes())
            .await
            .context("Failed to write command to socket")?;
        stream
            .write_all(b"\n")
            .await
            .context("Failed to write newline to socket")?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .context("Failed to read response from socket")?;

        let response: BrowserResponse =
            serde_json::from_str(response_line.trim()).context("Failed to deserialize response")?;

        debug!(
            "Received response from session {}: {:?}",
            safe_session_id, response
        );
        Ok(response)
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().await;
        sessions.values().map(|(info, _)| info.clone()).collect()
    }

    /// Close a specific session
    ///
    /// # Security
    /// The session_id is validated to prevent path traversal attacks.
    pub async fn close_session(&self, session_id: &str) -> Result<bool> {
        // SECURITY: Validate session_id
        let safe_session_id =
            sanitize_session_id(session_id).context("Invalid session_id format")?;

        let mut sessions = self.sessions.lock().await;

        if let Some((session_info, mut process)) = sessions.remove(&safe_session_id) {
            info!("Closing browser session: {}", safe_session_id);

            // Try to gracefully terminate the process
            if let Err(e) = process.kill() {
                warn!(
                    "Failed to kill browser process for session {}: {}",
                    safe_session_id, e
                );
            }

            // Clean up socket file
            if let Err(e) = std::fs::remove_file(&session_info.socket_path) {
                warn!(
                    "Failed to remove socket file {}: {}",
                    session_info.socket_path, e
                );
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self, max_age_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expired_sessions: Vec<String> = {
            let sessions = self.sessions.lock().await;
            sessions
                .iter()
                .filter(|(_, (info, _))| {
                    !info.persistent && (now - info.last_accessed) > max_age_seconds
                })
                .map(|(id, _)| id.clone())
                .collect()
        };

        for session_id in expired_sessions {
            info!("Cleaning up expired session: {}", session_id);
            if let Err(e) = self.close_session(&session_id).await {
                error!("Failed to close expired session {}: {}", session_id, e);
            }
        }
    }

    /// Shutdown all sessions
    pub async fn shutdown(&self) {
        let session_ids: Vec<String> = {
            let sessions = self.sessions.lock().await;
            sessions.keys().cloned().collect()
        };

        if !session_ids.is_empty() {
            info!("Shutting down {} browser session(s)", session_ids.len());

            for session_id in session_ids {
                if let Err(e) = self.close_session(&session_id).await {
                    error!(
                        "Failed to close session {} during shutdown: {}",
                        session_id, e
                    );
                }
            }
        }
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Best effort cleanup - try to lock synchronously
        let sessions = self.sessions.clone();
        let socket_dir = self.socket_dir.clone();

        std::thread::spawn(move || {
            // Try to get lock with blocking
            let mut sessions = sessions.blocking_lock();
            for (session_id, (session_info, mut process)) in sessions.drain() {
                if let Err(e) = process.kill() {
                    tracing::warn!(
                        session_id,
                        error = %e,
                        "Failed to kill browser process during drop"
                    );
                }
                if let Err(e) = std::fs::remove_file(&session_info.socket_path) {
                    tracing::warn!(
                        path = %session_info.socket_path,
                        error = %e,
                        "Failed to remove socket file during drop"
                    );
                }
            }

            // Try to clean up socket directory
            if let Err(e) = std::fs::remove_dir_all(&socket_dir) {
                tracing::warn!(error = %e, "Failed to remove socket directory during drop");
            }
        });
    }
}
