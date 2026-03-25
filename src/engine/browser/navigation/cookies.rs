use anyhow::Result;
use url::Url;

impl super::super::HeadlessWebBrowser {
    /// Parse cookies from HTTP response headers
    /// Note: The shared CookieStoreMutex handles Set-Cookie headers automatically
    /// This method is available for manual cookie parsing if needed
    pub(super) fn parse_cookies(&self, _headers: &reqwest::header::HeaderMap) -> Result<()> {
        // The cookie_store automatically handles Set-Cookie headers from responses
        // This function exists for backward compatibility and future manual parsing needs
        Ok(())
    }

    /// Get cookies for a specific domain
    /// Returns a vector of "name=value" strings for all cookies matching the domain
    pub fn get_cookies(&self, domain: &str) -> Vec<String> {
        // Build a URL from the domain for cookie matching
        let url = match Url::parse(&format!("https://{}/", domain)) {
            Ok(u) => u,
            Err(_) => {
                // Try with http if https fails
                match Url::parse(&format!("http://{}/", domain)) {
                    Ok(u) => u,
                    Err(_) => return vec![],
                }
            }
        };

        // Lock the cookie store and get matching cookies
        match self.cookie_store.lock() {
            Ok(store) => store
                .matches(&url)
                .iter()
                .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                .collect(),
            Err(_) => {
                tracing::warn!("Failed to acquire cookie store lock");
                vec![]
            }
        }
    }

    /// Get all cookies as "name=value" pairs, optionally filtered by domain
    pub fn get_all_cookies(&self) -> Vec<(String, String, String)> {
        match self.cookie_store.lock() {
            Ok(store) => store
                .iter_any()
                .map(|cookie| {
                    (
                        cookie.domain().unwrap_or("").to_string(),
                        cookie.name().to_string(),
                        cookie.value().to_string(),
                    )
                })
                .collect(),
            Err(_) => {
                tracing::warn!("Failed to acquire cookie store lock");
                vec![]
            }
        }
    }

    /// Set a cookie for a specific domain
    /// The cookie string should be in standard cookie format: "name=value; Path=/; Secure"
    pub fn set_cookie(&mut self, domain: &str, cookie_str: &str) -> Result<()> {
        // Build a URL from the domain
        let url = Url::parse(&format!("https://{}/", domain))
            .map_err(|e| anyhow::anyhow!("Invalid domain '{}': {}", domain, e))?;

        // Lock the store and parse+insert the cookie
        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire cookie store lock"))?;

        store
            .parse(cookie_str, &url)
            .map_err(|e| anyhow::anyhow!("Failed to parse/insert cookie: {}", e))?;

        Ok(())
    }

    /// Set multiple cookies from a header value string (semicolon-separated)
    pub fn set_cookies_from_header(&mut self, domain: &str, header_value: &str) -> Result<()> {
        let url = Url::parse(&format!("https://{}/", domain))
            .map_err(|e| anyhow::anyhow!("Invalid domain '{}': {}", domain, e))?;

        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire cookie store lock"))?;

        // Parse each cookie in the header
        for cookie_str in header_value
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            if let Err(e) = store.parse(cookie_str, &url) {
                tracing::warn!("Failed to insert cookie '{}': {}", cookie_str, e);
            }
        }

        Ok(())
    }

    /// Clear all cookies
    pub fn clear_cookies(&mut self) -> Result<()> {
        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire cookie store lock"))?;

        store.clear();
        Ok(())
    }

    /// Clear cookies for a specific domain
    pub fn clear_cookies_for_domain(&mut self, domain: &str) -> Result<usize> {
        let url = Url::parse(&format!("https://{}/", domain))
            .map_err(|e| anyhow::anyhow!("Invalid domain '{}': {}", domain, e))?;

        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire cookie store lock"))?;

        // Get cookies matching the domain and remove them
        let cookies_to_remove: Vec<(String, String, String)> = store
            .matches(&url)
            .iter()
            .map(|c| {
                (
                    c.domain().unwrap_or("").to_string(),
                    c.name().to_string(),
                    c.path().unwrap_or("/").to_string(),
                )
            })
            .collect();

        let count = cookies_to_remove.len();

        for (cookie_domain, name, path) in cookies_to_remove {
            store.remove(&cookie_domain, &path, &name);
        }

        Ok(count)
    }

    /// Get the cookie header string for a URL (for manual request building)
    pub fn get_cookie_header(&self, url_str: &str) -> Option<String> {
        let url = Url::parse(url_str).ok()?;

        let store = self.cookie_store.lock().ok()?;
        let cookies: Vec<_> = store
            .matches(&url)
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect();

        if cookies.is_empty() {
            None
        } else {
            Some(cookies.join("; "))
        }
    }
}
