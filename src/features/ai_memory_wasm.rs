//! AI Memory Heap WASM stub
//!
//! In WASM builds, filesystem-based storage is not available.
//! This module provides type-compatible stubs that use browser localStorage/IndexedDB
//! through the platform abstraction layer.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Re-export public types
pub use types::{
    BookmarkEntry, CredentialEntry, MemoryData, MemoryMetadata, MemorySearchCriteria,
    MemorySortBy, MemoryStatistics, NoteEntry, NotePriority, ResearchEntry, SessionData,
    SessionStatus,
};

mod types {
    use super::*;

    /// Main memory data structure
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryData {
        pub research: HashMap<String, ResearchEntry>,
        pub credentials: HashMap<String, CredentialEntry>,
        pub sessions: HashMap<String, SessionData>,
        pub bookmarks: HashMap<String, BookmarkEntry>,
        pub notes: HashMap<String, NoteEntry>,
        pub metadata: MemoryMetadata,
    }

    /// Research entry for storing investigation results
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResearchEntry {
        pub topic: String,
        pub summary: String,
        pub findings: Vec<String>,
        pub sources: Vec<String>,
        pub tags: Vec<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub confidence_score: f64,
        pub related_topics: Vec<String>,
    }

    /// Secure credential storage (encrypted)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CredentialEntry {
        pub service: String,
        pub username: String,
        pub encrypted_password: String,
        pub additional_data: HashMap<String, String>,
        pub created_at: DateTime<Utc>,
        pub last_used: Option<DateTime<Utc>>,
        pub tags: Vec<String>,
    }

    /// Session tracking data
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SessionData {
        pub session_id: String,
        pub context: String,
        pub progress: HashMap<String, Value>,
        pub objectives: Vec<String>,
        pub completed_tasks: Vec<String>,
        pub created_at: DateTime<Utc>,
        pub last_activity: DateTime<Utc>,
        pub status: SessionStatus,
    }

    /// Bookmark entries for important URLs and resources
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BookmarkEntry {
        pub url: String,
        pub title: String,
        pub description: String,
        pub content_preview: String,
        pub tags: Vec<String>,
        pub created_at: DateTime<Utc>,
        pub last_accessed: Option<DateTime<Utc>>,
        pub access_count: u64,
        pub importance_score: f64,
    }

    /// Note entries for general information storage
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NoteEntry {
        pub title: String,
        pub content: String,
        pub category: String,
        pub tags: Vec<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub priority: NotePriority,
        pub related_entries: Vec<String>,
    }

    /// Memory metadata for tracking usage and maintenance
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryMetadata {
        pub version: String,
        pub created_at: DateTime<Utc>,
        pub last_updated: DateTime<Utc>,
        pub total_entries: u64,
        pub total_size_bytes: u64,
        pub compression_count: u64,
        pub last_cleanup: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SessionStatus {
        Active,
        Paused,
        Completed,
        Archived,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum NotePriority {
        Critical,
        High,
        Medium,
        Low,
        Archive,
    }

    /// Search criteria for memory queries
    #[derive(Debug, Clone)]
    pub struct MemorySearchCriteria {
        pub query: Option<String>,
        pub tags: Option<Vec<String>>,
        pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
        pub category: Option<String>,
        pub limit: Option<usize>,
        pub sort_by: MemorySortBy,
    }

    #[derive(Debug, Clone)]
    pub enum MemorySortBy {
        CreatedAt,
        UpdatedAt,
        Relevance,
        Priority,
        AccessCount,
    }

    /// Memory statistics
    #[derive(Debug, Clone)]
    pub struct MemoryStatistics {
        pub research_count: usize,
        pub credential_count: usize,
        pub session_count: usize,
        pub bookmark_count: usize,
        pub note_count: usize,
        pub total_entries: u64,
        pub file_size_bytes: u64,
        pub last_updated: DateTime<Utc>,
        pub version: String,
    }

    impl Default for MemorySearchCriteria {
        fn default() -> Self {
            Self {
                query: None,
                tags: None,
                date_range: None,
                category: None,
                limit: None,
                sort_by: MemorySortBy::CreatedAt,
            }
        }
    }
}

/// AI Memory Heap - WASM implementation using localStorage
///
/// In WASM builds, this uses browser localStorage instead of filesystem.
/// Data is stored as JSON in localStorage with a "thalora_ai_memory" key.
#[derive(Debug, Clone)]
pub struct AiMemoryHeap {
    storage_key: String,
    memory_data: MemoryData,
}

impl AiMemoryHeap {
    /// Create a new AI memory heap with specified storage key
    pub fn new<P: AsRef<std::path::Path>>(_cache_file: P) -> Result<Self> {
        // In WASM, we ignore the file path and use localStorage instead
        let storage_key = "thalora_ai_memory".to_string();

        // Try to load existing data from localStorage
        let memory_data = Self::load_from_storage(&storage_key).unwrap_or_else(|_| {
            MemoryData {
                research: HashMap::new(),
                credentials: HashMap::new(),
                sessions: HashMap::new(),
                bookmarks: HashMap::new(),
                notes: HashMap::new(),
                metadata: MemoryMetadata {
                    version: "1.0.0".to_string(),
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                    total_entries: 0,
                    total_size_bytes: 0,
                    compression_count: 0,
                    last_cleanup: None,
                },
            }
        });

        Ok(Self {
            storage_key,
            memory_data,
        })
    }

    /// Create memory heap in default location (uses localStorage in WASM)
    pub fn new_default() -> Result<Self> {
        Self::new("thalora_ai_memory")
    }

    /// Load memory data from localStorage
    fn load_from_storage(key: &str) -> Result<MemoryData> {
        let window = web_sys::window()
            .ok_or_else(|| anyhow::anyhow!("No window object available"))?;
        let storage = window
            .local_storage()
            .map_err(|_| anyhow::anyhow!("Failed to access localStorage"))?
            .ok_or_else(|| anyhow::anyhow!("localStorage not available"))?;

        let value = storage
            .get_item(key)
            .map_err(|_| anyhow::anyhow!("Failed to get item from localStorage"))?
            .ok_or_else(|| anyhow::anyhow!("No data found in localStorage"))?;

        let memory_data: MemoryData = serde_json::from_str(&value)?;
        Ok(memory_data)
    }

    /// Save memory data to localStorage
    pub fn save(&mut self) -> Result<()> {
        self.memory_data.metadata.last_updated = Utc::now();
        self.memory_data.metadata.total_entries = self.count_total_entries();

        let json = serde_json::to_string(&self.memory_data)?;
        self.memory_data.metadata.total_size_bytes = json.len() as u64;

        let window = web_sys::window()
            .ok_or_else(|| anyhow::anyhow!("No window object available"))?;
        let storage = window
            .local_storage()
            .map_err(|_| anyhow::anyhow!("Failed to access localStorage"))?
            .ok_or_else(|| anyhow::anyhow!("localStorage not available"))?;

        storage
            .set_item(&self.storage_key, &json)
            .map_err(|_| anyhow::anyhow!("Failed to save to localStorage"))?;

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

    /// Get statistics about the memory heap
    pub fn get_statistics(&self) -> MemoryStatistics {
        MemoryStatistics {
            research_count: self.memory_data.research.len(),
            credential_count: self.memory_data.credentials.len(),
            session_count: self.memory_data.sessions.len(),
            bookmark_count: self.memory_data.bookmarks.len(),
            note_count: self.memory_data.notes.len(),
            total_entries: self.memory_data.metadata.total_entries,
            file_size_bytes: self.memory_data.metadata.total_size_bytes,
            last_updated: self.memory_data.metadata.last_updated,
            version: self.memory_data.metadata.version.clone(),
        }
    }

    /// Add a research entry
    pub fn add_research(&mut self, id: String, entry: ResearchEntry) -> Result<()> {
        self.memory_data.research.insert(id, entry);
        self.save()
    }

    /// Get a research entry
    pub fn get_research(&self, id: &str) -> Option<&ResearchEntry> {
        self.memory_data.research.get(id)
    }

    /// Add a credential entry
    pub fn add_credential(&mut self, id: String, entry: CredentialEntry) -> Result<()> {
        self.memory_data.credentials.insert(id, entry);
        self.save()
    }

    /// Get a credential entry
    pub fn get_credential(&self, id: &str) -> Option<&CredentialEntry> {
        self.memory_data.credentials.get(id)
    }

    /// Add a session
    pub fn add_session(&mut self, id: String, session: SessionData) -> Result<()> {
        self.memory_data.sessions.insert(id, session);
        self.save()
    }

    /// Get a session
    pub fn get_session(&self, id: &str) -> Option<&SessionData> {
        self.memory_data.sessions.get(id)
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, id: String, bookmark: BookmarkEntry) -> Result<()> {
        self.memory_data.bookmarks.insert(id, bookmark);
        self.save()
    }

    /// Get a bookmark
    pub fn get_bookmark(&self, id: &str) -> Option<&BookmarkEntry> {
        self.memory_data.bookmarks.get(id)
    }

    /// Add a note
    pub fn add_note(&mut self, id: String, note: NoteEntry) -> Result<()> {
        self.memory_data.notes.insert(id, note);
        self.save()
    }

    /// Get a note
    pub fn get_note(&self, id: &str) -> Option<&NoteEntry> {
        self.memory_data.notes.get(id)
    }

    /// Search all entries
    pub fn search(&self, _criteria: &MemorySearchCriteria) -> Vec<String> {
        // Simple implementation - returns all entry IDs
        let mut ids: Vec<String> = Vec::new();
        ids.extend(self.memory_data.research.keys().cloned());
        ids.extend(self.memory_data.credentials.keys().cloned());
        ids.extend(self.memory_data.sessions.keys().cloned());
        ids.extend(self.memory_data.bookmarks.keys().cloned());
        ids.extend(self.memory_data.notes.keys().cloned());
        ids
    }

    /// Clear all data
    pub fn clear(&mut self) -> Result<()> {
        self.memory_data.research.clear();
        self.memory_data.credentials.clear();
        self.memory_data.sessions.clear();
        self.memory_data.bookmarks.clear();
        self.memory_data.notes.clear();
        self.save()
    }
}
