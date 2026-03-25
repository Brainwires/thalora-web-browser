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
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    /// Access to private IPs, localhost, and cloud metadata endpoints is blocked.
    pub async fn navigate_to_with_js_option(
        &mut self,
        url: &str,
        wait_for_load: bool,
        wait_for_js: bool,
    ) -> Result<String> {
        eprintln!(
            "🔍 DEBUG: navigate_to_with_js_option - URL: {}, wait_for_load: {}, wait_for_js: {}",
            url, wait_for_load, wait_for_js
        );

        // SECURITY: Validate URL to prevent SSRF attacks
        // Block access to private IPs, localhost, and cloud metadata endpoints
        SsrfProtection::new().is_safe_url(url)?;

        // Dispatch pageswap event before navigation
        self.dispatch_pageswap_event(url).await?;

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
                eprintln!("🔍 DEBUG: HTTP request error details: {}", e);
                if let Some(source) = e.source() {
                    eprintln!("🔍 DEBUG: Error source: {}", source);
                }
                e
            })?;
        let content = response.text().await?;

        // Store the current content and URL
        self.current_content = content.clone();
        self.current_url = Some(url.to_string());

        eprintln!("🔍 DEBUG: Content length: {} characters", content.len());

        // Fetch external stylesheets from <link rel="stylesheet"> tags
        let external_css = self.fetch_all_stylesheets().await;
        self.external_stylesheets = external_css;
        eprintln!(
            "🔍 DEBUG: Stored {} external stylesheets",
            self.external_stylesheets.len()
        );

        // Store HTML content for form parsing when needed
        eprintln!(
            "🔍 DEBUG: HTML content available for form parsing: {} characters",
            content.len()
        );

        // Update document HTML in the renderer if available.
        // Non-fatal: the page content is already loaded; a renderer update failure
        // shouldn't abort navigation.
        if let Some(ref mut renderer) = self.renderer {
            eprintln!("🔍 DEBUG: Updating document HTML via renderer");
            if let Err(e) = renderer.update_document_html(&content) {
                eprintln!(
                    "WARNING: Failed to update document HTML: {} (continuing)",
                    e
                );
            }
        }

        // Add human-like navigation delays only in Stealth mode (MCP/headless)
        if self.navigation_mode == NavigationMode::Stealth {
            let navigation_delay = 1000 + (rand::random::<u64>() % 2000); // 1-3 seconds
            eprintln!(
                "🔍 DEBUG: Adding human-like navigation delay: {}ms",
                navigation_delay
            );
            sleep(Duration::from_millis(navigation_delay)).await;

            let processing_delay = 500 + (rand::random::<u64>() % 1500); // 0.5-2 seconds
            eprintln!(
                "🔍 DEBUG: Adding page processing delay: {}ms",
                processing_delay
            );
            sleep(Duration::from_millis(processing_delay)).await;
        } else {
            eprintln!("🔍 DEBUG: Interactive mode - skipping anti-bot delays");
        }

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        match self.form_analyzer.analyze_forms(&content) {
            Ok(forms) => {
                self.analyzed_forms = forms;
                eprintln!(
                    "🔍 DEBUG: Analyzed {} forms on page",
                    self.analyzed_forms.len()
                );

                let new_window_forms = self
                    .analyzed_forms
                    .iter()
                    .filter(|f| f.opens_new_window)
                    .count();
                if new_window_forms > 0 {
                    eprintln!(
                        "🔍 DEBUG: Found {} forms that open new windows",
                        new_window_forms
                    );
                }
            }
            Err(e) => eprintln!("🔍 DEBUG: Form analysis failed: {}", e),
        }

        // Headless browser behavior: load HTML and make it ready for interaction
        eprintln!("🔍 DEBUG: HTML content loaded");

        // Add to navigation history
        let title = self
            .extract_title(&content)
            .unwrap_or_else(|| url.to_string());
        self.add_to_history(url.to_string(), title);

        // If wait_for_js is enabled, execute page scripts and wait for completion
        if wait_for_js {
            eprintln!("🔍 DEBUG: wait_for_js enabled, executing page scripts");

            // Execute non-deferred inline scripts from the page
            self.execute_page_scripts(&content, false).await?;

            // Fire DOMContentLoaded event
            eprintln!("🔍 DEBUG: Firing DOMContentLoaded event");
            self.fire_dom_content_loaded().await?;

            // Execute deferred scripts AFTER DOMContentLoaded
            self.execute_page_scripts(&content, true).await?;

            // Wait for JavaScript execution to settle.
            // Use a short timeout in Interactive mode (GUI) to keep page loads fast.
            // Stealth mode (MCP/headless) can afford a longer wait.
            let js_timeout = if self.navigation_mode == NavigationMode::Stealth {
                5000
            } else {
                2000
            };
            match self.wait_for_js_execution(js_timeout).await {
                Ok(_) => eprintln!("🔍 DEBUG: JavaScript execution completed successfully"),
                Err(e) => eprintln!("🔍 DEBUG: JavaScript execution timeout (non-fatal): {}", e),
            }

            // After JS execution, capture the modified DOM back into current_content.
            // JavaScript may have added/removed elements (e.g., sidebar TOC, UI panels).
            // This ensures thalora_compute_styled_tree() gets the JS-modified DOM.
            match self
                .execute_javascript("document.documentElement.outerHTML")
                .await
            {
                Ok(html) if !html.is_empty() && html.len() > 100 => {
                    let original_len = self.current_content.len();
                    let full_html = if html.starts_with("<!") {
                        html.clone()
                    } else if html.starts_with("<html") {
                        format!("<!DOCTYPE html>{}", html)
                    } else {
                        format!("<!DOCTYPE html><html>{}</html>", html)
                    };
                    self.current_content = full_html;
                    eprintln!(
                        "🔍 DEBUG: Updated current_content with JS-modified DOM ({} → {} bytes)",
                        original_len,
                        self.current_content.len()
                    );
                }
                Ok(html) => {
                    eprintln!(
                        "🔍 DEBUG: outerHTML too short ({}), keeping original content",
                        html.len()
                    );
                }
                Err(e) => {
                    eprintln!("⚠️  WARNING: Failed to serialize JS-modified DOM: {}", e);
                    // Keep original content — JS may not have modified the DOM
                }
            }
        } else {
            eprintln!("🔍 DEBUG: wait_for_js disabled, ready for direct DOM interaction");
        }

        // Reset bypass_cache flag after navigation completes
        self.bypass_cache = false;

        Ok(self.current_content.clone())
    }

    /// Execute JavaScript from <script> tags in the page HTML
    pub async fn execute_page_scripts(&mut self, html: &str, only_deferred: bool) -> Result<()> {
        let mode = if only_deferred {
            "deferred"
        } else {
            "non-deferred"
        };
        eprintln!(
            "🔍 DEBUG: execute_page_scripts - extracting and executing {} scripts",
            mode
        );

        // Parse HTML to extract script tags
        let document = scraper::Html::parse_document(html);
        let script_selector = scraper::Selector::parse("script").unwrap();

        let mut scripts_executed = 0;
        let mut scripts_failed = 0;
        let mut external_scripts_fetched = 0;

        // Get the current URL to resolve relative script paths
        let base_url = self
            .current_url
            .clone()
            .unwrap_or_else(|| "https://example.com".to_string());

        for script_element in document.select(&script_selector) {
            // Get the script type attribute
            let script_type = script_element
                .value()
                .attr("type")
                .unwrap_or("text/javascript");

            // Check if this is a Cloudflare Rocket Loader mangled script
            // Rocket Loader rewrites script types to random tokens ending in "-text/javascript"
            let is_rocket_loader_script = script_type.ends_with("-text/javascript");

            // Skip non-JavaScript scripts (like templates, JSON-LD, etc.)
            // But allow Cloudflare Rocket Loader scripts
            if !is_rocket_loader_script
                && script_type != "text/javascript"
                && script_type != "application/javascript"
                && !script_type.is_empty()
                && script_type != "module"
            {
                eprintln!("🔍 DEBUG: Skipping script with type: {}", script_type);
                continue;
            }

            // Check for async/defer attributes
            let is_async = script_element.value().attr("async").is_some();
            let is_defer = script_element.value().attr("defer").is_some();

            // Filter based on what we're executing in this pass
            if only_deferred && !is_defer {
                continue; // Skip non-deferred scripts in deferred pass
            }
            if !only_deferred && is_defer {
                eprintln!(
                    "🔍 DEBUG: Skipping deferred script (will execute after DOMContentLoaded)"
                );
                continue; // Skip deferred scripts in normal pass
            }

            // Check if this is an external script
            if let Some(src) = script_element.value().attr("src") {
                eprintln!("🔍 DEBUG: Found external script: {}", src);

                // Resolve the URL (handle relative paths, protocol-relative URLs)
                let script_url = self.resolve_script_url(&base_url, src)?;

                eprintln!("🔍 DEBUG: Fetching external script from: {}", script_url);

                // Fetch the external script
                match self.fetch_external_script(&script_url).await {
                    Ok(script_content) => {
                        eprintln!(
                            "🔍 DEBUG: Fetched external script ({} chars)",
                            script_content.len()
                        );
                        external_scripts_fetched += 1;

                        // Execute the fetched script
                        match self.execute_javascript(&script_content).await {
                            Ok(result) => {
                                scripts_executed += 1;
                                eprintln!("🔍 DEBUG: External script executed successfully");
                            }
                            Err(e) => {
                                scripts_failed += 1;
                                eprintln!("⚠️  WARNING: External script execution failed: {}", e);
                                // Continue with other scripts even if one fails
                            }
                        }
                    }
                    Err(e) => {
                        scripts_failed += 1;
                        eprintln!(
                            "⚠️  WARNING: Failed to fetch external script {}: {}",
                            script_url, e
                        );
                        // Continue with other scripts
                    }
                }
            } else {
                // Inline script
                let script_content: String = script_element.text().collect();

                if script_content.trim().is_empty() {
                    continue;
                }

                eprintln!(
                    "🔍 DEBUG: Executing inline script ({} chars)",
                    script_content.len()
                );

                // Execute the script through the JavaScript engine
                match self.execute_javascript(&script_content).await {
                    Ok(result) => {
                        scripts_executed += 1;
                        eprintln!("🔍 DEBUG: Inline script executed successfully");
                    }
                    Err(e) => {
                        scripts_failed += 1;
                        eprintln!("⚠️  WARNING: Inline script execution failed: {}", e);
                        // Continue with other scripts even if one fails
                    }
                }
            }
        }

        eprintln!(
            "[JS] {} scripts: {} executed, {} failed, {} external fetched",
            mode, scripts_executed, scripts_failed, external_scripts_fetched
        );

        // Give scripts time to settle after execution
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Fire DOMContentLoaded event
    /// This signals that the DOM is fully parsed and deferred scripts should execute
    pub(super) async fn fire_dom_content_loaded(&mut self) -> Result<()> {
        let js_code = r#"
        (function() {
            try {
                // Set document.readyState to 'interactive' first
                Object.defineProperty(document, 'readyState', {
                    value: 'interactive',
                    writable: true,
                    configurable: true
                });

                // Create and dispatch DOMContentLoaded event
                var event = new Event('DOMContentLoaded', {
                    bubbles: true,
                    cancelable: false
                });
                document.dispatchEvent(event);

                // Then set readyState to 'complete'
                Object.defineProperty(document, 'readyState', {
                    value: 'complete',
                    writable: true,
                    configurable: true
                });

                // Fire load event on window
                var loadEvent = new Event('load', {
                    bubbles: false,
                    cancelable: false
                });
                window.dispatchEvent(loadEvent);

                return 'DOMContentLoaded and load events fired';
            } catch(e) {
                return 'Error: ' + e.message;
            }
        })()
        "#;

        match self.execute_javascript(js_code).await {
            Ok(result) => {
                eprintln!("🔍 DEBUG: DOMContentLoaded event result: {}", result);
                Ok(())
            }
            Err(e) => {
                eprintln!("⚠️  WARNING: Failed to fire DOMContentLoaded: {}", e);
                Ok(()) // Non-fatal
            }
        }
    }

    /// Wait for JavaScript execution to complete and DOM to stabilize using events
    pub async fn wait_for_js_execution(&mut self, timeout_ms: u64) -> Result<()> {
        eprintln!(
            "🔍 DEBUG: wait_for_js_execution - waiting for JS to complete (timeout: {}ms)",
            timeout_ms
        );

        let js_code = format!(
            r#"
        (function() {{
            return new Promise(function(resolve, reject) {{
                var timeoutId = setTimeout(function() {{
                    resolve(false); // Timeout
                }}, {});

                function checkReady() {{
                    try {{
                        // Check if document is ready
                        if (typeof document === 'undefined') return false;
                        if (document.readyState !== 'complete') return false;

                        // Check if there are pending AJAX requests (jQuery)
                        if (typeof jQuery !== 'undefined' && jQuery.active > 0) return false;

                        // Check for Angular pending requests
                        if (typeof angular !== 'undefined') {{
                            try {{
                                var ng = angular.element(document.body).injector();
                                if (ng && ng.get('$http').pendingRequests.length > 0) return false;
                            }} catch(e) {{
                                // Angular not fully initialized
                            }}
                        }}

                        // Check for pending fetch/XHR using performance API
                        if (typeof performance !== 'undefined' && performance.getEntriesByType) {{
                            try {{
                                var nav = performance.getEntriesByType('navigation')[0];
                                if (nav && nav.loadEventEnd === 0) return false;
                            }} catch(e) {{
                                // Performance API not fully supported
                            }}
                        }}

                        return true; // Ready
                    }} catch(e) {{
                        return false;
                    }}
                }}

                // If already ready, resolve immediately
                if (checkReady()) {{
                    clearTimeout(timeoutId);
                    resolve(true);
                    return;
                }}

                // Listen for readystatechange event
                document.addEventListener('readystatechange', function handler() {{
                    if (checkReady()) {{
                        clearTimeout(timeoutId);
                        document.removeEventListener('readystatechange', handler);
                        resolve(true);
                    }}
                }});

                // Also listen for load event as fallback
                window.addEventListener('load', function handler() {{
                    // Give a small delay for final scripts to execute
                    setTimeout(function() {{
                        if (checkReady()) {{
                            clearTimeout(timeoutId);
                            window.removeEventListener('load', handler);
                            resolve(true);
                        }}
                    }}, 100);
                }});
            }});
        }})()
        "#,
            timeout_ms
        );

        match self.execute_javascript(&js_code).await {
            Ok(result) => {
                if result.trim() == "true" {
                    eprintln!("🔍 DEBUG: wait_for_js_execution - JavaScript execution complete");
                    Ok(())
                } else {
                    eprintln!("🔍 DEBUG: wait_for_js_execution - timeout reached");
                    Err(anyhow!("Timeout waiting for JavaScript execution"))
                }
            }
            Err(e) => {
                eprintln!("🔍 DEBUG: wait_for_js_execution - error: {}", e);
                Err(e)
            }
        }
    }

    /// Wait for an element to appear in the DOM using MutationObserver
    /// Returns true if element found, false if timeout reached
    pub async fn wait_for_element(&mut self, selector: &str, timeout_ms: u64) -> Result<bool> {
        eprintln!(
            "🔍 DEBUG: wait_for_element - waiting for selector: {} (timeout: {}ms)",
            selector, timeout_ms
        );

        let escaped_selector = selector.replace("\"", "\\\"").replace("'", "\\'");

        // Use MutationObserver to watch for element appearance
        let js_code = format!(
            r#"
        (function() {{
            return new Promise(function(resolve, reject) {{
                // Check if element already exists
                var element = document.querySelector("{}");
                if (element) {{
                    resolve(true);
                    return;
                }}

                // Set up timeout
                var timeoutId = setTimeout(function() {{
                    observer.disconnect();
                    resolve(false); // Timeout - element not found
                }}, {});

                // Set up MutationObserver to watch for DOM changes
                var observer = new MutationObserver(function(mutations) {{
                    var element = document.querySelector("{}");
                    if (element) {{
                        clearTimeout(timeoutId);
                        observer.disconnect();
                        resolve(true);
                    }}
                }});

                // Observe the entire document for child additions
                observer.observe(document.body || document.documentElement, {{
                    childList: true,
                    subtree: true
                }});
            }});
        }})()
        "#,
            escaped_selector, timeout_ms, escaped_selector
        );

        match self.execute_javascript(&js_code).await {
            Ok(result) => {
                let found = result.trim() == "true";
                if found {
                    eprintln!("🔍 DEBUG: wait_for_element - element found: {}", selector);
                } else {
                    eprintln!(
                        "🔍 DEBUG: wait_for_element - timeout reached, element not found: {}",
                        selector
                    );
                }
                Ok(found)
            }
            Err(e) => {
                eprintln!("🔍 DEBUG: wait_for_element - error: {}", e);
                Err(e)
            }
        }
    }

    /// Navigate to URL without adding to history (for back/forward navigation)
    /// This is an internal method used by go_back() and go_forward()
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    pub(super) async fn navigate_internal(&mut self, url: &str) -> Result<String> {
        eprintln!(
            "🔍 DEBUG: navigate_internal - URL: {} (no history update)",
            url
        );

        // SECURITY: Validate URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        // Dispatch pageswap event before navigation
        self.dispatch_pageswap_event(url).await?;

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
        eprintln!(
            "🔍 DEBUG: navigate_internal - stored {} external stylesheets",
            self.external_stylesheets.len()
        );

        // Update document HTML in the renderer if available.
        // Non-fatal: the page content is already loaded; a renderer update failure
        // shouldn't abort navigation.
        if let Some(ref mut renderer) = self.renderer {
            if let Err(e) = renderer.update_document_html(&content) {
                eprintln!(
                    "WARNING: Failed to update document HTML (navigate_internal): {} (continuing)",
                    e
                );
            }
        }

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        if let Ok(forms) = self.form_analyzer.analyze_forms(&content) {
            self.analyzed_forms = forms;
        }

        Ok(content)
    }

    /// Resolve a script URL relative to the base URL
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
        if !self.bypass_cache {
            if let Some(cached) = self.resource_cache.get(url) {
                eprintln!("🔍 DEBUG: CACHE HIT (stylesheet): {}", url);
                return Ok(cached.content.clone());
            }
        }

        // SECURITY: Validate stylesheet URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        eprintln!("🔍 DEBUG: CACHE MISS (stylesheet): {}", url);
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
                        eprintln!("🔍 DEBUG: Found external stylesheet: {}", resolved_url);
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
            if !self.bypass_cache {
                if let Some(cached) = self.resource_cache.get(url) {
                    eprintln!("🔍 DEBUG: CACHE HIT (stylesheet): {}", url);
                    cached_results.insert(i, cached.content.clone());
                    continue;
                }
            }
            uncached.push((i, url.clone()));
        }

        eprintln!(
            "🔍 DEBUG: Stylesheets: {} cached, {} to fetch",
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
                        eprintln!(
                            "🔍 DEBUG: CACHE MISS (stylesheet) fetched: {} ({} chars)",
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

        eprintln!(
            "🔍 DEBUG: Successfully resolved {} of {} stylesheets",
            stylesheets.len(),
            stylesheet_urls.len()
        );
        stylesheets
    }

    /// Fetch an external script from a URL, using the resource cache when available.
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks via script loading.
    pub(super) async fn fetch_external_script(&mut self, url: &str) -> Result<String> {
        // Check cache first (unless bypass_cache is set for reload)
        if !self.bypass_cache {
            if let Some(cached) = self.resource_cache.get(url) {
                eprintln!("🔍 DEBUG: CACHE HIT (script): {}", url);
                return Ok(cached.content.clone());
            }
        }

        // SECURITY: Validate script URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        eprintln!("🔍 DEBUG: CACHE MISS (script): {}", url);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch script: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Script fetch failed with status: {}",
                response.status()
            ));
        }

        let content = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read script content: {}", e))?;

        // Insert into cache
        self.resource_cache.insert(url.to_string(), content.clone());

        Ok(content)
    }
}
