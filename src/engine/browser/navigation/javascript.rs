use anyhow::{Result, anyhow};
use rand;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{Duration, sleep};

use super::scripts::{HistoryMode, WAIT_POLL};
use crate::engine::browser::types::NavigationMode;
use crate::engine::security::SsrfProtection;

impl super::super::HeadlessWebBrowser {
    /// Navigate to URL with full control over waiting behavior
    ///
    /// The page is fetched and loaded as the current page: its scripts run
    /// (see [`load_page`](Self::load_page)), and navigations they ask for
    /// (`location.href = …`) are followed. `wait_for_js` lets the page's
    /// timers and requests run for up to 5 s instead of 1 s before
    /// returning.
    ///
    /// Returns the response body when no script ran, otherwise the document
    /// as the scripts left it, serialized.
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
        let content = self
            .fetch_and_load(url, wait_for_js, HistoryMode::Push)
            .await?;
        Ok(self.follow_script_navigations().await.unwrap_or(content))
    }

    /// Fetch `url` and make it the current page (headers, scripts,
    /// stylesheets, forms, history). Navigations the page's scripts ask for
    /// are left queued.
    pub(super) async fn fetch_and_load(
        &mut self,
        url: &str,
        wait_for_js: bool,
        history: HistoryMode,
    ) -> Result<String> {
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

        // The document's URL is where the response came from (after
        // redirects): `location`, relative URLs and the page's origin
        let final_url = response.url().to_string();
        let url = final_url.as_str();

        let content = response.text().await?;

        // Replace the current page with the parsed response and run its
        // scripts
        let scripts_ran = self.load_page(url, &content, wait_for_js).await;

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
        match self
            .page
            .with_doc(|doc| self.form_analyzer.analyze_forms(doc))
        {
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
        let title = self.page_title().unwrap_or_else(|| url.to_string());
        match history {
            HistoryMode::Push => self.add_to_history(url.to_string(), title),
            HistoryMode::Replace => {
                let index = self.history.current_index;
                match self.history.entries.get_mut(index) {
                    Some(entry) => {
                        entry.url = url.to_string();
                        entry.title = title;
                    }
                    None => self.add_to_history(url.to_string(), title),
                }
            }
            HistoryMode::Keep => {}
        }

        // Reset bypass_cache flag after navigation completes
        self.bypass_cache = false;

        // What the scripts made of the page, if any ran
        Ok(if scripts_ran {
            self.page.to_html()
        } else {
            content
        })
    }

    /// Wait until `condition` holds.
    /// Returns Ok(true) if it held within `timeout_ms`, Ok(false) otherwise.
    ///
    /// The condition is checked, the page's event loop is pumped for a short
    /// slice (timers, fetches; navigations the page asks for are followed)
    /// and the condition is checked again, until it holds or the timeout has
    /// passed. A page with nothing pending (and every static page) cannot
    /// change any more, so the answer is returned without waiting for the
    /// timeout.
    ///
    /// `NetworkIdle` holds when nothing is pending on the page at all: no
    /// request, timer or task. The page runtime does not tell requests and
    /// timers apart, so a page with a repeating timer never counts as idle.
    pub async fn wait_for_condition(
        &mut self,
        condition: &crate::engine::browser::types::WaitCondition,
        timeout_ms: u64,
    ) -> Result<bool> {
        use crate::engine::browser::types::WaitCondition;

        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let (met, idle_if_unmet) = match condition {
                WaitCondition::Selector(selector) => (self.element_exists(selector)?, false),
                WaitCondition::Text(text) => {
                    (self.get_current_content().contains(text.as_str()), false)
                }
                WaitCondition::UrlContains(fragment) => (
                    self.page
                        .url()
                        .is_some_and(|url| url.contains(fragment.as_str())),
                    false,
                ),
                // Idle is what the pumping below reports
                WaitCondition::NetworkIdle => (!self.page.has_js(), true),
            };
            if met {
                return Ok(true);
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if !self.page.has_js() || remaining.is_zero() {
                return Ok(false);
            }
            let idle = self.settle(remaining.min(WAIT_POLL)).await;
            let navigated = self.follow_script_navigations().await.is_some();
            if idle && !navigated {
                // Nothing is pending: the page cannot change by itself
                return Ok(if idle_if_unmet {
                    true
                } else {
                    self.condition_holds_now(condition)?
                });
            }
        }
    }

    /// One check of a wait condition against the page as it is.
    fn condition_holds_now(
        &self,
        condition: &crate::engine::browser::types::WaitCondition,
    ) -> Result<bool> {
        use crate::engine::browser::types::WaitCondition;
        Ok(match condition {
            WaitCondition::Selector(selector) => self.element_exists(selector)?,
            WaitCondition::Text(text) => self.get_current_content().contains(text.as_str()),
            WaitCondition::UrlContains(fragment) => self
                .page
                .url()
                .is_some_and(|url| url.contains(fragment.as_str())),
            WaitCondition::NetworkIdle => true,
        })
    }

    /// Whether an element matching `selector` is in the page right now.
    pub(super) fn element_exists(&self, selector: &str) -> Result<bool> {
        Ok(self
            .page
            .with_doc(|doc| doc.query_selector(doc.document(), selector))
            .map_err(|_| anyhow!("Invalid CSS selector: {}", selector))?
            .is_some())
    }

    /// Wait until an element matching `selector` is in the page, for up to
    /// `timeout_ms`. Returns true if found, false otherwise.
    ///
    /// The page's event loop runs while waiting, so elements added by
    /// timers, requests or promise jobs are found. When the page has nothing
    /// pending (and on a static page) the answer is returned at once: the
    /// element cannot appear any more.
    pub async fn wait_for_element(&mut self, selector: &str, timeout_ms: u64) -> Result<bool> {
        let found = self
            .wait_for_condition(
                &crate::engine::browser::types::WaitCondition::Selector(selector.to_string()),
                timeout_ms,
            )
            .await?;
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

        // Replace the current page with the parsed response and run its
        // scripts
        let scripts_ran = self.load_page(url, &content, false).await;

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
        if let Ok(forms) = self
            .page
            .with_doc(|doc| self.form_analyzer.analyze_forms(doc))
        {
            self.analyzed_forms = forms;
        }

        let content = if scripts_ran {
            self.page.to_html()
        } else {
            content
        };
        Ok(self.follow_script_navigations().await.unwrap_or(content))
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
                page_url: self.get_current_url(),
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
            && !csp.allows_style(url, self.page.url())
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
        let base_url = self
            .get_current_url()
            .unwrap_or_else(|| "https://example.com".to_string());

        // (href, integrity) of every <link rel="stylesheet">, in document order
        let links: Vec<(String, Option<String>)> = self.page.with_doc(|doc| {
            doc.query_selector_all(doc.document(), "link")
                .unwrap_or_default()
                .into_iter()
                .filter_map(|link| {
                    let link = doc.as_element(link)?;
                    // Only process <link rel="stylesheet"> elements
                    let rel = link.attr("rel").unwrap_or("");
                    if !rel
                        .split_whitespace()
                        .any(|r| r.eq_ignore_ascii_case("stylesheet"))
                    {
                        return None;
                    }
                    Some((
                        link.attr("href")?.to_string(),
                        link.attr("integrity").map(str::to_string),
                    ))
                })
                .collect()
        });

        let mut stylesheet_urls = Vec::new();

        // Collect integrity attributes alongside URLs for SRI verification
        let mut integrity_map: HashMap<String, String> = HashMap::new();

        for (href, integrity) in links {
            // Resolve relative URLs
            match self.resolve_script_url(&base_url, &href) {
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
                    if let Some(integrity) = integrity.as_deref() {
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
