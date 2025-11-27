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
// DOM Utilities (using scraper/html5ever)
// ============================================================================

use scraper::{Html, Selector, ElementRef};

/// DOM element information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmElement {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub text_content: String,
    pub inner_html: String,
}

/// DOM parser and manipulator for WASM using html5ever
#[wasm_bindgen]
pub struct WasmDOM {
    html: String,
    document: Html,
}

#[wasm_bindgen]
impl WasmDOM {
    /// Create a new DOM instance from HTML (parsed with html5ever)
    #[wasm_bindgen(constructor)]
    pub fn new(html: String) -> WasmDOM {
        let document = Html::parse_document(&html);
        WasmDOM { html, document }
    }

    /// Parse an HTML fragment (not a full document)
    pub fn parse_fragment(html: String) -> WasmDOM {
        let document = Html::parse_fragment(&html);
        WasmDOM { html, document }
    }

    /// Get the raw HTML
    pub fn get_html(&self) -> String {
        self.html.clone()
    }

    /// Get the serialized HTML from the parsed document
    pub fn get_serialized_html(&self) -> String {
        self.document.html()
    }

    /// Extract text content from the document
    pub fn get_text(&self) -> String {
        // Get text from body, excluding script and style tags
        let body_selector = Selector::parse("body").ok();
        let script_selector = Selector::parse("script, style, noscript").ok();

        if let Some(body_sel) = body_selector {
            if let Some(body) = self.document.select(&body_sel).next() {
                return self.extract_text_without_scripts(body, script_selector.as_ref());
            }
        }

        // Fallback: get all text
        self.document.root_element().text().collect::<Vec<_>>().join(" ")
    }

    /// Extract text content from an element, excluding scripts and styles
    fn extract_text_without_scripts(&self, element: ElementRef, script_selector: Option<&Selector>) -> String {
        let mut text_parts = Vec::new();

        fn collect_text(element: ElementRef, text_parts: &mut Vec<String>, script_selector: Option<&Selector>) {
            // Skip script/style elements
            if let Some(sel) = script_selector {
                if element.value().name() == "script"
                    || element.value().name() == "style"
                    || element.value().name() == "noscript"
                {
                    return;
                }
            }

            for child in element.children() {
                if let Some(text) = child.value().as_text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                } else if let Some(elem) = scraper::ElementRef::wrap(child) {
                    collect_text(elem, text_parts, script_selector);
                }
            }
        }

        collect_text(element, &mut text_parts, script_selector);
        text_parts.join(" ")
    }

    /// Extract title
    pub fn get_title(&self) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        self.document.select(&selector).next()
            .map(|e| e.text().collect::<String>().trim().to_string())
    }

    /// Extract meta description
    pub fn get_meta_description(&self) -> Option<String> {
        let selector = Selector::parse("meta[name='description']").ok()?;
        self.document.select(&selector).next()
            .and_then(|e| e.value().attr("content").map(|s| s.to_string()))
    }

    /// Extract meta keywords
    pub fn get_meta_keywords(&self) -> Vec<String> {
        let selector = match Selector::parse("meta[name='keywords']") {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        self.document.select(&selector).next()
            .and_then(|e| e.value().attr("content"))
            .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Extract Open Graph metadata
    pub fn get_og_metadata(&self) -> Result<JsValue, JsValue> {
        let mut metadata = HashMap::new();

        let og_properties = ["og:title", "og:description", "og:image", "og:url", "og:type", "og:site_name"];

        for prop in og_properties {
            let selector_str = format!("meta[property='{}']", prop);
            let selector = match Selector::parse(&selector_str) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Some(element) = self.document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    metadata.insert(prop.to_string(), content.to_string());
                }
            }
        }

        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Query elements by CSS selector
    pub fn query_selector(&self, selector: &str) -> Result<JsValue, JsValue> {
        let selector = Selector::parse(selector)
            .map_err(|e| JsValue::from_str(&format!("Invalid selector: {:?}", e)))?;

        let element = self.document.select(&selector).next()
            .map(|e| self.element_to_wasm(e));

        serde_wasm_bindgen::to_value(&element)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Query all elements matching a CSS selector
    pub fn query_selector_all(&self, selector: &str) -> Result<JsValue, JsValue> {
        let selector = Selector::parse(selector)
            .map_err(|e| JsValue::from_str(&format!("Invalid selector: {:?}", e)))?;

        let elements: Vec<WasmElement> = self.document.select(&selector)
            .map(|e| self.element_to_wasm(e))
            .collect();

        serde_wasm_bindgen::to_value(&elements)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Convert a scraper ElementRef to WasmElement
    fn element_to_wasm(&self, element: ElementRef) -> WasmElement {
        let value = element.value();

        let attributes: HashMap<String, String> = value.attrs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        WasmElement {
            tag: value.name().to_string(),
            id: value.id().map(|s| s.to_string()),
            classes: value.classes().map(|s| s.to_string()).collect(),
            attributes,
            text_content: element.text().collect::<String>(),
            inner_html: element.inner_html(),
        }
    }

    /// Extract all links
    pub fn get_links(&self) -> Result<JsValue, JsValue> {
        let selector = Selector::parse("a[href]")
            .map_err(|e| JsValue::from_str(&format!("Selector error: {:?}", e)))?;

        let links: Vec<WasmLink> = self.document.select(&selector)
            .filter_map(|e| {
                let href = e.value().attr("href")?;
                let text = e.text().collect::<String>().trim().to_string();
                Some(WasmLink {
                    url: href.to_string(),
                    text,
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&links)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract all images
    pub fn get_images(&self) -> Result<JsValue, JsValue> {
        let selector = Selector::parse("img[src]")
            .map_err(|e| JsValue::from_str(&format!("Selector error: {:?}", e)))?;

        let images: Vec<WasmImage> = self.document.select(&selector)
            .filter_map(|e| {
                let src = e.value().attr("src")?;
                let alt = e.value().attr("alt").unwrap_or("").to_string();
                Some(WasmImage {
                    url: src.to_string(),
                    alt,
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&images)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract all form elements
    pub fn get_forms(&self) -> Result<JsValue, JsValue> {
        let form_selector = Selector::parse("form")
            .map_err(|e| JsValue::from_str(&format!("Selector error: {:?}", e)))?;

        let input_selector = Selector::parse("input, textarea, select, button").ok();

        let forms: Vec<HashMap<String, serde_json::Value>> = self.document.select(&form_selector)
            .map(|form| {
                let mut form_data = HashMap::new();

                form_data.insert("action".to_string(),
                    serde_json::Value::String(form.value().attr("action").unwrap_or("").to_string()));
                form_data.insert("method".to_string(),
                    serde_json::Value::String(form.value().attr("method").unwrap_or("GET").to_string()));
                form_data.insert("id".to_string(),
                    serde_json::Value::String(form.value().attr("id").unwrap_or("").to_string()));

                // Get form fields
                if let Some(ref input_sel) = input_selector {
                    let fields: Vec<HashMap<String, String>> = form.select(input_sel)
                        .map(|input| {
                            let mut field = HashMap::new();
                            field.insert("type".to_string(), input.value().attr("type").unwrap_or("text").to_string());
                            field.insert("name".to_string(), input.value().attr("name").unwrap_or("").to_string());
                            field.insert("id".to_string(), input.value().attr("id").unwrap_or("").to_string());
                            field.insert("value".to_string(), input.value().attr("value").unwrap_or("").to_string());
                            field
                        })
                        .collect();
                    form_data.insert("fields".to_string(), serde_json::to_value(fields).unwrap_or_default());
                }

                form_data
            })
            .collect();

        serde_wasm_bindgen::to_value(&forms)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract headings (h1-h6) with their hierarchy
    pub fn get_headings(&self) -> Result<JsValue, JsValue> {
        let selector = Selector::parse("h1, h2, h3, h4, h5, h6")
            .map_err(|e| JsValue::from_str(&format!("Selector error: {:?}", e)))?;

        let headings: Vec<HashMap<String, String>> = self.document.select(&selector)
            .map(|h| {
                let mut heading = HashMap::new();
                heading.insert("level".to_string(), h.value().name().to_string());
                heading.insert("text".to_string(), h.text().collect::<String>().trim().to_string());
                if let Some(id) = h.value().id() {
                    heading.insert("id".to_string(), id.to_string());
                }
                heading
            })
            .collect();

        serde_wasm_bindgen::to_value(&headings)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get element count by tag name
    pub fn count_elements(&self, tag: &str) -> usize {
        if let Ok(selector) = Selector::parse(tag) {
            self.document.select(&selector).count()
        } else {
            0
        }
    }

    /// Check if an element exists matching the selector
    pub fn has_element(&self, selector: &str) -> bool {
        if let Ok(sel) = Selector::parse(selector) {
            self.document.select(&sel).next().is_some()
        } else {
            false
        }
    }

    /// Get attribute value for first matching element
    pub fn get_attribute(&self, selector: &str, attribute: &str) -> Option<String> {
        Selector::parse(selector).ok()
            .and_then(|sel| self.document.select(&sel).next())
            .and_then(|e| e.value().attr(attribute).map(|s| s.to_string()))
    }

    /// Get all script sources
    pub fn get_script_sources(&self) -> Vec<String> {
        let selector = match Selector::parse("script[src]") {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        self.document.select(&selector)
            .filter_map(|e| e.value().attr("src").map(|s| s.to_string()))
            .collect()
    }

    /// Get all stylesheet links
    pub fn get_stylesheet_links(&self) -> Vec<String> {
        let selector = match Selector::parse("link[rel='stylesheet']") {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        self.document.select(&selector)
            .filter_map(|e| e.value().attr("href").map(|s| s.to_string()))
            .collect()
    }

    /// Get inline styles
    pub fn get_inline_styles(&self) -> Result<JsValue, JsValue> {
        let selector = Selector::parse("style")
            .map_err(|e| JsValue::from_str(&format!("Selector error: {:?}", e)))?;

        let styles: Vec<String> = self.document.select(&selector)
            .map(|e| e.text().collect::<String>())
            .collect();

        serde_wasm_bindgen::to_value(&styles)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

// ============================================================================
// Readability API (Content Extraction)
// ============================================================================

use crate::features::readability::{
    ReadabilityEngine, ReadabilityConfig, QualityMetrics as ReadabilityQuality,
    ExtractionResult, OutputFormat,
};

/// Readability extraction result for WASM
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmReadabilityResult {
    /// Extracted clean content
    pub content: String,
    /// Output format used
    pub format: String,
    /// Title extracted from the page
    pub title: String,
    /// Author if found
    pub author: Option<String>,
    /// Publication date if found
    pub published_date: Option<String>,
    /// Main image URL if found
    pub main_image: Option<String>,
    /// Word count
    pub word_count: u32,
    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,
    /// Readability score (0-100)
    pub readability_score: u32,
    /// Whether extraction was successful
    pub success: bool,
    /// Error message if extraction failed
    pub error: Option<String>,
}

/// Readability configuration for WASM
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmReadabilityConfig {
    /// Minimum content score threshold (0.0 - 1.0)
    pub min_content_score: f32,
    /// Maximum link density allowed (0.0 - 1.0)
    pub max_link_density: f32,
    /// Minimum paragraph count for valid content
    pub min_paragraph_count: u32,
    /// Include images in extracted content
    pub include_images: bool,
    /// Include metadata (author, date, etc.)
    pub include_metadata: bool,
    /// Output format: "markdown", "text", or "html"
    pub output_format: String,
}

impl Default for WasmReadabilityConfig {
    fn default() -> Self {
        Self {
            min_content_score: 0.3,
            max_link_density: 0.25,
            min_paragraph_count: 3,
            include_images: true,
            include_metadata: true,
            output_format: "markdown".to_string(),
        }
    }
}

impl From<WasmReadabilityConfig> for ReadabilityConfig {
    fn from(config: WasmReadabilityConfig) -> Self {
        ReadabilityConfig {
            min_content_score: config.min_content_score,
            max_link_density: config.max_link_density,
            min_paragraph_count: config.min_paragraph_count,
            include_images: config.include_images,
            include_metadata: config.include_metadata,
            output_format: match config.output_format.as_str() {
                "text" => OutputFormat::Text,
                "structured" => OutputFormat::Structured,
                _ => OutputFormat::Markdown,
            },
        }
    }
}

/// Readability content extractor for WASM
///
/// Extracts clean, readable content from web pages using sophisticated
/// content detection algorithms similar to Chrome's reading mode.
#[wasm_bindgen]
pub struct WasmReadability {
    engine: ReadabilityEngine,
}

#[wasm_bindgen]
impl WasmReadability {
    /// Create a new readability extractor with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmReadability {
        WasmReadability {
            engine: ReadabilityEngine::new(),
        }
    }

    /// Create a readability extractor with custom configuration
    pub fn with_config(config: JsValue) -> Result<WasmReadability, JsValue> {
        let wasm_config: WasmReadabilityConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        Ok(WasmReadability {
            engine: ReadabilityEngine::with_config(wasm_config.into()),
        })
    }

    /// Extract readable content from HTML
    ///
    /// Returns clean, formatted content suitable for reading or AI processing.
    pub fn extract(&mut self, html: &str, url: &str) -> Result<JsValue, JsValue> {
        let result = self.engine.extract(html, url)
            .map_err(|e| JsValue::from_str(&format!("Extraction error: {}", e)))?;

        let wasm_result = WasmReadabilityResult {
            content: result.content.content,
            format: format!("{:?}", result.content.format),
            title: result.content.metadata.title.unwrap_or_default(),
            author: result.content.metadata.author,
            published_date: result.content.metadata.publication_date,
            main_image: result.content.metadata.main_image,
            word_count: result.quality.word_count,
            reading_time_minutes: result.quality.reading_time_minutes,
            readability_score: result.quality.readability_score,
            success: result.success,
            error: result.error,
        };

        serde_wasm_bindgen::to_value(&wasm_result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Quick extraction with default settings
    pub fn extract_quick(&mut self, html: &str, url: &str) -> Result<JsValue, JsValue> {
        self.extract(html, url)
    }

    /// Extract only text content without formatting
    pub fn extract_text_only(&mut self, html: &str, url: &str) -> Result<String, JsValue> {
        // Create a text-only config
        let config = ReadabilityConfig {
            min_content_score: 0.2,
            max_link_density: 0.5,
            min_paragraph_count: 1,
            include_images: false,
            include_metadata: false,
            output_format: OutputFormat::Text,
        };

        let mut engine = ReadabilityEngine::with_config(config);
        let result = engine.extract(html, url)
            .map_err(|e| JsValue::from_str(&format!("Extraction error: {}", e)))?;

        if result.success {
            Ok(result.content.content)
        } else {
            Ok(String::new())
        }
    }

    /// Check if a page is likely to have readable content
    pub fn is_readable(&mut self, html: &str, url: &str) -> bool {
        match self.engine.extract(html, url) {
            Ok(result) => result.success && result.quality.readability_score > 30,
            Err(_) => false,
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> Result<JsValue, JsValue> {
        let config = self.engine.config();
        let wasm_config = WasmReadabilityConfig {
            min_content_score: config.min_content_score,
            max_link_density: config.max_link_density,
            min_paragraph_count: config.min_paragraph_count,
            include_images: config.include_images,
            include_metadata: config.include_metadata,
            output_format: format!("{:?}", config.output_format).to_lowercase(),
        };

        serde_wasm_bindgen::to_value(&wasm_config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Update configuration
    pub fn set_config(&mut self, config: JsValue) -> Result<(), JsValue> {
        let wasm_config: WasmReadabilityConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        self.engine.set_config(wasm_config.into());
        Ok(())
    }

    /// Set output format ("markdown", "text", or "structured")
    pub fn set_output_format(&mut self, format: &str) {
        let mut config = self.engine.config().clone();
        config.output_format = match format {
            "text" => OutputFormat::Text,
            "structured" => OutputFormat::Structured,
            _ => OutputFormat::Markdown,
        };
        self.engine.set_config(config);
    }
}

impl Default for WasmReadability {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CSS Processor API
// ============================================================================

use crate::engine::renderer::css::{CssProcessor, ComputedStyles, ParsedRule, BoxModel};

/// CSS processor for WASM
///
/// Provides CSS parsing, style computation, and minification using lightningcss.
#[wasm_bindgen]
pub struct WasmCssProcessor {
    processor: CssProcessor,
}

#[wasm_bindgen]
impl WasmCssProcessor {
    /// Create a new CSS processor
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCssProcessor {
        WasmCssProcessor {
            processor: CssProcessor::new(),
        }
    }

    /// Parse CSS and add its rules to the processor
    pub fn parse(&mut self, css: &str) -> Result<(), JsValue> {
        self.processor.parse(css)
            .map_err(|e| JsValue::from_str(&format!("CSS parse error: {}", e)))
    }

    /// Compute styles for a given selector
    pub fn compute_style(&self, selector: &str) -> Result<JsValue, JsValue> {
        let styles = self.processor.compute_style(selector);
        serde_wasm_bindgen::to_value(&styles)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get a specific CSS property value
    pub fn get_property(&self, selector: &str, property: &str) -> Option<String> {
        self.processor.get_property(selector, property)
    }

    /// Minify CSS
    pub fn minify(&self, css: &str) -> Result<String, JsValue> {
        self.processor.minify(css)
            .map_err(|e| JsValue::from_str(&format!("Minify error: {}", e)))
    }

    /// Process CSS (returns processed CSS)
    pub fn process(&self, css: &str) -> Result<String, JsValue> {
        self.processor.process_css(css)
            .map_err(|e| JsValue::from_str(&format!("Process error: {}", e)))
    }

    /// Get all parsed rules
    pub fn get_rules(&self) -> Result<JsValue, JsValue> {
        let rules = self.processor.get_rules();
        serde_wasm_bindgen::to_value(rules)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Clear all parsed rules
    pub fn clear(&mut self) {
        self.processor.clear();
    }
}

impl Default for WasmCssProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Layout Engine API
// ============================================================================

use crate::engine::renderer::layout::{
    LayoutEngine, LayoutResult, LayoutElement, ElementLayout, ContentBox, LayoutNodeData,
};

/// Layout element for WASM input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmLayoutElement {
    pub id: String,
    pub tag: String,
    pub styles: HashMap<String, Option<String>>,
    pub children: Vec<WasmLayoutElement>,
}

impl From<WasmLayoutElement> for LayoutElement {
    fn from(elem: WasmLayoutElement) -> Self {
        LayoutElement {
            id: elem.id,
            tag: elem.tag,
            styles: ComputedStyles {
                display: elem.styles.get("display").and_then(|s| s.clone()),
                position: elem.styles.get("position").and_then(|s| s.clone()),
                width: elem.styles.get("width").and_then(|s| s.clone()),
                height: elem.styles.get("height").and_then(|s| s.clone()),
                flex_direction: elem.styles.get("flex-direction").and_then(|s| s.clone()),
                justify_content: elem.styles.get("justify-content").and_then(|s| s.clone()),
                align_items: elem.styles.get("align-items").and_then(|s| s.clone()),
                gap: elem.styles.get("gap").and_then(|s| s.clone()),
                margin: elem.styles.get("margin").and_then(|s| s.as_ref().map(|_| BoxModel::default())),
                padding: elem.styles.get("padding").and_then(|s| s.as_ref().map(|_| BoxModel::default())),
                ..Default::default()
            },
            children: elem.children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

/// Layout engine for WASM
///
/// Provides CSS-compliant layout computation using taffy.
#[wasm_bindgen]
pub struct WasmLayoutEngine {
    engine: LayoutEngine,
}

#[wasm_bindgen]
impl WasmLayoutEngine {
    /// Create a new layout engine with default viewport
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmLayoutEngine {
        WasmLayoutEngine {
            engine: LayoutEngine::new(),
        }
    }

    /// Create a layout engine with specific viewport dimensions
    pub fn with_viewport(width: f32, height: f32) -> WasmLayoutEngine {
        WasmLayoutEngine {
            engine: LayoutEngine::with_viewport(width, height),
        }
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.engine.set_viewport(width, height);
    }

    /// Calculate layout for an element tree
    pub fn calculate_layout(&mut self, root: JsValue) -> Result<JsValue, JsValue> {
        let wasm_root: WasmLayoutElement = serde_wasm_bindgen::from_value(root)
            .map_err(|e| JsValue::from_str(&format!("Invalid element tree: {}", e)))?;

        let layout_root: LayoutElement = wasm_root.into();

        let result = self.engine.calculate_layout_from_elements(&layout_root)
            .map_err(|e| JsValue::from_str(&format!("Layout calculation error: {}", e)))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

impl Default for WasmLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// JavaScript Engine API
// ============================================================================

use crate::engine::engine_wasm::JavaScriptEngine;
use wasm_bindgen_futures::future_to_promise;

/// JavaScript engine for WASM
///
/// Provides sandboxed JavaScript execution using Boa engine with full
/// Web API support (fetch, setTimeout, Promise, etc.)
#[wasm_bindgen]
pub struct WasmJsEngine {
    engine: Option<JavaScriptEngine>,
}

#[wasm_bindgen]
impl WasmJsEngine {
    /// Create a new JavaScript engine instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmJsEngine, JsValue> {
        let engine = JavaScriptEngine::new()
            .map_err(|e| JsValue::from_str(&format!("Failed to create JS engine: {}", e)))?;

        Ok(WasmJsEngine {
            engine: Some(engine),
        })
    }

    /// Execute JavaScript code and return the result
    ///
    /// Supports ES2025+ syntax including async/await, optional chaining,
    /// nullish coalescing, and more.
    pub async fn execute(&mut self, code: String) -> Result<JsValue, JsValue> {
        let engine = self.engine.as_mut()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;

        let result = engine.execute_enhanced(&code).await
            .map_err(|e| JsValue::from_str(&format!("Execution error: {}", e)))?;

        // Convert Boa JsValue to wasm_bindgen JsValue
        Self::convert_to_js_value_static(&result)
    }

    /// Execute JavaScript code synchronously (for simpler scripts)
    /// Note: In WASM, this uses futures::executor::block_on which may have limitations
    pub fn execute_sync(&mut self, code: &str) -> Result<JsValue, JsValue> {
        let engine = self.engine.as_mut()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;

        // Use futures::executor::block_on for WASM compatibility
        let result = futures::executor::block_on(async {
            engine.execute_enhanced(code).await
        }).map_err(|e| JsValue::from_str(&format!("Execution error: {}", e)))?;

        Self::convert_to_js_value_static(&result)
    }

    /// Set a global variable in the JavaScript context
    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        // First convert the value before borrowing engine mutably
        let boa_value = Self::convert_from_js_value_static(&value)?;

        let engine = self.engine.as_mut()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;

        engine.set_global_object(name, boa_value)
            .map_err(|e| JsValue::from_str(&format!("Failed to set global: {}", e)))
    }

    /// Get a global variable from the JavaScript context
    pub fn get_global(&mut self, name: &str) -> Result<JsValue, JsValue> {
        let engine = self.engine.as_mut()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;

        let result = engine.get_global_object(name)
            .map_err(|e| JsValue::from_str(&format!("Failed to get global: {}", e)))?;

        match result {
            Some(value) => Self::convert_to_js_value_static(&value),
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// Run pending microtasks (promises, etc.)
    pub fn run_jobs(&mut self) -> Result<(), JsValue> {
        let engine = self.engine.as_mut()
            .ok_or_else(|| JsValue::from_str("Engine not initialized"))?;

        engine.run_jobs()
            .map_err(|e| JsValue::from_str(&format!("Failed to run jobs: {}", e)))
    }

    /// Get engine version information
    pub fn version(&self) -> String {
        match &self.engine {
            Some(engine) => engine.version_info(),
            None => "Engine not initialized".to_string(),
        }
    }

    /// Convert Boa JsValue to wasm_bindgen JsValue (static method to avoid borrow issues)
    fn convert_to_js_value_static(boa_value: &thalora_browser_apis::boa_engine::JsValue) -> Result<JsValue, JsValue> {
        // Simple conversion - in practice we'd need more sophisticated handling
        if boa_value.is_undefined() {
            Ok(JsValue::UNDEFINED)
        } else if boa_value.is_null() {
            Ok(JsValue::NULL)
        } else if let Some(b) = boa_value.as_boolean() {
            Ok(JsValue::from_bool(b))
        } else if let Some(n) = boa_value.as_number() {
            Ok(JsValue::from_f64(n))
        } else if let Some(s) = boa_value.as_string() {
            Ok(JsValue::from_str(&s.to_std_string_escaped()))
        } else {
            // For complex objects, serialize to JSON
            Ok(JsValue::from_str(&format!("{:?}", boa_value)))
        }
    }

    /// Convert wasm_bindgen JsValue to Boa JsValue (static method to avoid borrow issues)
    fn convert_from_js_value_static(js_value: &JsValue) -> Result<thalora_browser_apis::boa_engine::JsValue, JsValue> {
        use thalora_browser_apis::boa_engine::JsValue as BoaValue;

        if js_value.is_undefined() {
            Ok(BoaValue::undefined())
        } else if js_value.is_null() {
            Ok(BoaValue::null())
        } else if let Some(b) = js_value.as_bool() {
            Ok(BoaValue::from(b))
        } else if let Some(n) = js_value.as_f64() {
            Ok(BoaValue::from(n))
        } else if let Some(s) = js_value.as_string() {
            Ok(BoaValue::from(thalora_browser_apis::boa_engine::js_string!(s)))
        } else {
            // Default to undefined for complex objects
            Ok(BoaValue::undefined())
        }
    }
}

impl Default for WasmJsEngine {
    fn default() -> Self {
        Self::new().unwrap_or(WasmJsEngine { engine: None })
    }
}

// ============================================================================
// Page Processor API (Unified Pipeline)
// ============================================================================

/// Page processing result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmPageResult {
    /// The page URL
    pub url: String,
    /// Page title
    pub title: Option<String>,
    /// Clean readable content (markdown)
    pub readable_content: String,
    /// Plain text content
    pub text_content: String,
    /// All links on the page
    pub links: Vec<WasmLink>,
    /// All images on the page
    pub images: Vec<WasmImage>,
    /// Metadata (OpenGraph, etc.)
    pub metadata: HashMap<String, String>,
    /// Whether readability extraction succeeded
    pub is_readable: bool,
    /// Readability score (0-100)
    pub readability_score: u32,
    /// Word count
    pub word_count: u32,
    /// Processing time in ms
    pub processing_time_ms: u32,
}

/// Unified page processor for WASM
///
/// Combines DOM parsing, CSS processing, readability extraction, and
/// JavaScript execution into a single processing pipeline.
#[wasm_bindgen]
pub struct WasmPageProcessor {
    readability: WasmReadability,
}

#[wasm_bindgen]
impl WasmPageProcessor {
    /// Create a new page processor
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPageProcessor {
        WasmPageProcessor {
            readability: WasmReadability::new(),
        }
    }

    /// Process a page and extract all relevant data
    pub fn process(&mut self, html: &str, url: &str) -> Result<JsValue, JsValue> {
        let start = js_sys::Date::now();

        // Parse DOM
        let dom = WasmDOM::new(html.to_string());

        // Extract basic info
        let title = dom.get_title();
        let text_content = dom.get_text();

        // Extract links and images
        let links: Vec<WasmLink> = serde_wasm_bindgen::from_value(dom.get_links()?)
            .unwrap_or_default();
        let images: Vec<WasmImage> = serde_wasm_bindgen::from_value(dom.get_images()?)
            .unwrap_or_default();

        // Extract metadata
        let metadata: HashMap<String, String> = serde_wasm_bindgen::from_value(dom.get_og_metadata()?)
            .unwrap_or_default();

        // Extract readable content
        let readability_result = self.readability.extract(html, url);
        let (readable_content, is_readable, readability_score, word_count) = match readability_result {
            Ok(result) => {
                let result: WasmReadabilityResult = serde_wasm_bindgen::from_value(result)
                    .unwrap_or(WasmReadabilityResult {
                        content: String::new(),
                        format: "text".to_string(),
                        title: String::new(),
                        author: None,
                        published_date: None,
                        main_image: None,
                        word_count: 0,
                        reading_time_minutes: 0,
                        readability_score: 0,
                        success: false,
                        error: None,
                    });
                (result.content, result.success, result.readability_score, result.word_count)
            },
            Err(_) => (String::new(), false, 0, 0),
        };

        let processing_time_ms = (js_sys::Date::now() - start) as u32;

        let result = WasmPageResult {
            url: url.to_string(),
            title,
            readable_content,
            text_content,
            links,
            images,
            metadata,
            is_readable,
            readability_score,
            word_count,
            processing_time_ms,
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Quick process - just extract text and links
    pub fn process_quick(&mut self, html: &str, url: &str) -> Result<JsValue, JsValue> {
        let dom = WasmDOM::new(html.to_string());

        let result = WasmPageResult {
            url: url.to_string(),
            title: dom.get_title(),
            readable_content: String::new(),
            text_content: dom.get_text(),
            links: serde_wasm_bindgen::from_value(dom.get_links()?).unwrap_or_default(),
            images: vec![],
            metadata: HashMap::new(),
            is_readable: false,
            readability_score: 0,
            word_count: 0,
            processing_time_ms: 0,
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Extract only readable content
    pub fn extract_readable(&mut self, html: &str, url: &str) -> Result<JsValue, JsValue> {
        self.readability.extract(html, url)
    }

    /// Check if a page is likely readable
    pub fn is_readable(&mut self, html: &str, url: &str) -> bool {
        self.readability.is_readable(html, url)
    }
}

impl Default for WasmPageProcessor {
    fn default() -> Self {
        Self::new()
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
