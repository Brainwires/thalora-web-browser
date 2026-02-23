use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

/// Controls whether artificial anti-bot delays are applied during navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMode {
    /// GUI mode: no artificial delays, fastest possible navigation.
    Interactive,
    /// MCP/headless mode: human-like random delays for anti-bot evasion.
    Stealth,
}

impl Default for NavigationMode {
    fn default() -> Self {
        NavigationMode::Stealth
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedData {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<Link>,
    pub images: Vec<Image>,
    pub metadata: HashMap<String, String>,
    pub extracted_data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: Option<String>,
    pub required: bool,
    pub placeholder: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionResponse {
    pub success: bool,
    pub message: String,
    pub redirect_url: Option<String>,
    pub new_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BrowserStorage {
    pub local_storage: HashMap<String, String>,
    pub session_storage: HashMap<String, String>,
}

impl Default for BrowserStorage {
    fn default() -> Self {
        Self {
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub cookies: HashMap<String, String>,
    pub auth_headers: HashMap<String, String>,
    pub csrf_tokens: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    pub entries: Vec<HistoryEntry>,
    pub current_index: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct StealthConfig {
    pub random_delays: bool,
    pub random_user_agents: bool,
    pub stealth_headers: bool,
    pub canvas_fingerprinting_protection: bool,
    pub webgl_fingerprinting_protection: bool,
    pub audio_fingerprinting_protection: bool,
    pub font_fingerprinting_protection: bool,
    pub screen_resolution_spoofing: bool,
    pub timezone_spoofing: bool,
    pub language_spoofing: bool,
    pub webrtc_protection: bool,
    pub battery_api_protection: bool,
    pub gamepad_api_protection: bool,
    pub request_timing: RequestTiming,
}

#[derive(Debug, Clone)]
pub struct RequestTiming {
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            random_delays: true,
            random_user_agents: true,
            stealth_headers: true,
            canvas_fingerprinting_protection: true,
            webgl_fingerprinting_protection: true,
            audio_fingerprinting_protection: true,
            font_fingerprinting_protection: true,
            screen_resolution_spoofing: true,
            timezone_spoofing: true,
            language_spoofing: true,
            webrtc_protection: true,
            battery_api_protection: true,
            gamepad_api_protection: true,
            request_timing: RequestTiming {
                min_delay_ms: 100,
                max_delay_ms: 2000,
            },
        }
    }
}

/// A cached resource (stylesheet or script content).
#[derive(Debug, Clone)]
pub struct CachedResource {
    pub content: String,
    pub fetched_at: Instant,
    pub url: String,
}

/// In-memory cache for fetched stylesheets and scripts, with LRU eviction.
#[derive(Debug)]
pub struct ResourceCache {
    entries: HashMap<String, CachedResource>,
    /// Insertion order for LRU eviction (oldest first)
    insertion_order: Vec<String>,
    max_entries: usize,
}

impl ResourceCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            insertion_order: Vec::new(),
            max_entries,
        }
    }

    /// Look up a cached resource by URL.
    pub fn get(&self, url: &str) -> Option<&CachedResource> {
        self.entries.get(url)
    }

    /// Insert a resource into the cache. Evicts the oldest entry if at capacity.
    pub fn insert(&mut self, url: String, content: String) {
        // If already cached, remove old position in insertion order
        if self.entries.contains_key(&url) {
            self.insertion_order.retain(|u| u != &url);
        }

        // Evict oldest if at capacity
        while self.entries.len() >= self.max_entries && !self.insertion_order.is_empty() {
            let oldest = self.insertion_order.remove(0);
            self.entries.remove(&oldest);
        }

        self.entries.insert(url.clone(), CachedResource {
            content,
            fetched_at: Instant::now(),
            url: url.clone(),
        });
        self.insertion_order.push(url);
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for ResourceCache {
    fn default() -> Self {
        Self::new(500)
    }
}

/// Events emitted by the JavaScript History API for GUI synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HistoryEvent {
    #[serde(rename = "pushState")]
    PushState { url: String, state_json: Option<String> },
    #[serde(rename = "replaceState")]
    ReplaceState { url: String, state_json: Option<String> },
    #[serde(rename = "popstate")]
    PopState { url: String, state_json: Option<String>, delta: i32 },
}