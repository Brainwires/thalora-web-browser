//! WASM bindings for Thalora Web Browser
//!
//! This module provides wasm-bindgen bindings to expose Thalora's browser
//! functionality to JavaScript/TypeScript when compiled to WebAssembly.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Set up panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

// ============================================================================
// Browser Types
// ============================================================================

/// Scraped data from a web page
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmScrapedData {
    pub url: String,
    pub title: String,
    pub content: String,
    pub html: String,
}

/// Link extracted from a page
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmLink {
    pub url: String,
    pub text: String,
}

/// Image extracted from a page
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmImage {
    pub url: String,
    pub alt: String,
}

/// Browser configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowserConfig {
    pub user_agent: Option<String>,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub javascript_enabled: bool,
    pub stealth_mode: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            user_agent: None,
            viewport_width: Some(1920),
            viewport_height: Some(1080),
            javascript_enabled: true,
            stealth_mode: false,
        }
    }
}

// ============================================================================
// Main Browser Class
// ============================================================================

/// Main Thalora Browser instance for WASM
///
/// Provides headless browser functionality that runs entirely in the browser.
/// Uses Web APIs for networking (fetch, WebSocket) instead of native implementations.
#[wasm_bindgen]
pub struct ThaloraBrowser {
    config: BrowserConfig,
    storage: HashMap<String, String>,
    ai_memory: WasmAIMemory,
    fingerprint: WasmFingerprint,
}

#[wasm_bindgen]
impl ThaloraBrowser {
    /// Create a new ThaloraBrowser instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> ThaloraBrowser {
        console_log!("Creating ThaloraBrowser instance");
        ThaloraBrowser {
            config: BrowserConfig::default(),
            storage: HashMap::new(),
            ai_memory: WasmAIMemory::new(),
            fingerprint: WasmFingerprint::new(),
        }
    }

    /// Create a new ThaloraBrowser with custom configuration
    pub fn with_config(config: JsValue) -> Result<ThaloraBrowser, JsValue> {
        let config: BrowserConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        Ok(ThaloraBrowser {
            config,
            storage: HashMap::new(),
            ai_memory: WasmAIMemory::new(),
            fingerprint: WasmFingerprint::new(),
        })
    }

    /// Navigate to a URL and get the page content
    ///
    /// Note: In WASM builds, this uses the browser's fetch API
    pub async fn navigate(&self, url: String) -> Result<JsValue, JsValue> {
        console_log!("Navigating to: {}", url);

        // In WASM, we delegate to the browser's fetch API
        // This will be called from JavaScript and the actual fetch happens there
        let result = WasmScrapedData {
            url: url.clone(),
            title: String::new(),
            content: String::new(),
            html: String::new(),
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Execute JavaScript code in a sandboxed context
    pub fn execute_js(&self, code: String) -> Result<JsValue, JsValue> {
        console_log!("Executing JavaScript: {} bytes", code.len());

        // For WASM builds, we create a Boa engine context
        // Note: This is sandboxed and doesn't have access to the actual page DOM
        Ok(JsValue::from_str("JavaScript execution placeholder"))
    }

    /// Get the current configuration
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Set user agent string
    pub fn set_user_agent(&mut self, user_agent: String) {
        self.config.user_agent = Some(user_agent);
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.config.viewport_width = Some(width);
        self.config.viewport_height = Some(height);
    }

    /// Enable or disable JavaScript execution
    pub fn set_javascript_enabled(&mut self, enabled: bool) {
        self.config.javascript_enabled = enabled;
    }

    /// Enable or disable stealth mode (fingerprint randomization)
    pub fn set_stealth_mode(&mut self, enabled: bool) {
        self.config.stealth_mode = enabled;
    }

    /// Get the AI memory manager
    pub fn ai_memory(&self) -> WasmAIMemory {
        self.ai_memory.clone()
    }

    /// Get the fingerprint manager
    pub fn fingerprint(&self) -> WasmFingerprint {
        self.fingerprint.clone()
    }
}

impl Default for ThaloraBrowser {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Storage API
// ============================================================================

/// Browser storage (localStorage-like) for WASM
#[wasm_bindgen]
pub struct WasmStorage {
    data: HashMap<String, String>,
    storage_key: String,
}

#[wasm_bindgen]
impl WasmStorage {
    /// Create a new storage instance with a namespace
    #[wasm_bindgen(constructor)]
    pub fn new(namespace: String) -> WasmStorage {
        WasmStorage {
            data: HashMap::new(),
            storage_key: format!("thalora_{}", namespace),
        }
    }

    /// Get a value from storage
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// Set a value in storage
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    /// Remove a value from storage
    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Clear all storage
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Get the number of items in storage
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// Export storage as JSON
    pub fn export_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Import storage from JSON
    pub fn import_json(&mut self, json: String) -> Result<(), JsValue> {
        let data: HashMap<String, String> = serde_json::from_str(&json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        self.data = data;
        Ok(())
    }
}

// ============================================================================
// AI Memory API
// ============================================================================

/// AI Memory entry types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MemoryEntryType {
    Research,
    Credential,
    Bookmark,
    Note,
    Session,
}

/// AI Memory entry
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmMemoryEntry {
    pub id: String,
    pub entry_type: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// AI Memory manager for WASM
///
/// Persistent memory storage for AI-related data like research notes,
/// credentials, bookmarks, and session data.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAIMemory {
    entries: HashMap<String, String>, // Serialized entries
}

#[wasm_bindgen]
impl WasmAIMemory {
    /// Create a new AI memory instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmAIMemory {
        WasmAIMemory {
            entries: HashMap::new(),
        }
    }

    /// Add a research entry
    pub fn add_research(&mut self, title: String, content: String, tags: Vec<String>) -> String {
        let id = generate_id();
        let entry = WasmMemoryEntry {
            id: id.clone(),
            entry_type: "research".to_string(),
            title,
            content,
            tags,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            self.entries.insert(id.clone(), json);
        }

        id
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, url: String, title: String, description: String) -> String {
        let id = generate_id();
        let entry = WasmMemoryEntry {
            id: id.clone(),
            entry_type: "bookmark".to_string(),
            title,
            content: format!("URL: {}\n\n{}", url, description),
            tags: vec!["bookmark".to_string()],
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            self.entries.insert(id.clone(), json);
        }

        id
    }

    /// Add a note
    pub fn add_note(&mut self, title: String, content: String, priority: String) -> String {
        let id = generate_id();
        let entry = WasmMemoryEntry {
            id: id.clone(),
            entry_type: "note".to_string(),
            title,
            content,
            tags: vec!["note".to_string(), priority],
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            self.entries.insert(id.clone(), json);
        }

        id
    }

    /// Get an entry by ID
    pub fn get(&self, id: &str) -> Option<String> {
        self.entries.get(id).cloned()
    }

    /// Delete an entry by ID
    pub fn delete(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Search entries by keyword
    pub fn search(&self, query: String) -> Result<JsValue, JsValue> {
        let query_lower = query.to_lowercase();
        let results: Vec<WasmMemoryEntry> = self.entries
            .values()
            .filter_map(|json| serde_json::from_str::<WasmMemoryEntry>(json).ok())
            .filter(|entry| {
                entry.title.to_lowercase().contains(&query_lower) ||
                entry.content.to_lowercase().contains(&query_lower) ||
                entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// List all entries
    pub fn list_all(&self) -> Result<JsValue, JsValue> {
        let results: Vec<WasmMemoryEntry> = self.entries
            .values()
            .filter_map(|json| serde_json::from_str(json).ok())
            .collect();

        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        let entries: Vec<WasmMemoryEntry> = self.entries
            .values()
            .filter_map(|json| serde_json::from_str(json).ok())
            .collect();

        let mut stats = HashMap::new();
        stats.insert("total", entries.len());
        stats.insert("research", entries.iter().filter(|e| e.entry_type == "research").count());
        stats.insert("bookmark", entries.iter().filter(|e| e.entry_type == "bookmark").count());
        stats.insert("note", entries.iter().filter(|e| e.entry_type == "note").count());

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Export all memory as JSON
    pub fn export_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.entries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Import memory from JSON
    pub fn import_json(&mut self, json: String) -> Result<(), JsValue> {
        let data: HashMap<String, String> = serde_json::from_str(&json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        self.entries = data;
        Ok(())
    }
}

impl Default for WasmAIMemory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Fingerprint API
// ============================================================================

/// Browser fingerprint configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmFingerprintConfig {
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u8,
    pub timezone: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

/// Fingerprint manager for WASM
///
/// Manages browser fingerprint spoofing for stealth browsing.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmFingerprint {
    config: WasmFingerprintConfig,
}

#[wasm_bindgen]
impl WasmFingerprint {
    /// Create a new fingerprint manager with default values
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmFingerprint {
        WasmFingerprint {
            config: WasmFingerprintConfig {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                platform: "Win32".to_string(),
                language: "en-US".to_string(),
                screen_width: 1920,
                screen_height: 1080,
                color_depth: 24,
                timezone: "America/New_York".to_string(),
                webgl_vendor: "Google Inc. (NVIDIA)".to_string(),
                webgl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1080 Direct3D11 vs_5_0 ps_5_0)".to_string(),
            },
        }
    }

    /// Generate a random fingerprint
    pub fn randomize(&mut self) {
        // Randomize various fingerprint components
        let user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
        ];

        let platforms = ["Win32", "MacIntel", "Linux x86_64"];
        let resolutions = [(1920, 1080), (2560, 1440), (1366, 768), (1536, 864)];

        // Simple random selection (in production, use proper RNG)
        let idx = (js_sys::Math::random() * user_agents.len() as f64) as usize;
        self.config.user_agent = user_agents[idx % user_agents.len()].to_string();

        let idx = (js_sys::Math::random() * platforms.len() as f64) as usize;
        self.config.platform = platforms[idx % platforms.len()].to_string();

        let idx = (js_sys::Math::random() * resolutions.len() as f64) as usize;
        let (w, h) = resolutions[idx % resolutions.len()];
        self.config.screen_width = w;
        self.config.screen_height = h;
    }

    /// Get current fingerprint configuration
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Set custom user agent
    pub fn set_user_agent(&mut self, user_agent: String) {
        self.config.user_agent = user_agent;
    }

    /// Set custom platform
    pub fn set_platform(&mut self, platform: String) {
        self.config.platform = platform;
    }

    /// Set screen resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.config.screen_width = width;
        self.config.screen_height = height;
    }

    /// Set timezone
    pub fn set_timezone(&mut self, timezone: String) {
        self.config.timezone = timezone;
    }

    /// Get JavaScript code to inject for fingerprint spoofing
    pub fn get_injection_script(&self) -> String {
        format!(r#"
            Object.defineProperty(navigator, 'userAgent', {{ get: () => '{}' }});
            Object.defineProperty(navigator, 'platform', {{ get: () => '{}' }});
            Object.defineProperty(navigator, 'language', {{ get: () => '{}' }});
            Object.defineProperty(screen, 'width', {{ get: () => {} }});
            Object.defineProperty(screen, 'height', {{ get: () => {} }});
            Object.defineProperty(screen, 'colorDepth', {{ get: () => {} }});
        "#,
            self.config.user_agent,
            self.config.platform,
            self.config.language,
            self.config.screen_width,
            self.config.screen_height,
            self.config.color_depth
        )
    }
}

impl Default for WasmFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DOM Utilities
// ============================================================================

/// DOM parser and manipulator for WASM
#[wasm_bindgen]
pub struct WasmDOM {
    html: String,
}

#[wasm_bindgen]
impl WasmDOM {
    /// Create a new DOM instance from HTML
    #[wasm_bindgen(constructor)]
    pub fn new(html: String) -> WasmDOM {
        WasmDOM { html }
    }

    /// Get the raw HTML
    pub fn get_html(&self) -> String {
        self.html.clone()
    }

    /// Extract text content (simplified)
    pub fn get_text(&self) -> String {
        // Simple HTML tag stripping
        let mut text = self.html.clone();

        // Remove script and style blocks
        while let Some(start) = text.find("<script") {
            if let Some(end) = text[start..].find("</script>") {
                text = format!("{}{}", &text[..start], &text[start + end + 9..]);
            } else {
                break;
            }
        }

        while let Some(start) = text.find("<style") {
            if let Some(end) = text[start..].find("</style>") {
                text = format!("{}{}", &text[..start], &text[start + end + 8..]);
            } else {
                break;
            }
        }

        // Remove remaining tags
        let mut result = String::new();
        let mut in_tag = false;
        for c in text.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
                result.push(' ');
            } else if !in_tag {
                result.push(c);
            }
        }

        // Normalize whitespace
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Extract title
    pub fn get_title(&self) -> Option<String> {
        let start = self.html.find("<title>")?;
        let end = self.html[start..].find("</title>")?;
        Some(self.html[start + 7..start + end].to_string())
    }

    /// Extract all links
    pub fn get_links(&self) -> Result<JsValue, JsValue> {
        let mut links = Vec::new();
        let mut pos = 0;

        while let Some(start) = self.html[pos..].find("<a ") {
            let abs_start = pos + start;
            if let Some(end) = self.html[abs_start..].find(">") {
                let tag = &self.html[abs_start..abs_start + end + 1];

                // Extract href
                if let Some(href_start) = tag.find("href=\"") {
                    let href_begin = href_start + 6;
                    if let Some(href_end) = tag[href_begin..].find("\"") {
                        let href = &tag[href_begin..href_begin + href_end];

                        // Find closing tag for text
                        let close_pos = abs_start + end + 1;
                        if let Some(close_tag) = self.html[close_pos..].find("</a>") {
                            let text = &self.html[close_pos..close_pos + close_tag];
                            links.push(WasmLink {
                                url: href.to_string(),
                                text: text.trim().to_string(),
                            });
                        }
                    }
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        serde_wasm_bindgen::to_value(&links)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_id() -> String {
    // Simple ID generation using random
    let rand: u64 = (js_sys::Math::random() * 1_000_000_000_000.0) as u64;
    format!("{:016x}", rand)
}

fn current_timestamp() -> String {
    // Get current time from JS
    let date = js_sys::Date::new_0();
    date.to_iso_string().as_string().unwrap_or_default()
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format bytes to human-readable string
#[wasm_bindgen]
pub fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{:.0} B", bytes)
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.2} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Get library version
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if running in WASM
#[wasm_bindgen]
pub fn is_wasm() -> bool {
    true
}
