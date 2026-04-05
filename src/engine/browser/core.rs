use crate::engine::browser::scraper::WebScraper;
use crate::engine::browser::types::{
    AuthContext, BrowserStorage, HistoryEntry, HistoryEvent, NavigationHistory, NavigationMode,
    ResourceCache,
};
use crate::engine::browser::{FormAnalyzer, FormInfo};
use crate::engine::renderer::RustRenderer;
use anyhow::Result;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, UPGRADE_INSECURE_REQUESTS,
    USER_AGENT,
};
use reqwest_cookie_store::CookieStoreMutex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[allow(dead_code)]
pub struct HeadlessWebBrowser {
    pub(super) client: reqwest::Client,
    pub(super) cookie_store: Arc<CookieStoreMutex>,
    pub(super) renderer: Option<RustRenderer>,
    pub(super) current_url: Option<String>,
    pub(super) current_content: String,
    pub(super) auth_context: AuthContext,
    pub(super) storage: BrowserStorage,
    pub(super) history: NavigationHistory,
    pub(super) scraper: WebScraper,
    pub(super) form_analyzer: FormAnalyzer,
    pub(super) analyzed_forms: Vec<FormInfo>,
    /// External stylesheets fetched from <link rel="stylesheet"> tags
    pub(super) external_stylesheets: Vec<String>,
    /// Controls whether artificial anti-bot delays are applied during navigation
    pub(super) navigation_mode: NavigationMode,
    /// In-memory cache for fetched stylesheets and scripts
    pub(super) resource_cache: ResourceCache,
    /// When true, bypass the resource cache (used during reload)
    pub(super) bypass_cache: bool,
    /// Queue of history events from JS History API for GUI synchronization
    pub(super) history_events: Arc<Mutex<Vec<HistoryEvent>>>,
    /// Parsed Content-Security-Policy for the current page
    pub(super) csp_policy: Option<super::navigation::csp::CspPolicy>,
    /// Parsed Permissions-Policy for the current page
    pub(super) permissions_policy: Option<super::navigation::csp::PermissionsPolicy>,
    /// HSTS store for upgrading HTTP to HTTPS
    pub(super) hsts_store: super::navigation::hsts::HstsStore,
}

impl HeadlessWebBrowser {
    pub fn new() -> Arc<Mutex<Self>> {
        Self::new_with_engine(crate::engine::engine_trait::EngineType::Boa)
    }

    pub fn new_with_engine(
        engine_type: crate::engine::engine_trait::EngineType,
    ) -> Arc<Mutex<Self>> {
        // Create shared cookie store for cookie management
        let cookie_store = Arc::new(CookieStoreMutex::new(
            reqwest_cookie_store::CookieStore::default(),
        ));

        // Configure client with enhanced stealth capabilities
        // Use centralized USER_AGENT constant for consistency
        let client = reqwest::Client::builder()
            .cookie_provider(Arc::clone(&cookie_store))
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(super::USER_AGENT)
            .gzip(true)
            .brotli(true)
            .deflate(true)
            // Let reqwest negotiate HTTP/2 via ALPN naturally - http2_prior_knowledge() can cause connection failures
            .http2_adaptive_window(true)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .expect("Failed to create HTTP client");

        let renderer = RustRenderer::new_with_engine(engine_type);

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
        let form_analyzer = FormAnalyzer::new();

        let browser = Self {
            client,
            cookie_store,
            renderer: Some(renderer),
            current_url: None,
            current_content: String::new(),
            auth_context,
            storage,
            history,
            scraper,
            form_analyzer,
            analyzed_forms: Vec::new(),
            external_stylesheets: Vec::new(),
            navigation_mode: NavigationMode::default(),
            resource_cache: ResourceCache::default(),
            bypass_cache: false,
            history_events: Arc::new(Mutex::new(Vec::new())),
            csp_policy: None,
            permissions_policy: None,
            hsts_store: super::navigation::hsts::HstsStore::new(),
        };

        let browser_arc = Arc::new(Mutex::new(browser));

        // Setup history API with reference to browser
        let _ = Self::setup_history_api(browser_arc.clone());

        browser_arc
    }

    pub fn setup_history_api(browser_arc: Arc<Mutex<Self>>) -> Result<()> {
        if let Ok(mut browser) = browser_arc.try_lock() {
            // Extract the events handle while we hold the lock, then pass it
            // directly to the renderer — avoids deadlock from re-locking browser_arc.
            let events_handle = browser.history_events.clone();
            if let Some(ref mut renderer) = browser.renderer {
                renderer.setup_history_api(events_handle)?;
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
        (
            self.storage.local_storage.clone(),
            self.storage.session_storage.clone(),
        )
    }

    pub fn get_chrome_headers(&self, url: &str) -> reqwest::header::HeaderMap {
        self.create_standard_browser_headers(url)
    }

    pub fn get_storage_mut(&mut self) -> &mut BrowserStorage {
        &mut self.storage
    }

    /// Get analyzed forms from the current page
    pub fn get_analyzed_forms(&self) -> &[FormInfo] {
        &self.analyzed_forms
    }

    /// Get forms that open new windows
    pub fn get_new_window_forms(&self) -> Vec<&FormInfo> {
        self.analyzed_forms
            .iter()
            .filter(|f| f.opens_new_window)
            .collect()
    }

    /// Get external stylesheets fetched from <link rel="stylesheet"> tags
    pub fn get_external_stylesheets(&self) -> &[String] {
        &self.external_stylesheets
    }

    /// Set the navigation mode (Interactive for GUI, Stealth for MCP/headless)
    pub fn set_navigation_mode(&mut self, mode: NavigationMode) {
        self.navigation_mode = mode;
    }

    /// Get the current navigation mode
    pub fn get_navigation_mode(&self) -> NavigationMode {
        self.navigation_mode
    }

    /// Drain all pending history events from the queue.
    pub fn drain_history_events(&self) -> Vec<HistoryEvent> {
        if let Ok(mut events) = self.history_events.lock() {
            events.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Push a history event into the queue (called from JS engine callback).
    pub fn push_history_event(&self, event: HistoryEvent) {
        if let Ok(mut events) = self.history_events.lock() {
            events.push(event);
        }
    }

    /// Get a clone of the history events Arc for sharing with callbacks.
    pub fn history_events_handle(&self) -> Arc<Mutex<Vec<HistoryEvent>>> {
        self.history_events.clone()
    }

    /// Find form information by submit button selector
    pub fn find_form_by_submit_button(&self, button_selector: &str) -> Option<FormInfo> {
        self.form_analyzer
            .find_form_by_submit_button(&self.current_content, button_selector)
            .ok()
            .flatten()
    }

    pub(super) fn add_to_history(&mut self, url: String, title: String) {
        let entry = HistoryEntry {
            url,
            title,
            timestamp: Instant::now(),
        };

        // Remove any entries after current index (when navigating back then to a new page)
        self.history
            .entries
            .truncate(self.history.current_index + 1);
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

    /// Execute JavaScript from page-loaded `<script>` tags (trusted context).
    /// Uses relaxed security that allows eval, Function, document.write, WebAssembly.
    pub async fn execute_page_javascript(&mut self, js_code: &str) -> Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.evaluate_page_javascript(js_code)
        } else {
            Err(anyhow::anyhow!("Renderer not available"))
        }
    }

    /// Execute JavaScript source as an ES module (trusted page context).
    pub async fn execute_module(&mut self, source: &str, url: &str) -> Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.evaluate_module(source, url)
        } else {
            Err(anyhow::anyhow!("Renderer not available"))
        }
    }

    /// Update the HTTP module loader's base URL for relative import resolution.
    pub fn set_module_base_url(&mut self, url: &str) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_module_base_url(url);
        }
    }

    pub fn create_standard_browser_headers(&self, url: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Use shared Chrome USER_AGENT constant - single source of truth!
        headers.insert(USER_AGENT, HeaderValue::from_static(super::USER_AGENT));

        // Chrome-style Accept header
        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
        ));

        // Chrome language preferences
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        // Chrome compression support
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );

        // Chrome sends client hints - add them for Chrome fingerprint consistency
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static("\"Chromium\";v=\"120\", \"Not A(Brand\";v=\"99\""),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static("\"Windows\""),
        );

        // Proper fetch metadata for Chrome
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

        // Add DNT header that Firefox sends
        headers.insert("dnt", HeaderValue::from_static("1"));

        // Add TE header for Firefox (important fingerprint!)
        headers.insert("te", HeaderValue::from_static("trailers"));

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

    /// Dispatch pageswap event before navigation
    pub async fn dispatch_pageswap_event(&mut self, target_url: &str) -> anyhow::Result<()> {
        if let Some(ref mut renderer) = self.renderer {
            // Create NavigationActivation data
            let current_url = self
                .current_url
                .clone()
                .unwrap_or_else(|| "about:blank".to_string());

            // Create navigation activation object with current and target URLs
            let activation_script = format!(
                r#"
                (function() {{
                    try {{
                        return {{
                            entry: {{ url: "{}" }},
                            from: {{ url: "{}" }},
                            navigationType: "navigate"
                        }};
                    }} catch(e) {{
                        console.log("Failed to create activation:", e.message);
                        return null;
                    }}
                }})()
            "#,
                target_url, current_url
            );

            let activation_value = match renderer.eval_js(&activation_script) {
                Ok(val) => Some(val),
                Err(e) => {
                    eprintln!("🔍 DEBUG: Failed to create navigation activation: {:?}", e);
                    None
                }
            };

            // Dispatch pageswap event on window
            let dispatch_script = r#"
                (function() {
                    try {
                        if (typeof window !== 'undefined' && typeof window.dispatchEvent === 'function') {
                            // Create pageswap event
                            var event = new PageSwapEvent('pageswap', {
                                bubbles: false,
                                cancelable: false,
                                activation: arguments[0] || null,
                                viewTransition: null
                            });

                            // Dispatch event on window
                            window.dispatchEvent(event);
                            return 'success';
                        } else {
                            return 'window.dispatchEvent not available';
                        }
                    } catch(e) {
                        return 'error: ' + e.message;
                    }
                })
            "#;

            match renderer.eval_js(dispatch_script) {
                Ok(result) => {
                    let result_str = renderer.js_value_to_string(result);
                    eprintln!("🔍 DEBUG: PageSwap event dispatch result: {}", result_str);

                    // Call the dispatch function with activation data
                    if let Some(activation) = activation_value {
                        let call_script = format!(
                            "({}).call(null, {})",
                            dispatch_script,
                            renderer.js_value_to_string(activation)
                        );
                        if let Ok(call_res) = renderer.eval_js(&call_script) {
                            eprintln!(
                                "🔍 DEBUG: PageSwap event with activation result: {}",
                                renderer.js_value_to_string(call_res)
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("🔍 DEBUG: Failed to dispatch pageswap event: {:?}", e);
                }
            }
        }
        Ok(())
    }
}

impl Drop for HeadlessWebBrowser {
    fn drop(&mut self) {
        // Log cleanup for debugging
        if let Some(url) = &self.current_url {
            eprintln!("🧹 Cleaning up browser for URL: {}", url);
        } else {
            eprintln!("🧹 Cleaning up unused browser instance");
        }

        // Shutdown JavaScript renderer explicitly
        if let Some(renderer) = self.renderer.take() {
            drop(renderer);
        }

        // Clear storage
        self.storage.local_storage.clear();
        self.storage.session_storage.clear();

        // Clear history
        self.history.entries.clear();

        // Clear auth context
        self.auth_context.cookies.clear();
        self.auth_context.auth_headers.clear();
        self.auth_context.csrf_tokens.clear();

        // Clear content
        self.current_content.clear();
        self.current_url = None;
        self.external_stylesheets.clear();
        self.resource_cache.clear();

        // Note: reqwest::Client will be dropped automatically
        // It will close connection pools when the last reference is dropped
    }
}
