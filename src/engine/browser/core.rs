use crate::engine::browser::scraper::WebScraper;
use crate::engine::browser::types::{
    AuthContext, BrowserStorage, HistoryEntry, NavigationHistory, NavigationMode, ResourceCache,
};
use crate::engine::browser::{FormAnalyzer, FormInfo};
use anyhow::Result;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, REFERER,
    UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};
use reqwest_cookie_store::CookieStoreMutex;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[allow(dead_code)]
pub struct HeadlessWebBrowser {
    pub(super) client: reqwest::Client,
    pub(super) cookie_store: Arc<CookieStoreMutex>,
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
    /// Parsed Content-Security-Policy for the current page
    pub(super) csp_policy: Option<super::navigation::csp::CspPolicy>,
    /// Parsed Permissions-Policy for the current page
    pub(super) permissions_policy: Option<super::navigation::csp::PermissionsPolicy>,
    /// HSTS store for upgrading HTTP to HTTPS
    pub(super) hsts_store: super::navigation::hsts::HstsStore,
    /// Referrer-Policy for the current page (e.g., "strict-origin-when-cross-origin")
    pub(super) referrer_policy: Option<String>,
    /// X-Content-Type-Options nosniff flag for the current page
    pub(super) nosniff: bool,
    /// Field values typed via `type_text_into_element`, keyed by field name,
    /// for the page at `filled_values_url`; merged into `submit_form`.
    pub(super) filled_values: HashMap<String, String>,
    pub(super) filled_values_url: Option<String>,
    /// Element refs issued by the last `snapshot()` of this page.
    pub(super) snapshot_refs: super::snapshot::RefTable,
}

impl HeadlessWebBrowser {
    pub fn new() -> Rc<Mutex<Self>> {
        // Create shared cookie store for cookie management
        let cookie_store = Arc::new(CookieStoreMutex::new(
            reqwest_cookie_store::CookieStore::default(),
        ));

        // Configure client with enhanced stealth capabilities
        // Use centralized USER_AGENT constant for consistency
        let client = reqwest::Client::builder()
            // SSRF: only connect to public addresses, re-check redirects
            .dns_resolver(Arc::new(
                crate::engine::security::ssrf::http::PublicOnlyResolver,
            ))
            .redirect(crate::engine::security::ssrf::http::redirect_policy())
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
            csp_policy: None,
            permissions_policy: None,
            hsts_store: super::navigation::hsts::HstsStore::new(),
            referrer_policy: None,
            nosniff: false,
            filled_values: HashMap::new(),
            filled_values_url: None,
            snapshot_refs: Default::default(),
        };

        Rc::new(Mutex::new(browser))
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

    /// Execute JavaScript in the page and return the raw string result.
    ///
    /// There is no JavaScript engine at the moment (Boa was removed, V8 is
    /// not wired in yet), so this always fails with
    /// [`JS_UNAVAILABLE`](crate::engine::JS_UNAVAILABLE).
    pub async fn execute_javascript(&mut self, _js_code: &str) -> Result<String> {
        Err(crate::engine::js_unavailable())
    }

    /// Render the current page to PNG with the built-in layout engine
    /// (including fetched external stylesheets).
    pub fn screenshot_png(
        &mut self,
        options: crate::engine::renderer::paint::ScreenshotOptions,
    ) -> Result<Vec<u8>> {
        if self.current_content.is_empty() {
            return Err(anyhow::anyhow!("No page loaded. Navigate to a page first."));
        }
        let layout = crate::engine::renderer::page_layout::compute_page_layout_with_css(
            &self.current_content,
            options.width as f32,
            options.height as f32,
            &self.external_stylesheets,
        )?;
        crate::engine::renderer::paint::render_png(&layout, options)
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
        if url.starts_with("https://www.google.com") || url.starts_with("https://www.bing.com") {
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

        // Apply Referrer-Policy when setting Referer header
        if let Some(ref current_url) = self.current_url {
            let referer = match self.referrer_policy.as_deref() {
                Some("no-referrer") => None,
                Some("origin") => {
                    // Send only the origin (scheme + host + port)
                    url::Url::parse(current_url)
                        .ok()
                        .map(|u| format!("{}://{}/", u.scheme(), u.host_str().unwrap_or("")))
                }
                Some("same-origin") => {
                    // Only send referrer for same-origin requests
                    let same_origin = url::Url::parse(current_url)
                        .ok()
                        .and_then(|curr| {
                            url::Url::parse(url).ok().map(|target| {
                                curr.scheme() == target.scheme()
                                    && curr.host_str() == target.host_str()
                            })
                        })
                        .unwrap_or(false);
                    if same_origin {
                        Some(current_url.clone())
                    } else {
                        None
                    }
                }
                Some("strict-origin") => {
                    // Send origin only, and only if HTTPS->HTTPS (not HTTPS->HTTP)
                    let is_downgrade = current_url.starts_with("https://")
                        && url.starts_with("http://")
                        && !url.starts_with("https://");
                    if is_downgrade {
                        None
                    } else {
                        url::Url::parse(current_url)
                            .ok()
                            .map(|u| format!("{}://{}/", u.scheme(), u.host_str().unwrap_or("")))
                    }
                }
                Some("strict-origin-when-cross-origin") | None => {
                    // Default policy: full URL for same-origin, origin for cross-origin,
                    // nothing for HTTPS→HTTP downgrade
                    let is_downgrade = current_url.starts_with("https://")
                        && url.starts_with("http://")
                        && !url.starts_with("https://");
                    if is_downgrade {
                        None
                    } else {
                        let same_origin = url::Url::parse(current_url)
                            .ok()
                            .and_then(|curr| {
                                url::Url::parse(url).ok().map(|target| {
                                    curr.scheme() == target.scheme()
                                        && curr.host_str() == target.host_str()
                                })
                            })
                            .unwrap_or(false);
                        if same_origin {
                            Some(current_url.clone())
                        } else {
                            url::Url::parse(current_url).ok().map(|u| {
                                format!("{}://{}/", u.scheme(), u.host_str().unwrap_or(""))
                            })
                        }
                    }
                }
                _ => Some(current_url.clone()), // unsafe-url or unknown: send full URL
            };

            if let Some(ref referer_value) = referer
                && let Ok(hv) = HeaderValue::from_str(referer_value)
            {
                headers.insert(REFERER, hv);
            }
        }

        headers
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
