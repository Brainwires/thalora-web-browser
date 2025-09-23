use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, UPGRADE_INSECURE_REQUESTS};
use crate::engine::renderer::RustRenderer;
use crate::engine::browser::types::{AuthContext, BrowserStorage, NavigationHistory, HistoryEntry};
use crate::engine::browser::scraper::WebScraper;

#[allow(dead_code)]
pub struct HeadlessWebBrowser {
    pub(super) client: reqwest::Client,
    pub(super) renderer: Option<RustRenderer>,
    pub(super) current_url: Option<String>,
    pub(super) current_content: String,
    pub(super) auth_context: AuthContext,
    pub(super) storage: BrowserStorage,
    pub(super) history: NavigationHistory,
    pub(super) scraper: WebScraper,
}

impl HeadlessWebBrowser {
    pub fn new() -> Arc<Mutex<Self>> {
        // Configure client with enhanced stealth capabilities
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .http2_prior_knowledge()
            .http2_adaptive_window(true)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .expect("Failed to create HTTP client");

    let renderer = RustRenderer::new();

        let auth_context = AuthContext {
            cookies: HashMap::new(),
            auth_headers: HashMap::new(),
            csrf_tokens: HashMap::new(),
        };

        let storage = BrowserStorage::default();

        let history = NavigationHistory {
            entries: Vec::new(),
            current_index: 0,
        };

        let scraper = WebScraper::new();

        let browser = Self {
            client,
            renderer: Some(renderer),
            current_url: None,
            current_content: String::new(),
            auth_context,
            storage,
            history,
            scraper,
        };

        let browser_arc = Arc::new(Mutex::new(browser));

        // Setup history API with reference to browser
    let _ = Self::setup_history_api(browser_arc.clone());

        browser_arc
    }

    pub fn setup_history_api(browser_arc: Arc<Mutex<Self>>) -> Result<()> {
        if let Ok(mut browser) = browser_arc.try_lock() {
            if let Some(ref mut renderer) = browser.renderer {
                renderer.setup_history_api(browser_arc.clone())?;
            }
        }
        Ok(())
    }

    pub fn get_current_url(&self) -> Option<String> {
        self.current_url.clone()
    }

    pub fn get_current_content(&self) -> String {
        self.current_content.clone()
    }

    pub fn get_history(&self) -> &NavigationHistory {
        &self.history
    }

    pub fn get_storage_data(&self) -> (HashMap<String, String>, HashMap<String, String>) {
        (self.storage.local_storage.clone(), self.storage.session_storage.clone())
    }

    pub fn get_chrome_headers(&self, url: &str) -> reqwest::header::HeaderMap {
        self.create_standard_browser_headers(url)
    }

    pub fn get_storage_mut(&mut self) -> &mut BrowserStorage {
        &mut self.storage
    }

    pub(super) fn add_to_history(&mut self, url: String, title: String) {
        let entry = HistoryEntry {
            url,
            title,
            timestamp: Instant::now(),
        };

        // Remove any entries after current index (when navigating back then to a new page)
        self.history.entries.truncate(self.history.current_index + 1);
        self.history.entries.push(entry);
        self.history.current_index = self.history.entries.len() - 1;
    }

    pub fn can_go_back(&self) -> bool {
        self.history.current_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history.current_index < self.history.entries.len().saturating_sub(1)
    }

    /// Execute JavaScript in the internal renderer and return the raw string result.
    /// Tests call this on a MutexGuard (so &mut self) and await the future.
    pub async fn execute_javascript(&mut self, js_code: &str) -> Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            // Delegate to renderer's evaluate_javascript which already handles timeouts and safety
            renderer.evaluate_javascript(js_code)
        } else {
            Err(anyhow::anyhow!("Renderer not available"))
        }
    }

    pub fn create_standard_browser_headers(&self, url: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Use Firefox User-Agent - Firefox is less likely to be blocked by Google
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0"
        ));

        // Firefox-style Accept header
        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
        ));

        // More realistic language preferences with Firefox ordering
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));

        // Firefox compression support (no zstd)
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));

        // Firefox doesn't send Chrome's client hints - this is important!
        // Google detects mismatched User-Agent vs client hints

        // Proper fetch metadata for Firefox
        if url.starts_with("https://www.google.com") {
            headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        } else if url.starts_with("https://www.bing.com") {
            headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        } else {
            headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
            headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
        }

        headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

        // Firefox doesn't send cache-control on initial requests
        // headers.insert("cache-control", HeaderValue::from_static("max-age=0"));

        // Add DNT header that Firefox sends
        headers.insert("dnt", HeaderValue::from_static("1"));

        // Firefox doesn't send priority header
        // headers.insert("priority", HeaderValue::from_static("u=0, i"));

        // Add some realistic Firefox headers
        headers.insert("connection", HeaderValue::from_static("keep-alive"));

        headers
    }

    /// TEMPORARY: Get debugging information about missing APIs for Bing search
    pub fn get_bing_debug_info(&mut self) -> anyhow::Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.get_bing_debug_info()
        } else {
            Ok("No renderer available".to_string())
        }
    }
}