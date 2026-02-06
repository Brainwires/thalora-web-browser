use anyhow::{Result, anyhow};
use tokio::time::{sleep, Duration};
use std::error::Error;
use std::collections::HashMap;

use crate::engine::security::SsrfProtection;
use thalora_browser_apis::dom::document::ScriptEntry;

#[cfg(feature = "solver")]
use crate::features::solver::{ChallengeSolver, ChallengeDetector, DetectedChallenge, ChallengeType};

/// Maximum number of Cloudflare challenge retry attempts
const MAX_CF_RETRIES: u32 = 3;

#[cfg(feature = "solver")]
/// Default viewport dimensions for behavioral simulation
const VIEWPORT_WIDTH: f64 = 1366.0;
#[cfg(feature = "solver")]
const VIEWPORT_HEIGHT: f64 = 768.0;

impl super::super::HeadlessWebBrowser {
    /// Detect if the page content is a Cloudflare challenge page
    /// Uses simple string matching — no solver dependency
    fn is_cloudflare_challenge(&self, content: &str) -> bool {
        Self::is_challenge_content(content)
    }

    // =========================================================================
    // Solver-based challenge handling (only when "solver" feature is enabled)
    // =========================================================================

    #[cfg(feature = "solver")]
    /// Detect if the page content is a challenge page using the challenge solver
    fn detect_challenge(&self, content: &str, url: &str) -> Option<DetectedChallenge> {
        let detector = ChallengeDetector::new();
        detector.detect(content, url)
    }

    #[cfg(feature = "solver")]
    /// Handle challenge by simulating realistic browser behavior (solver mode)
    async fn handle_cloudflare_challenge(&mut self, url: &str, content: &str) -> Result<String> {
        let solver = ChallengeSolver::new();

        let challenge = match solver.detect_challenge(content, url) {
            Some(c) => c,
            None => return Ok(content.to_string()),
        };

        eprintln!("CHALLENGE: Detected {:?} (confidence: {:.2})", challenge.challenge_type, challenge.confidence);

        self.current_content = content.to_string();
        self.current_url = Some(url.to_string());

        // Set window.location.href before executing scripts
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_page_url(url)?;
        }

        if let Some(ref mut renderer) = self.renderer {
            renderer.update_document_html(content)?;
        }

        // Execute page scripts
        self.execute_page_scripts(content, false).await?;
        self.fire_dom_content_loaded().await?;
        self.execute_page_scripts(content, true).await?;

        // Behavioral simulation
        match solver.generate_interaction_js(&challenge, VIEWPORT_WIDTH, VIEWPORT_HEIGHT) {
            Ok(behavior_js) => {
                let wrapped_behavior_js = format!(
                    r#"(async function() {{
                        try {{
                            {}
                            window._asyncResult = {{ success: true, completed: true }};
                        }} catch (e) {{
                            window._asyncResult = {{ success: false, error: e.message }};
                        }}
                        window._asyncComplete = true;
                    }})()"#,
                    behavior_js.trim_start_matches("(async function() {")
                        .trim_end_matches("})()")
                        .trim()
                );

                let behavior_result = if let Some(ref mut renderer) = self.renderer {
                    let behavior_timeout = std::time::Duration::from_millis(10000);
                    renderer.evaluate_javascript_with_async_wait(&wrapped_behavior_js, behavior_timeout, 50)
                } else {
                    self.execute_javascript(&behavior_js).await
                };

                match behavior_result {
                    Ok(_) => {}
                    Err(e) => eprintln!("CHALLENGE: Behavioral simulation warning: {} (continuing)", e),
                }
            }
            Err(e) => {
                eprintln!("CHALLENGE: Could not generate behavioral simulation: {} (continuing)", e);
            }
        }

        // Wait for resolution
        let timeout_ms = challenge.expected_resolution_time_ms.max(10000);
        let wait_js = solver.generate_wait_for_resolution_js(&challenge, timeout_ms);

        let wait_result = if let Some(ref mut renderer) = self.renderer {
            let timeout_duration = std::time::Duration::from_millis(timeout_ms + 1000);
            renderer.evaluate_javascript_with_async_wait(&wait_js, timeout_duration, 100)
        } else {
            self.execute_javascript(&wait_js).await
        };

        match wait_result {
            Ok(result) => {
                let result_str = result.trim();
                if result_str.contains("\"success\":true") || result_str.contains("success: true") || result_str == "true" {
                    // Bridge: sync JS document.cookie into the HTTP cookie store
                    self.bridge_js_cookies_to_http_store(url);

                    // Retry the original request with cookies now set
                    let headers = self.create_standard_browser_headers(url);
                    let response = self.client.get(url).headers(headers).send().await?;
                    let new_content = response.text().await?;

                    if self.detect_challenge(&new_content, url).is_some() {
                        Err(anyhow!("Challenge could not be resolved automatically"))
                    } else {
                        Ok(new_content)
                    }
                } else {
                    Err(anyhow!("Challenge resolution failed: {}", result_str))
                }
            }
            Err(e) => Err(e)
        }
    }

    // =========================================================================
    // Natural challenge handling (when "solver" feature is disabled)
    // =========================================================================

    #[cfg(not(feature = "solver"))]
    /// Handle challenge naturally by executing the page's JavaScript and
    /// waiting for cookies/redirects, without any behavioral injection
    async fn handle_cloudflare_challenge(&mut self, url: &str, content: &str) -> Result<String> {
        eprintln!("CHALLENGE: Detected challenge page, executing JS naturally for {}", url);

        self.current_content = content.to_string();
        self.current_url = Some(url.to_string());

        // Set window.location.href BEFORE updating HTML or executing scripts
        // so that relative URL resolution (fetch, XHR, dynamic scripts) works
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_page_url(url)?;
        }

        // Update the renderer with challenge page HTML
        if let Some(ref mut renderer) = self.renderer {
            renderer.update_document_html(content)?;
        }

        // Execute ALL page scripts (challenge JS that sets cookies + redirects)
        self.execute_page_scripts(content, false).await?;
        self.fire_dom_content_loaded().await?;
        self.execute_page_scripts(content, true).await?;

        // Wait for challenge JS to compute and set cookies
        // Cloudflare's non-interactive challenge typically needs 3-8 seconds
        let wait_result = self.wait_for_challenge_resolution(15000).await;
        match wait_result {
            Ok(true) => eprintln!("CHALLENGE: Challenge JS appears to have completed"),
            Ok(false) => eprintln!("CHALLENGE: Challenge resolution timed out"),
            Err(e) => eprintln!("CHALLENGE: Error waiting for resolution: {} (continuing)", e),
        }

        // Bridge: sync JS document.cookie into the HTTP cookie store
        // This ensures cookies set by challenge JavaScript (like cf_clearance)
        // are included in the re-fetch request
        self.bridge_js_cookies_to_http_store(url);

        // Re-fetch the original URL — cookies should now be set
        let headers = self.create_standard_browser_headers(url);
        let response = self.client.get(url).headers(headers).send().await?;
        let new_content = response.text().await?;

        if Self::is_challenge_content(&new_content) {
            Err(anyhow!("Challenge could not be resolved — still on challenge page"))
        } else {
            eprintln!("CHALLENGE: Successfully passed challenge!");
            Ok(new_content)
        }
    }

    #[cfg(not(feature = "solver"))]
    /// Wait for challenge JS to complete by checking for cf_clearance cookie
    /// or other resolution indicators
    async fn wait_for_challenge_resolution(&mut self, timeout_ms: u64) -> Result<bool> {
        let js_code = format!(r#"
        (function() {{
            return new Promise(function(resolve) {{
                var timeoutId = setTimeout(function() {{
                    resolve(false);
                }}, {timeout_ms});

                // Poll for resolution indicators
                var checkInterval = setInterval(function() {{
                    try {{
                        // Check if cf_clearance cookie was set
                        if (document.cookie && document.cookie.indexOf('cf_clearance') !== -1) {{
                            clearInterval(checkInterval);
                            clearTimeout(timeoutId);
                            resolve(true);
                            return;
                        }}
                        // Check if the page is no longer a challenge page
                        var title = document.title || '';
                        if (title !== 'Just a moment...' && title !== '') {{
                            clearInterval(checkInterval);
                            clearTimeout(timeoutId);
                            resolve(true);
                            return;
                        }}
                    }} catch(e) {{}}
                }}, 500);
            }});
        }})()
        "#);

        let result = if let Some(ref mut renderer) = self.renderer {
            let timeout_duration = std::time::Duration::from_millis(timeout_ms + 1000);
            renderer.evaluate_javascript_with_async_wait(&js_code, timeout_duration, 200)
        } else {
            self.execute_javascript(&js_code).await
        };

        match result {
            Ok(r) => Ok(r.trim() == "true"),
            Err(e) => Err(e),
        }
    }

    /// Navigate to URL with full control over waiting behavior
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    /// Access to private IPs, localhost, and cloud metadata endpoints is blocked.
    ///
    /// # Cloudflare Handling
    /// This function automatically detects and attempts to resolve Cloudflare
    /// challenge pages by executing their JavaScript naturally.
    pub async fn navigate_to_with_js_option(&mut self, url: &str, wait_for_load: bool, wait_for_js: bool) -> Result<String> {
        // SECURITY: Validate URL to prevent SSRF attacks
        SsrfProtection::new().is_safe_url(url)?;

        // Dispatch pageswap event before navigation
        self.dispatch_pageswap_event(url).await?;

        // Get browser-specific headers for stealth
        let headers = self.create_standard_browser_headers(url);

        // Fetch the page content with proper browser headers
        let response = self.client.get(url).headers(headers).send().await.map_err(|e| {
            if let Some(source) = e.source() {
                eprintln!("HTTP request error: {} (source: {})", e, source);
            }
            e
        })?;
        let mut content = response.text().await?;

        // Check for Cloudflare challenge and handle it
        let mut retry_count = 0;
        while self.is_cloudflare_challenge(&content) && retry_count < MAX_CF_RETRIES {
            retry_count += 1;
            eprintln!("CLOUDFLARE: Challenge detected, attempt {}/{}", retry_count, MAX_CF_RETRIES);

            match self.handle_cloudflare_challenge(url, &content).await {
                Ok(new_content) => {
                    content = new_content;
                    if !self.is_cloudflare_challenge(&content) {
                        eprintln!("CLOUDFLARE: Challenge resolved on attempt {}", retry_count);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("CLOUDFLARE: Challenge handling failed: {}", e);
                    if retry_count >= MAX_CF_RETRIES {
                        return Err(anyhow!("Failed to pass Cloudflare challenge after {} attempts: {}", MAX_CF_RETRIES, e));
                    }
                    // Wait before retrying
                    sleep(Duration::from_millis(2000 * retry_count as u64)).await;

                    // Refresh the page for another attempt
                    let headers = self.create_standard_browser_headers(url);
                    let response = self.client.get(url).headers(headers).send().await?;
                    content = response.text().await?;
                }
            }
        }

        // Final sanity check
        if self.is_cloudflare_challenge(&content) {
            return Err(anyhow!("Cloudflare challenge could not be resolved — still on challenge page after {} attempts", MAX_CF_RETRIES));
        }

        // Store the current content and URL
        self.current_content = content.clone();
        self.current_url = Some(url.to_string());

        // Set window.location.href so scripts can resolve relative URLs
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_page_url(url)?;
        }

        // Update document HTML in the renderer if available
        if let Some(ref mut renderer) = self.renderer {
            renderer.update_document_html(&content)?;
        }

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        match self.form_analyzer.analyze_forms(&content) {
            Ok(forms) => {
                self.analyzed_forms = forms;
            }
            Err(_e) => {}
        }

        // Add to navigation history
        let title = self.extract_title(&content).unwrap_or_else(|| url.to_string());
        self.add_to_history(url.to_string(), title);

        // If wait_for_js is enabled, execute page scripts and wait for completion
        if wait_for_js {
            // Execute non-deferred inline scripts from the page
            self.execute_page_scripts(&content, false).await?;

            // Fire DOMContentLoaded event
            self.fire_dom_content_loaded().await?;

            // Execute deferred scripts AFTER DOMContentLoaded
            self.execute_page_scripts(&content, true).await?;

            // Wait for JavaScript execution to complete
            match self.wait_for_js_execution(10000).await {
                Ok(_) => {}
                Err(_e) => {} // timeout is non-fatal
            }

            // After JS execution, serialize the live DOM back to HTML
            // This captures any content rendered dynamically by JavaScript
            match self.execute_javascript("(function(){ try { return document.documentElement.outerHTML; } catch(e) { return ''; } })()").await {
                Ok(updated_html) => {
                    let trimmed = updated_html.trim();
                    if !trimmed.is_empty() && trimmed.len() > self.current_content.len() / 2 {
                        // Wrap in doctype + html if needed
                        let full_html = if trimmed.starts_with("<!") || trimmed.starts_with("<html") {
                            trimmed.to_string()
                        } else {
                            format!("<!DOCTYPE html>{}", trimmed)
                        };
                        self.current_content = full_html;
                    }
                }
                Err(_) => {} // Fall back to original content
            }
        }

        Ok(self.current_content.clone())
    }

    /// Execute JavaScript from <script> tags in the page HTML
    pub async fn execute_page_scripts(&mut self, html: &str, only_deferred: bool) -> Result<()> {
        // Parse HTML to extract script tags
        let document = scraper::Html::parse_document(html);
        let script_selector = scraper::Selector::parse("script").unwrap();

        let mut _scripts_executed = 0;
        let mut _external_scripts_fetched = 0;

        // Get the current URL to resolve relative script paths
        let base_url = self.current_url.clone().unwrap_or_else(|| "https://example.com".to_string());

        // Check if this is a challenge page by content - if so, use trusted execution for all scripts
        let is_challenge_by_content = Self::is_challenge_content(html);

        for script_element in document.select(&script_selector) {
            // Get the script type attribute
            let script_type = script_element.value().attr("type").unwrap_or("text/javascript");

            // Check if this is a Cloudflare Rocket Loader mangled script
            let is_rocket_loader_script = script_type.ends_with("-text/javascript");

            // Skip non-JavaScript scripts (like templates, JSON-LD, etc.)
            if !is_rocket_loader_script
                && script_type != "text/javascript"
                && script_type != "application/javascript"
                && !script_type.is_empty()
                && script_type != "module" {
                continue;
            }

            // Check for async/defer attributes
            let is_async = script_element.value().attr("async").is_some();
            let is_defer = script_element.value().attr("defer").is_some();

            // Filter based on what we're executing in this pass
            if only_deferred && !is_defer {
                continue;
            }
            if !only_deferred && is_defer {
                continue;
            }

            // Check if this is an external script
            if let Some(src) = script_element.value().attr("src") {
                // Resolve the URL (handle relative paths, protocol-relative URLs)
                let script_url = self.resolve_script_url(&base_url, src)?;
                // Collect ALL attributes from the HTML element for script registration
                let mut script_attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    script_attrs.insert(key.to_string(), value.to_string());
                }

                // Check if this is a trusted challenge provider script or on a challenge page
                let is_trusted = Self::is_trusted_challenge_script(&script_url) || is_challenge_by_content;

                // Fetch the external script
                match self.fetch_external_script(&script_url).await {
                    Ok(script_content) => {
                        _external_scripts_fetched += 1;

                        // Register the script BEFORE executing it
                        let script_entry = ScriptEntry {
                            src: Some(script_url.clone()),
                            script_type: script_attrs.get("type").cloned(),
                            async_: is_async,
                            defer: is_defer,
                            text: script_content.clone(),
                            attributes: script_attrs.clone(),
                        };

                        if let Some(ref mut renderer) = self.renderer {
                            let _ = renderer.register_script(script_entry.clone());
                            let _ = renderer.set_current_script(&script_entry);
                        }

                        // Execute the fetched script
                        let exec_result = if is_trusted {
                            self.execute_javascript_trusted(&script_content).await
                        } else {
                            self.execute_javascript(&script_content).await
                        };

                        if let Some(ref mut renderer) = self.renderer {
                            let _ = renderer.clear_current_script();
                        }

                        match exec_result {
                            Ok(_result) => {
                                _scripts_executed += 1;
                            }
                            Err(e) => {
                                eprintln!("WARNING: External script execution failed for {}: {}", script_url, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WARNING: Failed to fetch external script {}: {}", script_url, e);
                    }
                }
            } else {
                // Inline script
                let script_content: String = script_element.text().collect();

                if script_content.trim().is_empty() {
                    continue;
                }

                // Collect ALL attributes from the HTML element for script registration
                let mut script_attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    script_attrs.insert(key.to_string(), value.to_string());
                }

                // Register the inline script
                let script_entry = ScriptEntry {
                    src: None,
                    script_type: script_attrs.get("type").cloned(),
                    async_: is_async,
                    defer: is_defer,
                    text: script_content.clone(),
                    attributes: script_attrs.clone(),
                };

                if let Some(ref mut renderer) = self.renderer {
                    let _ = renderer.register_script(script_entry.clone());
                    let _ = renderer.set_current_script(&script_entry);
                }

                // Use trusted execution for challenge page scripts
                let is_challenge_page = Self::is_challenge_page(&base_url) || is_challenge_by_content;

                let exec_result = if is_challenge_page {
                    self.execute_javascript_trusted(&script_content).await
                } else {
                    self.execute_javascript(&script_content).await
                };

                if let Some(ref mut renderer) = self.renderer {
                    let _ = renderer.clear_current_script();
                }

                match exec_result {
                    Ok(_result) => {
                        _scripts_executed += 1;
                    }
                    Err(e) => {
                        eprintln!("WARNING: Inline script execution failed ({}B): {}", script_content.len(), e);
                    }
                }
            }
        }

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
            Ok(_) => Ok(()),
            Err(_) => Ok(()) // Non-fatal
        }
    }

    /// Wait for JavaScript execution to complete and DOM to stabilize using events
    pub async fn wait_for_js_execution(&mut self, timeout_ms: u64) -> Result<()> {
        let js_code = format!(r#"
        (function() {{
            return new Promise(function(resolve, reject) {{
                var timeoutId = setTimeout(function() {{
                    resolve(false); // Timeout
                }}, {});

                function checkReady() {{
                    try {{
                        if (typeof document === 'undefined') return false;
                        if (document.readyState !== 'complete') return false;

                        if (typeof jQuery !== 'undefined' && jQuery.active > 0) return false;

                        if (typeof angular !== 'undefined') {{
                            try {{
                                var ng = angular.element(document.body).injector();
                                if (ng && ng.get('$http').pendingRequests.length > 0) return false;
                            }} catch(e) {{}}
                        }}

                        if (typeof performance !== 'undefined' && performance.getEntriesByType) {{
                            try {{
                                var nav = performance.getEntriesByType('navigation')[0];
                                if (nav && nav.loadEventEnd === 0) return false;
                            }} catch(e) {{}}
                        }}

                        return true;
                    }} catch(e) {{
                        return false;
                    }}
                }}

                if (checkReady()) {{
                    clearTimeout(timeoutId);
                    resolve(true);
                    return;
                }}

                document.addEventListener('readystatechange', function handler() {{
                    if (checkReady()) {{
                        clearTimeout(timeoutId);
                        document.removeEventListener('readystatechange', handler);
                        resolve(true);
                    }}
                }});

                window.addEventListener('load', function handler() {{
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
        "#, timeout_ms);

        match self.execute_javascript(&js_code).await {
            Ok(result) => {
                if result.trim() == "true" {
                    Ok(())
                } else {
                    Err(anyhow!("Timeout waiting for JavaScript execution"))
                }
            }
            Err(e) => Err(e)
        }
    }

    /// Wait for an element to appear in the DOM using MutationObserver
    /// Returns true if element found, false if timeout reached
    pub async fn wait_for_element(&mut self, selector: &str, timeout_ms: u64) -> Result<bool> {
        let escaped_selector = selector.replace("\"", "\\\"").replace("'", "\\'");

        let js_code = format!(r#"
        (function() {{
            return new Promise(function(resolve, reject) {{
                var element = document.querySelector("{}");
                if (element) {{
                    resolve(true);
                    return;
                }}

                var timeoutId = setTimeout(function() {{
                    observer.disconnect();
                    resolve(false);
                }}, {});

                var observer = new MutationObserver(function(mutations) {{
                    var element = document.querySelector("{}");
                    if (element) {{
                        clearTimeout(timeoutId);
                        observer.disconnect();
                        resolve(true);
                    }}
                }});

                observer.observe(document.body || document.documentElement, {{
                    childList: true,
                    subtree: true
                }});
            }});
        }})()
        "#, escaped_selector, timeout_ms, escaped_selector);

        match self.execute_javascript(&js_code).await {
            Ok(result) => Ok(result.trim() == "true"),
            Err(e) => Err(e)
        }
    }

    /// Navigate to URL without adding to history (for back/forward navigation)
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    pub(super) async fn navigate_internal(&mut self, url: &str) -> Result<String> {
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

        // Set window.location.href so scripts can resolve relative URLs
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_page_url(url)?;
        }

        // Update document HTML in the renderer if available
        if let Some(ref mut renderer) = self.renderer {
            renderer.update_document_html(&content)?;
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
        if src.starts_with("//") {
            return Ok(format!("https:{}", src));
        }

        if src.starts_with("http://") || src.starts_with("https://") {
            return Ok(src.to_string());
        }

        let base = url::Url::parse(base_url)
            .map_err(|e| anyhow!("Invalid base URL: {}", e))?;

        let resolved = base.join(src)
            .map_err(|e| anyhow!("Failed to resolve script URL: {}", e))?;

        Ok(resolved.to_string())
    }

    /// Fetch an external script from a URL
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks via script loading.
    pub(super) async fn fetch_external_script(&self, url: &str) -> Result<String> {
        SsrfProtection::new().is_safe_url(url)?;

        let response = self.client.get(url).send().await
            .map_err(|e| anyhow!("Failed to fetch script: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Script fetch failed with status: {}", response.status()));
        }

        let content = response.text().await
            .map_err(|e| anyhow!("Failed to read script content: {}", e))?;

        Ok(content)
    }

    /// Check if a script URL is from a trusted challenge provider.
    fn is_trusted_challenge_script(url: &str) -> bool {
        let trusted_domains = [
            "challenges.cloudflare.com",
            "www.google.com/recaptcha",
            "www.gstatic.com/recaptcha",
            "hcaptcha.com",
            "js.hcaptcha.com",
            "newassets.hcaptcha.com",
            "cdn.jsdelivr.net/npm/turnstile",
        ];

        for domain in &trusted_domains {
            if url.contains(domain) {
                return true;
            }
        }

        false
    }

    /// Check if the current page URL indicates a challenge page.
    fn is_challenge_page(url: &str) -> bool {
        let challenge_indicators = [
            "challenges.cloudflare.com",
            "/cdn-cgi/challenge",
            "captcha",
            "recaptcha",
            "hcaptcha",
        ];

        let url_lower = url.to_lowercase();
        for indicator in &challenge_indicators {
            if url_lower.contains(indicator) {
                return true;
            }
        }

        false
    }

    /// Check if the page content indicates it's a Cloudflare challenge page.
    fn is_challenge_content(content: &str) -> bool {
        // Definitive markers that ONLY appear on challenge pages
        let definitive_markers = [
            "_cf_chl_opt",           // Challenge options object
            "challenge-error-text",  // Challenge error UI element
        ];

        // Strong markers — present on challenge pages but could appear in other contexts
        let strong_markers = [
            "challenges.cloudflare.com",
            "/cdn-cgi/challenge-platform/",
        ];

        let supporting_markers = [
            "Just a moment...",
            "cf-turnstile",
            "cf-please-wait",
            "Checking your browser",
        ];

        // Check definitive markers first
        if definitive_markers.iter().any(|m| content.contains(m)) {
            return true;
        }

        // Require BOTH strong AND supporting markers to avoid false positives
        // from pages that happen to reference Cloudflare's CDN
        let has_strong = strong_markers.iter().any(|m| content.contains(m));
        let has_supporting = supporting_markers.iter().any(|m| content.contains(m));

        has_strong && has_supporting
    }

    /// Bridge cookies from the JS engine's `document.cookie` into the HTTP cookie store.
    ///
    /// When challenge JavaScript sets cookies (like `cf_clearance`) via `document.cookie`,
    /// those only live in the JS engine's document object. The HTTP client's cookie store
    /// (used for subsequent requests) doesn't see them. This method extracts all cookies
    /// from `document.cookie` and inserts them into the HTTP cookie store so the next
    /// HTTP request includes them.
    fn bridge_js_cookies_to_http_store(&mut self, url: &str) {
        let js_cookies = if let Some(ref mut renderer) = self.renderer {
            match renderer.evaluate_javascript_direct("document.cookie") {
                Ok(raw) => {
                    // evaluate_javascript_direct may return the string quoted
                    let trimmed = raw.trim().trim_matches('"').to_string();
                    if trimmed.is_empty() { None } else { Some(trimmed) }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        if let Some(js_cookies) = js_cookies {
            let domain = url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(String::from))
                .unwrap_or_default();

            if domain.is_empty() {
                return;
            }

            let mut bridged = 0usize;
            for cookie_pair in js_cookies.split("; ") {
                let cookie_pair = cookie_pair.trim();
                if !cookie_pair.is_empty() && cookie_pair.contains('=') {
                    // Format as a Set-Cookie header with domain and path so the
                    // cookie store accepts and sends it for subsequent requests
                    let set_cookie_str = format!("{}; Domain={}; Path=/", cookie_pair, domain);
                    if self.set_cookie(&domain, &set_cookie_str).is_ok() {
                        bridged += 1;
                    }
                }
            }

            if bridged > 0 {
                eprintln!("CHALLENGE: Bridged {} JS cookie(s) to HTTP store for domain {}", bridged, domain);
            }
        }
    }
}
