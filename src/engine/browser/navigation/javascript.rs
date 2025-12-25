use anyhow::{Result, anyhow};
use tokio::time::{sleep, Duration};
use std::error::Error;
use rand;

use crate::engine::security::SsrfProtection;

impl super::super::HeadlessWebBrowser {
    /// Navigate to URL with full control over waiting behavior
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    /// Access to private IPs, localhost, and cloud metadata endpoints is blocked.
    pub async fn navigate_to_with_js_option(&mut self, url: &str, wait_for_load: bool, wait_for_js: bool) -> Result<String> {
        eprintln!("🔍 DEBUG: navigate_to_with_js_option - URL: {}, wait_for_load: {}, wait_for_js: {}", url, wait_for_load, wait_for_js);

        // SECURITY: Validate URL to prevent SSRF attacks
        // Block access to private IPs, localhost, and cloud metadata endpoints
        SsrfProtection::new().is_safe_url(url)?;

        // Dispatch pageswap event before navigation
        self.dispatch_pageswap_event(url).await?;

        // Get browser-specific headers for stealth
        let headers = self.create_standard_browser_headers(url);

        // Fetch the page content with proper browser headers
        let response = self.client.get(url).headers(headers).send().await.map_err(|e| {
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

        // Store HTML content for form parsing when needed
        eprintln!("🔍 DEBUG: HTML content available for form parsing: {} characters", content.len());

        // Update document HTML in the renderer if available
        if let Some(ref mut renderer) = self.renderer {
            eprintln!("🔍 DEBUG: Updating document HTML via renderer");
            renderer.update_document_html(&content)?;
        }

        // Add human-like navigation delays inline
        let navigation_delay = 1000 + (rand::random::<u64>() % 2000); // 1-3 seconds
        eprintln!("🔍 DEBUG: Adding human-like navigation delay: {}ms", navigation_delay);
        sleep(Duration::from_millis(navigation_delay)).await;

        let processing_delay = 500 + (rand::random::<u64>() % 1500); // 0.5-2 seconds
        eprintln!("🔍 DEBUG: Adding page processing delay: {}ms", processing_delay);
        sleep(Duration::from_millis(processing_delay)).await;

        // Analyze forms for target="_blank" detection
        self.form_analyzer = self.form_analyzer.clone().with_base_url(url.to_string());
        match self.form_analyzer.analyze_forms(&content) {
            Ok(forms) => {
                self.analyzed_forms = forms;
                eprintln!("🔍 DEBUG: Analyzed {} forms on page", self.analyzed_forms.len());

                let new_window_forms = self.analyzed_forms.iter().filter(|f| f.opens_new_window).count();
                if new_window_forms > 0 {
                    eprintln!("🔍 DEBUG: Found {} forms that open new windows", new_window_forms);
                }
            }
            Err(e) => eprintln!("🔍 DEBUG: Form analysis failed: {}", e),
        }

        // Headless browser behavior: load HTML and make it ready for interaction
        eprintln!("🔍 DEBUG: HTML content loaded");

        // Add to navigation history
        let title = self.extract_title(&content).unwrap_or_else(|| url.to_string());
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

            // Wait for JavaScript execution to complete
            match self.wait_for_js_execution(10000).await {
                Ok(_) => eprintln!("🔍 DEBUG: JavaScript execution completed successfully"),
                Err(e) => eprintln!("🔍 DEBUG: JavaScript execution timeout (non-fatal): {}", e),
            }
        } else {
            eprintln!("🔍 DEBUG: wait_for_js disabled, ready for direct DOM interaction");
        }

        Ok(self.current_content.clone())
    }

    /// Execute JavaScript from <script> tags in the page HTML
    pub async fn execute_page_scripts(&mut self, html: &str, only_deferred: bool) -> Result<()> {
        let mode = if only_deferred { "deferred" } else { "non-deferred" };
        eprintln!("🔍 DEBUG: execute_page_scripts - extracting and executing {} scripts", mode);

        // Parse HTML to extract script tags
        let document = scraper::Html::parse_document(html);
        let script_selector = scraper::Selector::parse("script").unwrap();

        let mut scripts_executed = 0;
        let mut external_scripts_fetched = 0;

        // Get the current URL to resolve relative script paths
        let base_url = self.current_url.clone().unwrap_or_else(|| "https://example.com".to_string());

        for script_element in document.select(&script_selector) {
            // Get the script type attribute
            let script_type = script_element.value().attr("type").unwrap_or("text/javascript");

            // Check if this is a Cloudflare Rocket Loader mangled script
            // Rocket Loader rewrites script types to random tokens ending in "-text/javascript"
            let is_rocket_loader_script = script_type.ends_with("-text/javascript");

            // Skip non-JavaScript scripts (like templates, JSON-LD, etc.)
            // But allow Cloudflare Rocket Loader scripts
            if !is_rocket_loader_script
                && script_type != "text/javascript"
                && script_type != "application/javascript"
                && !script_type.is_empty()
                && script_type != "module" {
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
                eprintln!("🔍 DEBUG: Skipping deferred script (will execute after DOMContentLoaded)");
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
                        eprintln!("🔍 DEBUG: Fetched external script ({} chars)", script_content.len());
                        external_scripts_fetched += 1;

                        // Execute the fetched script
                        match self.execute_javascript(&script_content).await {
                            Ok(result) => {
                                scripts_executed += 1;
                                eprintln!("🔍 DEBUG: External script executed successfully");
                            }
                            Err(e) => {
                                eprintln!("⚠️  WARNING: External script execution failed: {}", e);
                                // Continue with other scripts even if one fails
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  WARNING: Failed to fetch external script {}: {}", script_url, e);
                        // Continue with other scripts
                    }
                }
            } else {
                // Inline script
                let script_content: String = script_element.text().collect();

                if script_content.trim().is_empty() {
                    continue;
                }

                eprintln!("🔍 DEBUG: Executing inline script ({} chars)", script_content.len());

                // Execute the script through the JavaScript engine
                match self.execute_javascript(&script_content).await {
                    Ok(result) => {
                        scripts_executed += 1;
                        eprintln!("🔍 DEBUG: Inline script executed successfully");
                    }
                    Err(e) => {
                        eprintln!("⚠️  WARNING: Inline script execution failed: {}", e);
                        // Continue with other scripts even if one fails
                    }
                }
            }
        }

        eprintln!("🔍 DEBUG: Fetched {} external scripts, executed {} total scripts", external_scripts_fetched, scripts_executed);

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
        eprintln!("🔍 DEBUG: wait_for_js_execution - waiting for JS to complete (timeout: {}ms)", timeout_ms);

        let js_code = format!(r#"
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
        "#, timeout_ms);

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
        eprintln!("🔍 DEBUG: wait_for_element - waiting for selector: {} (timeout: {}ms)", selector, timeout_ms);

        let escaped_selector = selector.replace("\"", "\\\"").replace("'", "\\'");

        // Use MutationObserver to watch for element appearance
        let js_code = format!(r#"
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
        "#, escaped_selector, timeout_ms, escaped_selector);

        match self.execute_javascript(&js_code).await {
            Ok(result) => {
                let found = result.trim() == "true";
                if found {
                    eprintln!("🔍 DEBUG: wait_for_element - element found: {}", selector);
                } else {
                    eprintln!("🔍 DEBUG: wait_for_element - timeout reached, element not found: {}", selector);
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
        eprintln!("🔍 DEBUG: navigate_internal - URL: {} (no history update)", url);

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
        // SECURITY: Validate script URL to prevent SSRF attacks
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
}
