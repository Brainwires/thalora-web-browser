use anyhow::{Result, anyhow};
use rand;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{Duration, sleep};

use crate::engine::browser::types::NavigationMode;
use crate::engine::security::SsrfProtection;

impl super::super::HeadlessWebBrowser {
    /// Navigate to URL with full control over waiting behavior
    ///
    /// The page is fetched and its static HTML becomes the current content.
    /// There is no JavaScript engine at the moment (Boa was removed, V8 is
    /// not wired in yet), so `wait_for_js` is accepted for compatibility but
    /// page scripts are never executed.
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    /// Access to private IPs, localhost, and cloud metadata endpoints is blocked.
    pub async fn navigate_to_with_js_option(
        &mut self,
        url: &str,
        wait_for_load: bool,
        wait_for_js: bool,
    ) -> Result<String> {
        tracing::debug!(
            "navigate_to_with_js_option - URL: {}, wait_for_load: {}, wait_for_js: {}",
            url,
            wait_for_load,
            wait_for_js
        );

        // SECURITY: Validate URL to prevent SSRF attacks
        // Block access to private IPs, localhost, and cloud metadata endpoints
        SsrfProtection::new().is_safe_url(url)?;

        // HSTS: upgrade http:// to https:// if policy requires it
        let url = if let Some(upgraded) = self.hsts_store.should_upgrade(url) {
            eprintln!("🔒 HSTS: Upgrading {} -> {}", url, upgraded);
            upgraded
        } else {
            url.to_string()
        };
        let url = url.as_str();

        // Get browser-specific headers for stealth
        let headers = self.create_standard_browser_headers(url);

        // Fetch the page content with proper browser headers
        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                tracing::debug!("HTTP request error details: {}", e);
                if let Some(source) = e.source() {
                    tracing::debug!("Error source: {}", source);
                }
                e
            })?;

        // Extract and parse Content-Security-Policy header
        let csp_header = response
            .headers()
            .get("content-security-policy")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(ref csp) = csp_header {
            eprintln!(
                "🔒 CSP: Content-Security-Policy header found: {}",
                if csp.len() > 100 { &csp[..100] } else { csp }
            );
            self.csp_policy = Some(super::csp::CspPolicy::parse(csp));
        } else {
            self.csp_policy = None;
        }

        // Publish the img-src/style-src parts of the policy for the scraper
        // and the layout engine
        self.sync_csp_state();

        // Extract and parse Strict-Transport-Security (HSTS) header
        if let Some(hsts_value) = response
            .headers()
            .get("strict-transport-security")
            .and_then(|v| v.to_str().ok())
            && let Some(domain) = response.url().host_str()
        {
            eprintln!("🔒 HSTS: Storing policy for {}: {}", domain, hsts_value);
            self.hsts_store.parse_header(domain, hsts_value);
        }

        // Extract and parse Permissions-Policy header (or legacy Feature-Policy)
        let permissions_header = response
            .headers()
            .get("permissions-policy")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let feature_policy_header = response
            .headers()
            .get("feature-policy")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(ref pp) = permissions_header {
            self.permissions_policy = Some(super::csp::PermissionsPolicy::parse(pp));
        } else if let Some(ref fp) = feature_policy_header {
            self.permissions_policy = Some(super::csp::PermissionsPolicy::parse_legacy(fp));
        } else {
            self.permissions_policy = None;
        }

        // Extract Referrer-Policy header
        self.referrer_policy = response
            .headers()
            .get("referrer-policy")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Extract X-Content-Type-Options header
        self.nosniff = response
            .headers()
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("nosniff"))
            .unwrap_or(false);

        let content = response.text().await?;

        // Store the current content and URL
        self.current_content = content.clone();
        self.current_url = Some(url.to_string());

        tracing::debug!("Content length: {} characters", content.len());

        // Fetch external stylesheets from <link rel="stylesheet"> tags
        let external_css = self.fetch_all_stylesheets().await;
        self.external_stylesheets = external_css;
        tracing::debug!(
            "Stored {} external stylesheets",
            self.external_stylesheets.len()
        );

        // Store HTML content for form parsing when needed
        tracing::debug!(
            "HTML content available for form parsing: {} characters",
            content.len()
        );

        // Human-like random delays (1.5-5s per navigation) are opt-in: they
        // only help against some bot detection and make every agent action
        // slow. Enable with THALORA_STEALTH_DELAYS=true in Stealth mode.
        let stealth_delays = std::env::var("THALORA_STEALTH_DELAYS")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        if self.navigation_mode == NavigationMode::Stealth && stealth_delays {
            let navigation_delay = 1000 + (rand::random::<u64>() % 2000); // 1-3 seconds
            tracing::debug!("Adding human-like navigation delay: {}ms", navigation_delay);
            sleep(Duration::from_millis(navigation_delay)).await;

            let processing_delay = 500 + (rand::random::<u64>() % 1500); // 0.5-2 seconds
            tracing::debug!("Adding page processing delay: {}ms", processing_delay);
            sleep(Duration::from_millis(processing_delay)).await;
        } else {
            tracing::debug!("Skipping anti-bot delays (THALORA_STEALTH_DELAYS not set)");
        }

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        match self.form_analyzer.analyze_forms(&content) {
            Ok(forms) => {
                self.analyzed_forms = forms;
                tracing::debug!("Analyzed {} forms on page", self.analyzed_forms.len());

                let new_window_forms = self
                    .analyzed_forms
                    .iter()
                    .filter(|f| f.opens_new_window)
                    .count();
                if new_window_forms > 0 {
                    tracing::debug!("Found {} forms that open new windows", new_window_forms);
                }
            }
            Err(e) => tracing::debug!("Form analysis failed: {}", e),
        }

        // Headless browser behavior: load HTML and make it ready for interaction
        tracing::debug!("HTML content loaded");

        // Add to navigation history
        let title = self
            .extract_title(&content)
            .unwrap_or_else(|| url.to_string());
        self.add_to_history(url.to_string(), title);

        if wait_for_js && content.contains("<script") {
            tracing::debug!(
                "wait_for_js requested but page scripts are not executed: {}",
                crate::engine::JS_UNAVAILABLE
            );
        }

        // Reset bypass_cache flag after navigation completes
        self.bypass_cache = false;

        Ok(self.current_content.clone())
    }

    /// Wait until `condition` holds.
    /// Returns Ok(true) if it held within `timeout_ms`, Ok(false) otherwise.
    ///
    /// Without a JavaScript engine the page cannot change on its own, so the
    /// condition is checked once against the loaded page and the result is
    /// returned immediately. `NetworkIdle` always holds: nothing on the page
    /// can start a request.
    pub async fn wait_for_condition(
        &mut self,
        condition: &crate::engine::browser::types::WaitCondition,
        timeout_ms: u64,
    ) -> Result<bool> {
        use crate::engine::browser::types::WaitCondition;

        match condition {
            WaitCondition::Selector(selector) => self.wait_for_element(selector, timeout_ms).await,
            WaitCondition::NetworkIdle => Ok(true),
            WaitCondition::Text(text) => Ok(self.current_content.contains(text.as_str())),
            WaitCondition::UrlContains(fragment) => Ok(self
                .current_url
                .as_deref()
                .is_some_and(|url| url.contains(fragment.as_str()))),
        }
    }

    /// Check whether an element matching `selector` is present in the loaded
    /// page. Returns true if found, false otherwise.
    ///
    /// Without a JavaScript engine the page cannot change on its own, so
    /// this answers immediately instead of polling until `timeout_ms`.
    pub async fn wait_for_element(&mut self, selector: &str, _timeout_ms: u64) -> Result<bool> {
        let parsed = scraper::Selector::parse(selector)
            .map_err(|_| anyhow!("Invalid CSS selector: {}", selector))?;
        let document = scraper::Html::parse_document(&self.current_content);
        let found = document.select(&parsed).next().is_some();
        tracing::debug!("wait_for_element - selector: {} found: {}", selector, found);
        Ok(found)
    }

    /// Navigate to URL without adding to history (for back/forward navigation)
    /// This is an internal method used by go_back() and go_forward()
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    pub(super) async fn navigate_internal(&mut self, url: &str) -> Result<String> {
        tracing::debug!("navigate_internal - URL: {} (no history update)", url);

        // SECURITY: Validate URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        // Get browser-specific headers for stealth
        let headers = self.create_standard_browser_headers(url);

        // Fetch the page content with proper browser headers
        let response = self.client.get(url).headers(headers).send().await?;
        let content = response.text().await?;

        // Store the current content and URL
        self.current_content = content.clone();
        self.current_url = Some(url.to_string());

        // Fetch external stylesheets — required for correct CSS rendering.
        // Previously fetched stylesheets will be cache hits, so this is fast.
        let external_css = self.fetch_all_stylesheets().await;
        self.external_stylesheets = external_css;
        tracing::debug!(
            "navigate_internal - stored {} external stylesheets",
            self.external_stylesheets.len()
        );

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        if let Ok(forms) = self.form_analyzer.analyze_forms(&content) {
            self.analyzed_forms = forms;
        }

        Ok(content)
    }

    /// Publish the parts of the page's CSP that apply to static resources
    /// (img-src, style-src, default-src) for the scraper and layout engine.
    fn sync_csp_state(&self) {
        use super::csp::SourceExpression;
        use crate::engine::security::csp_state::{self, CspSource, CspState};

        fn convert_sources(sources: &[SourceExpression]) -> Vec<CspSource> {
            sources
                .iter()
                .filter_map(|s| match s {
                    SourceExpression::OriginSelf => Some(CspSource::OriginSelf),
                    SourceExpression::UnsafeInline => Some(CspSource::UnsafeInline),
                    SourceExpression::None => Some(CspSource::None),
                    SourceExpression::Url(u) => Some(CspSource::Url(u.clone())),
                    // Nonce, Hash, StrictDynamic and 'unsafe-eval' don't
                    // affect image URLs or inline styles
                    _ => None,
                })
                .collect()
        }

        if let Some(ref csp) = self.csp_policy {
            csp_state::set_csp_state(CspState {
                page_url: self.current_url.clone(),
                has_policy: csp.has_policy(),
                img_src: convert_sources(&csp.img_src),
                style_src: convert_sources(&csp.style_src),
                default_src: convert_sources(&csp.default_src),
            });
        } else {
            csp_state::clear_csp_state();
        }
    }

    /// Resolve a resource URL relative to the base URL
    pub(super) fn resolve_script_url(&self, base_url: &str, src: &str) -> Result<String> {
        // Handle protocol-relative URLs (//example.com/script.js)
        if src.starts_with("//") {
            // Use https by default
            return Ok(format!("https:{}", src));
        }

        // Handle absolute URLs
        if src.starts_with("http://") || src.starts_with("https://") {
            return Ok(src.to_string());
        }

        // Handle relative URLs
        let base = url::Url::parse(base_url).map_err(|e| anyhow!("Invalid base URL: {}", e))?;

        let resolved = base
            .join(src)
            .map_err(|e| anyhow!("Failed to resolve script URL: {}", e))?;

        Ok(resolved.to_string())
    }

    /// Fetch an external stylesheet from a URL, using the resource cache when available.
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks via stylesheet loading.
    pub(super) async fn fetch_external_stylesheet(&mut self, url: &str) -> Result<String> {
        // Check cache first (unless bypass_cache is set for reload)
        if !self.bypass_cache
            && let Some(cached) = self.resource_cache.get(url)
        {
            tracing::debug!("CACHE HIT (stylesheet): {}", url);
            return Ok(cached.content.clone());
        }

        // SECURITY: Validate stylesheet URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        // CSP: Check if style-src allows this external stylesheet
        if let Some(ref csp) = self.csp_policy
            && !csp.allows_style(url, self.current_url.as_deref())
        {
            eprintln!("🔒 CSP: External stylesheet blocked by style-src: {}", url);
            return Err(anyhow!(
                "Stylesheet blocked by Content-Security-Policy style-src"
            ));
        }

        tracing::debug!("CACHE MISS (stylesheet): {}", url);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch stylesheet: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Stylesheet fetch failed with status: {}",
                response.status()
            ));
        }

        let content = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read stylesheet content: {}", e))?;

        // Insert into cache
        self.resource_cache.insert(url.to_string(), content.clone());

        Ok(content)
    }

    /// Fetch all external stylesheets from <link rel="stylesheet"> tags in the page HTML.
    /// Uses the resource cache for previously fetched stylesheets, and fetches uncached
    /// ones concurrently using cloned HTTP clients.
    pub(super) async fn fetch_all_stylesheets(&mut self) -> Vec<String> {
        let html = self.current_content.clone();
        let base_url = self
            .current_url
            .clone()
            .unwrap_or_else(|| "https://example.com".to_string());

        let document = scraper::Html::parse_document(&html);
        let link_selector = scraper::Selector::parse("link").unwrap();

        let mut stylesheet_urls = Vec::new();

        // Collect integrity attributes alongside URLs for SRI verification
        let mut integrity_map: HashMap<String, String> = HashMap::new();

        for link_element in document.select(&link_selector) {
            // Only process <link rel="stylesheet"> elements
            let rel = link_element.value().attr("rel").unwrap_or("");
            if !rel
                .split_whitespace()
                .any(|r| r.eq_ignore_ascii_case("stylesheet"))
            {
                continue;
            }

            if let Some(href) = link_element.value().attr("href") {
                // Resolve relative URLs
                match self.resolve_script_url(&base_url, href) {
                    Ok(resolved_url) => {
                        // Mixed content blocking: block HTTP stylesheets on HTTPS pages
                        if super::mixed_content::should_block_mixed_content(
                            &base_url,
                            &resolved_url,
                            super::mixed_content::ResourceType::Stylesheet,
                        ) == super::mixed_content::MixedContentResult::Block
                        {
                            eprintln!(
                                "🔒 BLOCKED: Mixed content stylesheet {} on HTTPS page",
                                resolved_url
                            );
                            continue;
                        }

                        tracing::debug!("Found external stylesheet: {}", resolved_url);

                        // Store integrity attribute for SRI verification after fetch
                        if let Some(integrity) = link_element.value().attr("integrity") {
                            integrity_map.insert(resolved_url.clone(), integrity.to_string());
                        }

                        stylesheet_urls.push(resolved_url);
                    }
                    Err(e) => {
                        eprintln!(
                            "⚠️  WARNING: Failed to resolve stylesheet URL '{}': {}",
                            href, e
                        );
                    }
                }
            }
        }

        if stylesheet_urls.is_empty() {
            return Vec::new();
        }

        // Separate cached and uncached URLs
        let mut cached_results: HashMap<usize, String> = HashMap::new();
        let mut uncached: Vec<(usize, String)> = Vec::new();

        for (i, url) in stylesheet_urls.iter().enumerate() {
            if !self.bypass_cache
                && let Some(cached) = self.resource_cache.get(url)
            {
                tracing::debug!("CACHE HIT (stylesheet): {}", url);
                cached_results.insert(i, cached.content.clone());
                continue;
            }
            uncached.push((i, url.clone()));
        }

        tracing::debug!(
            "Stylesheets: {} cached, {} to fetch",
            cached_results.len(),
            uncached.len()
        );

        // Fetch uncached stylesheets concurrently using cloned client (no &mut self borrow)
        if !uncached.is_empty() {
            let client = self.client.clone();
            let fetch_futures: Vec<_> = uncached
                .iter()
                .map(|(_, url)| {
                    let client = client.clone();
                    let url = url.clone();
                    async move {
                        // Validate SSRF
                        if let Err(e) = SsrfProtection::new().is_safe_url(&url) {
                            return Err(anyhow!("SSRF blocked: {}", e));
                        }
                        let response = client
                            .get(&url)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to fetch stylesheet: {}", e))?;
                        if !response.status().is_success() {
                            return Err(anyhow!(
                                "Stylesheet fetch failed with status: {}",
                                response.status()
                            ));
                        }
                        let content = response
                            .text()
                            .await
                            .map_err(|e| anyhow!("Failed to read stylesheet content: {}", e))?;
                        Ok::<(String, String), anyhow::Error>((url, content))
                    }
                })
                .collect();

            let results = futures::future::join_all(fetch_futures).await;

            for (result, (idx, url)) in results.into_iter().zip(uncached.iter()) {
                match result {
                    Ok((fetched_url, content)) => {
                        // SRI verification for stylesheets with integrity attribute
                        if let Some(integrity) = integrity_map.get(&fetched_url)
                            && !super::sri::verify_integrity(content.as_bytes(), integrity)
                        {
                            eprintln!(
                                "🔒 BLOCKED: SRI integrity check failed for stylesheet {}",
                                fetched_url
                            );
                            continue;
                        }

                        tracing::debug!(
                            "CACHE MISS (stylesheet) fetched: {} ({} chars)",
                            fetched_url,
                            content.len()
                        );
                        self.resource_cache.insert(fetched_url, content.clone());
                        cached_results.insert(*idx, content);
                    }
                    Err(e) => {
                        eprintln!("⚠️  WARNING: Failed to fetch stylesheet {}: {}", url, e);
                    }
                }
            }
        }

        // Reassemble results in original order
        let mut stylesheets = Vec::new();
        for i in 0..stylesheet_urls.len() {
            if let Some(content) = cached_results.remove(&i) {
                stylesheets.push(content);
            }
        }

        tracing::debug!(
            "Successfully resolved {} of {} stylesheets",
            stylesheets.len(),
            stylesheet_urls.len()
        );
        stylesheets
    }
}
