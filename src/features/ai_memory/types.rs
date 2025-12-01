use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    pub confidence_score: f64, // 0.0 to 1.0
    pub related_topics: Vec<String>,
}

/// Secure credential storage (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub service: String,
    pub username: String,
    pub encrypted_password: String, // Base64 encoded encrypted password
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
    pub importance_score: f64, // 0.0 to 1.0
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
