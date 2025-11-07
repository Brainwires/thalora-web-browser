use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vfs::fs;

// Module declarations
mod crypto;
mod search;
mod storage;
mod types;

// Re-export public types
pub use types::{
    BookmarkEntry, CredentialEntry, MemoryData, MemoryMetadata, MemorySearchCriteria,
    MemorySortBy, MemoryStatistics, NoteEntry, NotePriority, ResearchEntry, SessionData,
    SessionStatus,
};

/// AI Memory Heap - Persistent cache system for AI agents
/// Stores research data, credentials, and other information across context compressions
#[derive(Debug, Clone)]
pub struct AiMemoryHeap {
    cache_file: PathBuf,
    memory_data: MemoryData,
}

impl AiMemoryHeap {
    /// Create a new AI memory heap with specified cache file location
    pub fn new<P: AsRef<Path>>(cache_file: P) -> Result<Self> {
        let cache_file = cache_file.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let memory_data = if cache_file.exists() {
            Self::load_from_file(&cache_file)?
        } else {
            MemoryData {
                research: HashMap::new(),
                credentials: HashMap::new(),
                sessions: HashMap::new(),
                bookmarks: HashMap::new(),
                notes: HashMap::new(),
                metadata: types::MemoryMetadata {
                    version: "1.0.0".to_string(),
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                    total_entries: 0,
                    total_size_bytes: 0,
                    compression_count: 0,
                    last_cleanup: None,
                },
            }
        };

        Ok(Self {
            cache_file,
            memory_data,
        })
    }

    /// Create memory heap in default location (~/.thalora/ai_memory.json)
    pub fn new_default() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let cache_file = home_dir.join(".thalora").join("ai_memory.json");
        Self::new(cache_file)
    }

    /// Load memory data from file
    fn load_from_file(path: &Path) -> Result<MemoryData> {
        let content = fs::read_to_string(path)?;
        let memory_data: MemoryData = serde_json::from_str(&content)?;
        Ok(memory_data)
    }

    /// Save memory data to file
    pub fn save(&mut self) -> Result<()> {
        self.memory_data.metadata.last_updated = Utc::now();
        self.memory_data.metadata.total_entries = self.count_total_entries();

        let json = serde_json::to_string_pretty(&self.memory_data)?;
        self.memory_data.metadata.total_size_bytes = json.len() as u64;

        fs::write(&self.cache_file, json)?;
        Ok(())
    }

    /// Count total entries across all categories
    fn count_total_entries(&self) -> u64 {
        (self.memory_data.research.len()
            + self.memory_data.credentials.len()
            + self.memory_data.sessions.len()
            + self.memory_data.bookmarks.len()
            + self.memory_data.notes.len()) as u64
    }

    // === RESEARCH MANAGEMENT ===

    /// Store research findings
    pub fn store_research(&mut self, key: &str, research: ResearchEntry) -> Result<()> {
        storage::store_research(&mut self.memory_data, key, research)?;
        self.save()?;
        Ok(())
    }

    /// Get research entry by key
    pub fn get_research(&self, key: &str) -> Option<&ResearchEntry> {
        storage::get_research(&self.memory_data, key)
    }

    /// Update research entry
    pub fn update_research<F>(&mut self, key: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut ResearchEntry),
    {
        let updated = storage::update_research(&mut self.memory_data, key, updater)?;
        if updated {
            self.save()?;
        }
        Ok(updated)
    }

    /// Search research entries
    pub fn search_research<'a>(
        &'a self,
        criteria: &'a MemorySearchCriteria,
    ) -> Vec<(&'a String, &'a ResearchEntry)> {
        search::search_research(&self.memory_data, criteria)
    }

    // === CREDENTIAL MANAGEMENT ===

    /// Store credentials (password will be encrypted)
    pub fn store_credentials(
        &mut self,
        key: &str,
        service: &str,
        username: &str,
        password: &str,
        additional_data: HashMap<String, String>,
    ) -> Result<()> {
        storage::store_credentials(
            &mut self.memory_data,
            key,
            service,
            username,
            password,
            additional_data,
        )?;
        self.save()?;
        Ok(())
    }

    /// Get credentials (password will be decrypted)
    pub fn get_credentials(
        &mut self,
        key: &str,
    ) -> Result<Option<(String, String, String, HashMap<String, String>)>> {
        let result = storage::get_credentials(&mut self.memory_data, key)?;
        if result.is_some() {
            self.save()?; // Save updated last_used timestamp
        }
        Ok(result)
    }

    /// List all stored credential keys (without exposing passwords)
    pub fn list_credential_keys(&self) -> Vec<(&String, &str, &str)> {
        storage::list_credential_keys(&self.memory_data)
    }

    // === SESSION MANAGEMENT ===

    /// Start new session
    pub fn start_session(
        &mut self,
        session_id: &str,
        context: &str,
        objectives: Vec<String>,
    ) -> Result<()> {
        storage::start_session(&mut self.memory_data, session_id, context, objectives)?;
        self.save()?;
        Ok(())
    }

    /// Update session progress
    pub fn update_session_progress(
        &mut self,
        session_id: &str,
        key: &str,
        value: Value,
    ) -> Result<bool> {
        let updated =
            storage::update_session_progress(&mut self.memory_data, session_id, key, value)?;
        if updated {
            self.save()?;
        }
        Ok(updated)
    }

    /// Mark task as completed in session
    pub fn complete_session_task(&mut self, session_id: &str, task: &str) -> Result<bool> {
        let updated = storage::complete_session_task(&mut self.memory_data, session_id, task)?;
        if updated {
            self.save()?;
        }
        Ok(updated)
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<(&String, &SessionData)> {
        storage::get_active_sessions(&self.memory_data)
    }

    // === BOOKMARK MANAGEMENT ===

    /// Search bookmarks
    pub fn search_bookmarks<'a>(
        &'a self,
        criteria: &'a MemorySearchCriteria,
    ) -> Vec<(&'a String, &'a BookmarkEntry)> {
        search::search_bookmarks(&self.memory_data, criteria)
    }

    /// Store bookmark
    pub fn store_bookmark(
        &mut self,
        key: &str,
        url: &str,
        title: &str,
        description: &str,
        content_preview: &str,
        tags: Vec<String>,
    ) -> Result<()> {
        storage::store_bookmark(
            &mut self.memory_data,
            key,
            url,
            title,
            description,
            content_preview,
            tags,
        )?;
        self.save()?;
        Ok(())
    }

    /// Access bookmark (increments counter)
    pub fn access_bookmark(&mut self, key: &str) -> Option<BookmarkEntry> {
        let result = storage::access_bookmark(&mut self.memory_data, key);
        if result.is_some() {
            drop(self.save()); // Ignore save errors for access tracking
        }
        result
    }

    // === NOTE MANAGEMENT ===

    /// Store note
    pub fn store_note(
        &mut self,
        key: &str,
        title: &str,
        content: &str,
        category: &str,
        tags: Vec<String>,
        priority: NotePriority,
    ) -> Result<()> {
        storage::store_note(
            &mut self.memory_data,
            key,
            title,
            content,
            category,
            tags,
            priority,
        )?;
        self.save()?;
        Ok(())
    }

    /// Update note
    pub fn update_note<F>(&mut self, key: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut types::NoteEntry),
    {
        let updated = storage::update_note(&mut self.memory_data, key, updater)?;
        if updated {
            self.save()?;
        }
        Ok(updated)
    }

    /// Search notes
    pub fn search_notes<'a>(&'a self, criteria: &'a MemorySearchCriteria) -> Vec<(&'a String, &'a NoteEntry)> {
        search::search_notes(&self.memory_data, criteria)
    }

    // === UTILITY METHODS ===

    /// Get memory statistics
    pub fn get_statistics(&self) -> MemoryStatistics {
        MemoryStatistics {
            research_count: self.memory_data.research.len(),
            credential_count: self.memory_data.credentials.len(),
            session_count: self.memory_data.sessions.len(),
            bookmark_count: self.memory_data.bookmarks.len(),
            note_count: self.memory_data.notes.len(),
            total_entries: self.count_total_entries(),
            file_size_bytes: self.memory_data.metadata.total_size_bytes,
            last_updated: self.memory_data.metadata.last_updated,
            version: self.memory_data.metadata.version.clone(),
        }
    }

    /// Export memory data as JSON
    pub fn export_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.memory_data)?)
    }

    /// Import memory data from JSON (merges with existing data)
    pub fn import_json(&mut self, json: &str) -> Result<()> {
        let imported_data: MemoryData = serde_json::from_str(json)?;

        // Merge data
        self.memory_data.research.extend(imported_data.research);
        self.memory_data
            .credentials
            .extend(imported_data.credentials);
        self.memory_data.sessions.extend(imported_data.sessions);
        self.memory_data.bookmarks.extend(imported_data.bookmarks);
        self.memory_data.notes.extend(imported_data.notes);

        self.save()?;
        Ok(())
    }

    /// Clean up old entries based on age and usage
    pub fn cleanup_old_entries(&mut self, max_age_days: i64) -> Result<usize> {
        let removed_count = storage::cleanup_old_entries(&mut self.memory_data, max_age_days)?;

        if removed_count > 0 {
            self.save()?;
        }

        Ok(removed_count)
    }
}
