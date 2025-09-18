use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::engine::renderer::RustRenderer;
use crate::engine::browser::types::{AuthContext, BrowserStorage, NavigationHistory, HistoryEntry, StealthConfig};
use crate::engine::browser::stealth::StealthManager;
use crate::engine::browser::scraper::WebScraper;

pub struct HeadlessWebBrowser {
    pub(super) client: reqwest::Client,
    pub(super) renderer: Option<RustRenderer>,
    pub(super) current_url: Option<String>,
    pub(super) current_content: String,
    pub(super) auth_context: AuthContext,
    pub(super) storage: BrowserStorage,
    pub(super) history: NavigationHistory,
    pub(super) stealth_manager: StealthManager,
    pub(super) scraper: WebScraper,
}

impl HeadlessWebBrowser {
    pub fn new() -> Arc<Mutex<Self>> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        let mut renderer = RustRenderer::new();

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

        let stealth_config = StealthConfig::default();
        let stealth_manager = StealthManager::new(stealth_config);
        let scraper = WebScraper::new();

        let browser = Self {
            client,
            renderer: Some(renderer),
            current_url: None,
            current_content: String::new(),
            auth_context,
            storage,
            history,
            stealth_manager,
            scraper,
        };

        let browser_arc = Arc::new(Mutex::new(browser));

        // Setup history API with reference to browser
        if let Err(e) = Self::setup_history_api(browser_arc.clone()) {
            eprintln!("Failed to setup history API: {}", e);
        }

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
}