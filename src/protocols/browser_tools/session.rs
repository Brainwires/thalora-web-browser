/// Session identifier for managing browser sessions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserSession {
    pub session_id: String,
    #[serde(skip, default = "std::time::Instant::now")]
    pub created_at: std::time::Instant,
    #[serde(skip, default = "std::time::Instant::now")]
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
