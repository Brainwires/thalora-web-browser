use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use vfs::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

/// AI Memory Heap - Persistent cache system for AI agents
/// Stores research data, credentials, and other information across context compressions
#[derive(Debug, Clone)]
pub struct AiMemoryHeap {
    cache_file: PathBuf,
    memory_data: MemoryData,
}

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
        };
        
        Ok(Self {
            cache_file,
            memory_data,
        })
    }
    
    /// Create memory heap in default location (~/.thalora/ai_memory.json)
    pub fn new_default() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
        (self.memory_data.research.len() +
         self.memory_data.credentials.len() +
         self.memory_data.sessions.len() +
         self.memory_data.bookmarks.len() +
         self.memory_data.notes.len()) as u64
    }
    
    // === RESEARCH MANAGEMENT ===
    
    /// Store research findings
    pub fn store_research(&mut self, key: &str, research: ResearchEntry) -> Result<()> {
        self.memory_data.research.insert(key.to_string(), research);
        self.save()?;
        Ok(())
    }
    
    /// Get research entry by key
    pub fn get_research(&self, key: &str) -> Option<&ResearchEntry> {
        self.memory_data.research.get(key)
    }
    
    /// Update research entry
    pub fn update_research<F>(&mut self, key: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut ResearchEntry),
    {
        if let Some(research) = self.memory_data.research.get_mut(key) {
            research.updated_at = Utc::now();
            updater(research);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Search research entries
    pub fn search_research(&self, criteria: &MemorySearchCriteria) -> Vec<(&String, &ResearchEntry)> {
        let mut results: Vec<_> = self.memory_data.research
            .iter()
            .filter(|(_, entry)| self.matches_research_criteria(entry, criteria))
            .collect();
        
        self.sort_research_results(&mut results, &criteria.sort_by);
        
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    fn matches_research_criteria(&self, entry: &ResearchEntry, criteria: &MemorySearchCriteria) -> bool {
        // Query matching
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            if !entry.topic.to_lowercase().contains(&query_lower) &&
               !entry.summary.to_lowercase().contains(&query_lower) &&
               !entry.findings.iter().any(|f| f.to_lowercase().contains(&query_lower)) {
                return false;
            }
        }
        
        // Tag matching
        if let Some(tags) = &criteria.tags {
            if !tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }
        
        // Date range matching
        if let Some((start, end)) = &criteria.date_range {
            if entry.created_at < *start || entry.created_at > *end {
                return false;
            }
        }
        
        true
    }
    
    fn sort_research_results(&self, results: &mut Vec<(&String, &ResearchEntry)>, sort_by: &MemorySortBy) {
        match sort_by {
            MemorySortBy::CreatedAt => results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at)),
            MemorySortBy::UpdatedAt => results.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at)),
            MemorySortBy::Relevance => results.sort_by(|a, b| b.1.confidence_score.partial_cmp(&a.1.confidence_score).unwrap_or(std::cmp::Ordering::Equal)),
            _ => {} // Other sort types handled in specific contexts
        }
    }
    
    // === CREDENTIAL MANAGEMENT ===
    
    /// Store credentials (password will be encrypted)
    pub fn store_credentials(&mut self, key: &str, service: &str, username: &str, password: &str, additional_data: HashMap<String, String>) -> Result<()> {
        let encrypted_password = self.encrypt_password(password)?;
        
        let credential = CredentialEntry {
            service: service.to_string(),
            username: username.to_string(),
            encrypted_password,
            additional_data,
            created_at: Utc::now(),
            last_used: None,
            tags: vec![],
        };
        
        self.memory_data.credentials.insert(key.to_string(), credential);
        self.save()?;
        Ok(())
    }
    
    /// Get credentials (password will be decrypted)
    pub fn get_credentials(&mut self, key: &str) -> Result<Option<(String, String, String, HashMap<String, String>)>> {
        if let Some(cred) = self.memory_data.credentials.get(key) {
            let encrypted_password = cred.encrypted_password.clone();
            let password = self.decrypt_password(&encrypted_password)?;
            let result = Some((cred.service.clone(), cred.username.clone(), password, cred.additional_data.clone()));
            
            // Update last_used timestamp
            if let Some(cred_mut) = self.memory_data.credentials.get_mut(key) {
                cred_mut.last_used = Some(Utc::now());
                self.save()?;
            }
            
            Ok(result)
        } else {
            Ok(None)
        }
    }
    
    /// List all stored credential keys (without exposing passwords)
    pub fn list_credential_keys(&self) -> Vec<(&String, &str, &str)> {
        self.memory_data.credentials
            .iter()
            .map(|(key, cred)| (key, cred.service.as_str(), cred.username.as_str()))
            .collect()
    }
    
    /// Simple XOR encryption for passwords (not secure for production, but sufficient for dev cache)
    fn encrypt_password(&self, password: &str) -> Result<String> {
        let key = b"thalora_ai_memory_key_2025"; // Simple key
        let encrypted: Vec<u8> = password.bytes()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        Ok(general_purpose::STANDARD.encode(&encrypted))
    }
    
    fn decrypt_password(&self, encrypted_base64: &str) -> Result<String> {
        let key = b"thalora_ai_memory_key_2025"; // Same key
        let encrypted = general_purpose::STANDARD.decode(encrypted_base64)?;
        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[i % key.len()])
            .collect();
        Ok(String::from_utf8(decrypted)?)
    }
    
    // === SESSION MANAGEMENT ===
    
    /// Start new session
    pub fn start_session(&mut self, session_id: &str, context: &str, objectives: Vec<String>) -> Result<()> {
        let session = SessionData {
            session_id: session_id.to_string(),
            context: context.to_string(),
            progress: HashMap::new(),
            objectives,
            completed_tasks: vec![],
            created_at: Utc::now(),
            last_activity: Utc::now(),
            status: SessionStatus::Active,
        };
        
        self.memory_data.sessions.insert(session_id.to_string(), session);
        self.save()?;
        Ok(())
    }
    
    /// Update session progress
    pub fn update_session_progress(&mut self, session_id: &str, key: &str, value: Value) -> Result<bool> {
        if let Some(session) = self.memory_data.sessions.get_mut(session_id) {
            session.progress.insert(key.to_string(), value);
            session.last_activity = Utc::now();
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Mark task as completed in session
    pub fn complete_session_task(&mut self, session_id: &str, task: &str) -> Result<bool> {
        if let Some(session) = self.memory_data.sessions.get_mut(session_id) {
            session.completed_tasks.push(task.to_string());
            session.last_activity = Utc::now();
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<(&String, &SessionData)> {
        self.memory_data.sessions
            .iter()
            .filter(|(_, session)| matches!(session.status, SessionStatus::Active))
            .collect()
    }
    
    // === BOOKMARK MANAGEMENT ===
    
    /// Search bookmarks
    pub fn search_bookmarks(&self, criteria: &MemorySearchCriteria) -> Vec<(&String, &BookmarkEntry)> {
        let mut results: Vec<_> = self.memory_data.bookmarks
            .iter()
            .filter(|(_, entry)| self.matches_bookmark_criteria(entry, criteria))
            .collect();
        
        // Sort by access count for bookmarks
        results.sort_by(|a, b| b.1.access_count.cmp(&a.1.access_count));
        
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    fn matches_bookmark_criteria(&self, entry: &BookmarkEntry, criteria: &MemorySearchCriteria) -> bool {
        // Query matching
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            if !entry.title.to_lowercase().contains(&query_lower) &&
               !entry.description.to_lowercase().contains(&query_lower) &&
               !entry.url.to_lowercase().contains(&query_lower) {
                return false;
            }
        }
        
        // Tag matching
        if let Some(tags) = &criteria.tags {
            if !tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }
        
        // Date range matching
        if let Some((start, end)) = &criteria.date_range {
            if entry.created_at < *start || entry.created_at > *end {
                return false;
            }
        }
        
        true
    }
    
    /// Store bookmark
    pub fn store_bookmark(&mut self, key: &str, url: &str, title: &str, description: &str, content_preview: &str, tags: Vec<String>) -> Result<()> {
        let bookmark = BookmarkEntry {
            url: url.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            content_preview: content_preview.to_string(),
            tags,
            created_at: Utc::now(),
            last_accessed: None,
            access_count: 0,
            importance_score: 0.5,
        };
        
        self.memory_data.bookmarks.insert(key.to_string(), bookmark);
        self.save()?;
        Ok(())
    }
    
    /// Access bookmark (increments counter)
    pub fn access_bookmark(&mut self, key: &str) -> Option<BookmarkEntry> {
        if let Some(bookmark) = self.memory_data.bookmarks.get_mut(key) {
            bookmark.last_accessed = Some(Utc::now());
            bookmark.access_count += 1;
            let result = Some(bookmark.clone());
            let _ = self.save(); // Ignore save errors for access tracking
            result
        } else {
            None
        }
    }
    
    // === NOTE MANAGEMENT ===
    
    /// Store note
    pub fn store_note(&mut self, key: &str, title: &str, content: &str, category: &str, tags: Vec<String>, priority: NotePriority) -> Result<()> {
        let note = NoteEntry {
            title: title.to_string(),
            content: content.to_string(),
            category: category.to_string(),
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            priority,
            related_entries: vec![],
        };
        
        self.memory_data.notes.insert(key.to_string(), note);
        self.save()?;
        Ok(())
    }
    
    /// Update note
    pub fn update_note<F>(&mut self, key: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut NoteEntry),
    {
        if let Some(note) = self.memory_data.notes.get_mut(key) {
            note.updated_at = Utc::now();
            updater(note);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Search notes
    pub fn search_notes(&self, criteria: &MemorySearchCriteria) -> Vec<(&String, &NoteEntry)> {
        let mut results: Vec<_> = self.memory_data.notes
            .iter()
            .filter(|(_, entry)| self.matches_note_criteria(entry, criteria))
            .collect();
        
        self.sort_note_results(&mut results, &criteria.sort_by);
        
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    fn matches_note_criteria(&self, entry: &NoteEntry, criteria: &MemorySearchCriteria) -> bool {
        // Query matching
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            if !entry.title.to_lowercase().contains(&query_lower) &&
               !entry.content.to_lowercase().contains(&query_lower) {
                return false;
            }
        }
        
        // Category matching
        if let Some(category) = &criteria.category {
            if entry.category != *category {
                return false;
            }
        }
        
        // Tag matching
        if let Some(tags) = &criteria.tags {
            if !tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }
        
        // Date range matching
        if let Some((start, end)) = &criteria.date_range {
            if entry.created_at < *start || entry.created_at > *end {
                return false;
            }
        }
        
        true
    }
    
    fn sort_note_results(&self, results: &mut Vec<(&String, &NoteEntry)>, sort_by: &MemorySortBy) {
        match sort_by {
            MemorySortBy::CreatedAt => results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at)),
            MemorySortBy::UpdatedAt => results.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at)),
            MemorySortBy::Priority => results.sort_by(|a, b| {
                let a_priority = match a.1.priority {
                    NotePriority::Critical => 4,
                    NotePriority::High => 3,
                    NotePriority::Medium => 2,
                    NotePriority::Low => 1,
                    NotePriority::Archive => 0,
                };
                let b_priority = match b.1.priority {
                    NotePriority::Critical => 4,
                    NotePriority::High => 3,
                    NotePriority::Medium => 2,
                    NotePriority::Low => 1,
                    NotePriority::Archive => 0,
                };
                b_priority.cmp(&a_priority)
            }),
            _ => {} // Other sort types
        }
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
        self.memory_data.credentials.extend(imported_data.credentials);
        self.memory_data.sessions.extend(imported_data.sessions);
        self.memory_data.bookmarks.extend(imported_data.bookmarks);
        self.memory_data.notes.extend(imported_data.notes);
        
        self.save()?;
        Ok(())
    }
    
    /// Clean up old entries based on age and usage
    pub fn cleanup_old_entries(&mut self, max_age_days: i64) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(max_age_days);
        let mut removed_count = 0;
        
        // Clean up old sessions
        let old_sessions: Vec<String> = self.memory_data.sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff_date && !matches!(session.status, SessionStatus::Active))
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in old_sessions {
            self.memory_data.sessions.remove(&key);
            removed_count += 1;
        }
        
        // Clean up low-importance bookmarks
        let old_bookmarks: Vec<String> = self.memory_data.bookmarks
            .iter()
            .filter(|(_, bookmark)| {
                bookmark.created_at < cutoff_date &&
                bookmark.access_count == 0 &&
                bookmark.importance_score < 0.3
            })
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in old_bookmarks {
            self.memory_data.bookmarks.remove(&key);
            removed_count += 1;
        }
        
        // Clean up archived notes
        let old_notes: Vec<String> = self.memory_data.notes
            .iter()
            .filter(|(_, note)| {
                matches!(note.priority, NotePriority::Archive) &&
                note.updated_at < cutoff_date
            })
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in old_notes {
            self.memory_data.notes.remove(&key);
            removed_count += 1;
        }
        
        if removed_count > 0 {
            self.memory_data.metadata.last_cleanup = Some(Utc::now());
            self.save()?;
        }
        
        Ok(removed_count)
    }
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