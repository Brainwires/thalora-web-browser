use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use rquest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, UPGRADE_INSECURE_REQUESTS};
use rquest_util::Emulation;
use crate::engine::renderer::RustRenderer;
use crate::engine::browser::types::{AuthContext, BrowserStorage, NavigationHistory, HistoryEntry};
use crate::engine::browser::scraper::WebScraper;
use crate::engine::browser::{FormAnalyzer, FormInfo};

/// A cookie store wrapper that implements rquest's CookieStore trait
pub struct CookieStoreWrapper(pub RwLock<cookie_store::CookieStore>);

impl rquest::cookie::CookieStore for CookieStoreWrapper {
    fn set_cookies(&self, url: &url::Url, cookie_headers: &mut dyn Iterator<Item = &rquest::header::HeaderValue>) {
        let mut store = self.0.write().unwrap();
        for header in cookie_headers {
            if let Ok(header_str) = header.to_str() {
                if let Ok(raw_cookie) = cookie::Cookie::parse(header_str) {
                    let cookie = cookie_store::RawCookie::new(raw_cookie.name().to_string(), raw_cookie.value().to_string());
                    let _ = store.insert_raw(&cookie, url);
                }
            }
        }
    }

    fn cookies(&self, url: &url::Url) -> Option<rquest::header::HeaderValue> {
        let store = self.0.read().unwrap();
        let cookies: Vec<String> = store
            .matches(url)
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect();

        if cookies.is_empty() {
            None
        } else {
            rquest::header::HeaderValue::from_str(&cookies.join("; ")).ok()
        }
    }
}

#[allow(dead_code)]
pub struct HeadlessWebBrowser {
    pub(super) client: rquest::Client,
    pub(super) cookie_store: Arc<CookieStoreWrapper>,
    pub(super) renderer: Option<RustRenderer>,
    pub(super) current_url: Option<String>,
    pub(super) current_content: String,
    pub(super) auth_context: AuthContext,
    pub(super) storage: BrowserStorage,
    pub(super) history: NavigationHistory,
    pub(super) scraper: WebScraper,
    pub(super) form_analyzer: FormAnalyzer,
    pub(super) analyzed_forms: Vec<FormInfo>,
}

impl HeadlessWebBrowser {
    pub fn new() -> Arc<Mutex<Self>> {
        Self::new_with_engine(crate::engine::engine_trait::EngineType::Boa)
    }

    pub fn new_with_engine(engine_type: crate::engine::engine_trait::EngineType) -> Arc<Mutex<Self>> {
        // Create shared cookie store for cookie management
        let cookie_store = Arc::new(CookieStoreWrapper(
            RwLock::new(cookie_store::CookieStore::default())
        ));

        // Configure client with Chrome 131 TLS/HTTP2 fingerprint impersonation
        // This makes the browser appear as a real Chrome 131 to Cloudflare and other
        // fingerprinting systems by matching:
        // - TLS cipher suites and extension order (JA3 fingerprint)
        // - HTTP/2 settings and frame behavior (Akamai fingerprint)
        // - Header ordering and values
        let client = rquest::Client::builder()
            .emulation(Emulation::Chrome131)
            .cookie_provider(Arc::clone(&cookie_store))
            .redirect(rquest::redirect::Policy::limited(10)) // Follow up to 10 redirects
            .timeout(std::time::Duration::from_secs(30))
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .zstd(true) // Chrome 131+ supports zstd compression
            .build()
            .expect("Failed to create HTTP client with Chrome 131 emulation");

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

    pub fn get_chrome_headers(&self, url: &str) -> rquest::header::HeaderMap {
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
        self.analyzed_forms.iter().filter(|f| f.opens_new_window).collect()
    }

    /// Find form information by submit button selector
    pub fn find_form_by_submit_button(&self, button_selector: &str) -> Option<FormInfo> {
        self.form_analyzer.find_form_by_submit_button(&self.current_content, button_selector).ok().flatten()
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

    /// Get any pending navigation requests queued by JavaScript execution
    /// (from link clicks, form submissions, etc.)
    /// Returns a list of NavigationRequest that the caller should process
    pub fn get_pending_navigations(&self) -> Vec<thalora_browser_apis::browser::navigation_bridge::NavigationRequest> {
        thalora_browser_apis::browser::navigation_bridge::drain_navigation_requests()
    }

    /// Check if there are any pending navigation requests
    pub fn has_pending_navigations(&self) -> bool {
        thalora_browser_apis::browser::navigation_bridge::has_pending_navigations()
    }

    /// Resolve a potentially relative URL against the current page URL
    pub fn resolve_url(&self, url: &str) -> Option<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Some(url.to_string())
        } else if let Some(ref current_url) = self.current_url {
            // Resolve relative to current URL
            match url::Url::parse(current_url) {
                Ok(base) => match base.join(url) {
                    Ok(resolved) => Some(resolved.to_string()),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Execute JavaScript from a trusted source (like Cloudflare challenges) without security checks.
    /// This allows challenge scripts to use advanced JavaScript features that would normally be blocked.
    ///
    /// # Security
    /// Only use this for scripts from known trusted domains like challenges.cloudflare.com.
    pub async fn execute_javascript_trusted(&mut self, js_code: &str) -> Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            // Use trusted execution with 10 second timeout for complex challenge scripts
            renderer.evaluate_javascript_trusted(js_code, std::time::Duration::from_secs(10))
        } else {
            Err(anyhow::anyhow!("Renderer not available"))
        }
    }

    pub fn create_standard_browser_headers(&self, url: &str) -> HeaderMap {
        use thalora_constants::SEC_CH_UA;

        let mut headers = HeaderMap::new();

        // Use shared Chrome USER_AGENT constant - single source of truth!
        headers.insert(USER_AGENT, HeaderValue::from_static(super::USER_AGENT));

        // Chrome-style Accept header
        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
        ));

        // Chrome language preferences
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        // Chrome compression support (Chrome 131+ supports zstd)
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br, zstd"));

        // Chrome sends client hints - use centralized constants for version consistency
        headers.insert("sec-ch-ua", HeaderValue::from_static(SEC_CH_UA));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));

        // Priority hints (Chrome 131+)
        headers.insert("priority", HeaderValue::from_static("u=0, i"));

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
            let current_url = self.current_url.clone().unwrap_or_else(|| "about:blank".to_string());

            // Create navigation activation object with current and target URLs
            let activation_script = format!(r#"
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
            "#, target_url, current_url);

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
                            eprintln!("🔍 DEBUG: PageSwap event with activation result: {}", renderer.js_value_to_string(call_res));
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

        // Note: rquest::Client will be dropped automatically
        // It will close connection pools when the last reference is dropped
    }
}