use anyhow::{Result, anyhow};
use tokio::time::{sleep, Duration};
use std::error::Error;
use std::collections::HashMap;
use rand;

use crate::engine::security::SsrfProtection;
use crate::features::solver::{ChallengeSolver, ChallengeDetector, DetectedChallenge, ChallengeType};
use thalora_browser_apis::dom::document::ScriptEntry;

/// Maximum number of Cloudflare challenge retry attempts
const MAX_CF_RETRIES: u32 = 3;

/// Default viewport dimensions for behavioral simulation
const VIEWPORT_WIDTH: f64 = 1366.0;
const VIEWPORT_HEIGHT: f64 = 768.0;

impl super::super::HeadlessWebBrowser {
    /// Detect if the page content is a challenge page using the challenge solver
    fn detect_challenge(&self, content: &str, url: &str) -> Option<DetectedChallenge> {
        let detector = ChallengeDetector::new();
        detector.detect(content, url)
    }

    /// Detect if the page content is a Cloudflare challenge page (legacy method)
    fn is_cloudflare_challenge(&self, content: &str) -> bool {
        self.detect_challenge(content, "").is_some()
    }

    /// Handle challenge by simulating realistic browser behavior
    /// This is the key insight: we don't "bypass" challenges, we behave like a real browser
    async fn handle_cloudflare_challenge(&mut self, url: &str, content: &str) -> Result<String> {
        // Use the new challenge solver for detection and behavioral simulation
        let solver = ChallengeSolver::new();

        // Detect the specific challenge type
        let challenge = match solver.detect_challenge(content, url) {
            Some(c) => c,
            None => {
                eprintln!("🛡️ CHALLENGE: No challenge detected, returning content as-is");
                return Ok(content.to_string());
            }
        };

        eprintln!("🛡️ CHALLENGE: Detected {:?} (confidence: {:.2})", challenge.challenge_type, challenge.confidence);
        eprintln!("🛡️ CHALLENGE: Requires interaction: {}", challenge.requires_interaction);
        eprintln!("🛡️ CHALLENGE: Starting behavioral resolution for {}", url);

        // Store the challenge page content
        self.current_content = content.to_string();
        self.current_url = Some(url.to_string());

        // Update the renderer with challenge page HTML
        if let Some(ref mut renderer) = self.renderer {
            renderer.update_document_html(content)?;
        }

        // Phase 1: Execute page scripts (Cloudflare's JS challenge)
        eprintln!("🛡️ CHALLENGE: Phase 1 - Executing challenge scripts...");
        self.execute_page_scripts(content, false).await?;
        self.fire_dom_content_loaded().await?;
        self.execute_page_scripts(content, true).await?;

        // Phase 2: Behavioral simulation (act like a real user)
        eprintln!("🛡️ CHALLENGE: Phase 2 - Simulating user behavior...");

        // First, check if trusted event dispatcher is available
        match self.execute_javascript("typeof window.__dispatchTrustedMouseEvent").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: typeof __dispatchTrustedMouseEvent = {:?}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: Error checking __dispatchTrustedMouseEvent: {}", e),
        }

        // Debug: Check what's in the DOM before widget detection (step by step to isolate issues)
        // First, simple test to verify JS execution works
        match self.execute_javascript("'hello'").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: Simple JS test: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: Simple JS test error: {}", e),
        }

        // Test if document exists
        match self.execute_javascript("typeof document").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: typeof document: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: typeof document error: {}", e),
        }

        // Test if body exists
        match self.execute_javascript("document.body ? 'has body' : 'no body'").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: document.body check: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: document.body check error: {}", e),
        }

        // Test body children count
        match self.execute_javascript("document.body && document.body.children ? document.body.children.length : -1").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: body children count: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: body children count error: {}", e),
        }

        // Check for cf-turnstile class
        let cf_check_js = "document.querySelector('.cf-turnstile') ? 'found' : 'not found'";
        match self.execute_javascript(cf_check_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: .cf-turnstile: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: .cf-turnstile error: {}", e),
        }

        // Check cf-turnstile element details
        let cf_detail_js = r#"(function() {
            var el = document.querySelector('.cf-turnstile');
            if (!el) return 'not found';
            return JSON.stringify({
                tagName: el.tagName,
                childCount: el.children ? el.children.length : -1,
                innerHTML: el.innerHTML ? el.innerHTML.substring(0, 200) : 'no innerHTML',
                dataSitekey: el.getAttribute ? (el.getAttribute('data-sitekey') ? 'yes' : 'no') : 'no getAttribute',
                classAttr: el.getAttribute ? (el.getAttribute('class') || 'none') : 'no getAttribute',
                style: el.getAttribute ? (el.getAttribute('style') || 'none') : 'no getAttribute',
                className: el.className || 'empty'
            });
        })()"#;
        match self.execute_javascript(cf_detail_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: .cf-turnstile details: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: .cf-turnstile details error: {}", e),
        }

        // Count iframes
        match self.execute_javascript("document.querySelectorAll('iframe').length").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: iframe count: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: iframe count error: {}", e),
        }

        // Check if turnstile global exists and what methods it has
        match self.execute_javascript("typeof turnstile").await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: typeof turnstile: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: typeof turnstile error: {}", e),
        }

        // Check turnstile object details
        let turnstile_check_js = r#"(function() {
            if (typeof turnstile === 'undefined') return 'turnstile undefined';
            var props = [];
            for (var key in turnstile) {
                props.push(key + ':' + typeof turnstile[key]);
            }
            return props.join(', ');
        })()"#;
        match self.execute_javascript(turnstile_check_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: turnstile properties: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: turnstile properties error: {}", e),
        }

        // Check if our element passes instanceof checks
        let instanceof_js = r#"(function() {
            var el = document.querySelector('.cf-turnstile');
            if (!el) return 'no element';
            return JSON.stringify({
                isHTMLElement: el instanceof HTMLElement,
                isElement: el instanceof Element,
                isNode: el instanceof Node,
                constructorName: el.constructor ? el.constructor.name : 'none',
                prototypeChain: (function() {
                    var chain = [];
                    var proto = Object.getPrototypeOf(el);
                    var depth = 0;
                    while (proto && depth < 10) {
                        chain.push(proto.constructor ? proto.constructor.name : 'unknown');
                        proto = Object.getPrototypeOf(proto);
                        depth++;
                    }
                    return chain;
                })()
            });
        })()"#;
        match self.execute_javascript(instanceof_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: element instanceof checks: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: element instanceof error: {}", e),
        }

        // Try to manually trigger turnstile.render with selector string
        let render_js = r#"(function() {
            try {
                if (typeof turnstile === 'undefined') return 'turnstile undefined';
                if (typeof turnstile.render !== 'function') return 'turnstile.render not a function';

                var container = document.querySelector('.cf-turnstile');
                if (!container) return 'no container found';
                var sitekey = container.getAttribute('data-sitekey');
                if (!sitekey) return 'no sitekey';

                // Try using selector string instead of element reference
                var result = turnstile.render('.cf-turnstile', { sitekey: sitekey });
                return 'render called with selector, returned: ' + String(result);
            } catch (e) {
                return 'render error: ' + e.message + ' | stack: ' + String(e.stack).substring(0, 200);
            }
        })()"#;
        match self.execute_javascript(render_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: turnstile render result: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: turnstile render error: {}", e),
        }

        // Check container after render attempt
        let after_render_js = r#"(function() {
            var container = document.querySelector('.cf-turnstile');
            if (!container) return 'no container';
            return JSON.stringify({
                childCount: container.children ? container.children.length : -1,
                innerHTML: container.innerHTML ? container.innerHTML.substring(0, 300) : 'empty'
            });
        })()"#;
        match self.execute_javascript(after_render_js).await {
            Ok(result) => eprintln!("🛡️ CHALLENGE: after render: {}", result),
            Err(e) => eprintln!("🛡️ CHALLENGE: after render error: {}", e),
        }

        match solver.generate_interaction_js(&challenge, VIEWPORT_WIDTH, VIEWPORT_HEIGHT) {
            Ok(behavior_js) => {
                // Log the generated JS for debugging
                eprintln!("🛡️ CHALLENGE: Generated behavioral JS (first 500 chars): {}", &behavior_js[..behavior_js.len().min(500)]);

                // Wrap behavioral JS to signal completion for async wait
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
                    // Extract the body of the async IIFE - remove outer wrapper
                    behavior_js.trim_start_matches("(async function() {")
                        .trim_end_matches("})()")
                        .trim()
                );

                // Execute behavioral simulation using async wait to run event loop
                let behavior_result = if let Some(ref mut renderer) = self.renderer {
                    // Allow up to 10 seconds for behavioral simulation (has delays between events)
                    let behavior_timeout = std::time::Duration::from_millis(10000);
                    renderer.evaluate_javascript_with_async_wait(&wrapped_behavior_js, behavior_timeout, 50)
                } else {
                    // Fallback to regular execution (won't wait for async)
                    self.execute_javascript(&behavior_js).await
                };

                match behavior_result {
                    Ok(result) => eprintln!("🛡️ CHALLENGE: Behavioral simulation completed: {:?}", result),
                    Err(e) => eprintln!("🛡️ CHALLENGE: Behavioral simulation warning: {} (continuing)", e),
                }
            }
            Err(e) => {
                eprintln!("🛡️ CHALLENGE: Could not generate behavioral simulation: {} (continuing)", e);
            }
        }

        // Small additional delay to let events settle
        let settle_delay = 200 + (rand::random::<u64>() % 300);
        sleep(Duration::from_millis(settle_delay)).await;

        // Phase 3: Wait for challenge resolution
        let timeout_ms = challenge.expected_resolution_time_ms.max(10000);
        eprintln!("🛡️ CHALLENGE: Phase 3 - Waiting for resolution ({}ms timeout)...", timeout_ms);

        let wait_js = solver.generate_wait_for_resolution_js(&challenge, timeout_ms);

        // Use async wait method to properly handle setTimeout callbacks
        let wait_result = if let Some(ref mut renderer) = self.renderer {
            let timeout_duration = std::time::Duration::from_millis(timeout_ms + 1000); // Extra buffer
            let poll_interval = 100; // Poll every 100ms
            renderer.evaluate_javascript_with_async_wait(&wait_js, timeout_duration, poll_interval)
        } else {
            // Fallback to regular execution (won't work for async)
            self.execute_javascript(&wait_js).await
        };

        // Analyze the result
        match wait_result {
            Ok(result) => {
                let result_str = result.trim();
                eprintln!("🛡️ CHALLENGE: Resolution result: {}", result_str);

                // Check if resolution was successful (result contains "success": true)
                if result_str.contains("\"success\":true") || result_str.contains("success: true") || result_str == "true" {
                    eprintln!("🛡️ CHALLENGE: Resolution successful, retrying request...");

                    // Add a small delay before retrying
                    let retry_delay = 500 + (rand::random::<u64>() % 500);
                    sleep(Duration::from_millis(retry_delay)).await;

                    // Retry the original request - the cookies should now be set
                    let headers = self.create_standard_browser_headers(url);
                    let response = self.client.get(url).headers(headers).send().await?;
                    let new_content = response.text().await?;

                    // Check if we're still on a challenge page
                    if self.detect_challenge(&new_content, url).is_some() {
                        eprintln!("🛡️ CHALLENGE: Still on challenge page after resolution");
                        Err(anyhow!("Challenge could not be resolved automatically"))
                    } else {
                        eprintln!("🛡️ CHALLENGE: Successfully passed challenge!");
                        Ok(new_content)
                    }
                } else {
                    eprintln!("🛡️ CHALLENGE: Resolution indicated failure or timeout");
                    Err(anyhow!("Challenge resolution failed: {}", result_str))
                }
            }
            Err(e) => {
                eprintln!("🛡️ CHALLENGE: Resolution error: {}", e);
                Err(e)
            }
        }
    }

    /// Legacy wait method - now uses the solver's wait logic
    async fn wait_for_cloudflare_clearance(&mut self, timeout_ms: u64) -> Result<bool> {
        let solver = ChallengeSolver::new();

        // Create a generic Cloudflare challenge for the wait logic
        let challenge = DetectedChallenge {
            challenge_type: ChallengeType::CloudflareJS,
            confidence: 1.0,
            widget_selector: None,
            click_target_selector: None,
            requires_interaction: false,
            expected_resolution_time_ms: timeout_ms,
            detected_markers: vec![],
        };

        let wait_js = solver.generate_wait_for_resolution_js(&challenge, timeout_ms);

        // Use async wait method to properly handle setTimeout callbacks
        let result = if let Some(ref mut renderer) = self.renderer {
            let timeout_duration = std::time::Duration::from_millis(timeout_ms + 1000);
            let poll_interval = 100;
            renderer.evaluate_javascript_with_async_wait(&wait_js, timeout_duration, poll_interval)
        } else {
            self.execute_javascript(&wait_js).await
        };

        match result {
            Ok(result) => {
                let success = result.contains("\"success\":true") || result.contains("success: true") || result.trim() == "true";
                Ok(success)
            }
            Err(e) => Err(e)
        }
    }

    /// Navigate to URL with full control over waiting behavior
    ///
    /// # Security
    /// This function validates URLs to prevent SSRF attacks (CWE-918).
    /// Access to private IPs, localhost, and cloud metadata endpoints is blocked.
    ///
    /// # Cloudflare Handling
    /// This function automatically detects and attempts to solve Cloudflare
    /// challenge pages by executing their JavaScript verification.
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
        let mut content = response.text().await?;

        // Check for Cloudflare challenge and handle it
        let mut retry_count = 0;
        while self.is_cloudflare_challenge(&content) && retry_count < MAX_CF_RETRIES {
            retry_count += 1;
            eprintln!("🛡️ CLOUDFLARE: Challenge detected, attempt {}/{}", retry_count, MAX_CF_RETRIES);

            match self.handle_cloudflare_challenge(url, &content).await {
                Ok(new_content) => {
                    content = new_content;
                    if !self.is_cloudflare_challenge(&content) {
                        eprintln!("🛡️ CLOUDFLARE: Challenge resolved on attempt {}", retry_count);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("🛡️ CLOUDFLARE: Challenge handling failed: {}", e);
                    if retry_count >= MAX_CF_RETRIES {
                        return Err(anyhow!("Failed to bypass Cloudflare challenge after {} attempts: {}", MAX_CF_RETRIES, e));
                    }
                    // Wait before retrying
                    sleep(Duration::from_millis(1000 * retry_count as u64)).await;

                    // Refresh the page for another attempt
                    let headers = self.create_standard_browser_headers(url);
                    let response = self.client.get(url).headers(headers).send().await?;
                    content = response.text().await?;
                }
            }
        }

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

        // Check if this is a challenge page by content - if so, use trusted execution for all scripts
        let is_challenge_by_content = Self::is_challenge_content(html);
        if is_challenge_by_content {
            eprintln!("🔓 TRUSTED: Detected challenge page content - using trusted execution for all scripts");
        }

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

                // Collect ALL attributes from the HTML element for script registration
                let mut script_attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    script_attrs.insert(key.to_string(), value.to_string());
                }

                // Check if this is a trusted challenge provider script or on a challenge page
                let is_trusted = Self::is_trusted_challenge_script(&script_url) || is_challenge_by_content;
                if is_trusted {
                    eprintln!("🔓 TRUSTED: Script from trusted challenge provider or challenge page");
                }

                // Fetch the external script
                match self.fetch_external_script(&script_url).await {
                    Ok(script_content) => {
                        eprintln!("🔍 DEBUG: Fetched external script ({} chars)", script_content.len());
                        external_scripts_fetched += 1;

                        // Register the script BEFORE executing it
                        // This ensures document.scripts can find it when the script runs
                        let script_entry = ScriptEntry {
                            src: Some(script_url.clone()),
                            script_type: script_attrs.get("type").cloned(),
                            async_: is_async,
                            defer: is_defer,
                            text: script_content.clone(),
                            attributes: script_attrs.clone(),
                        };

                        if let Some(ref mut renderer) = self.renderer {
                            if let Err(e) = renderer.register_script(script_entry.clone()) {
                                eprintln!("⚠️  WARNING: Failed to register script: {}", e);
                            } else {
                                eprintln!("🔍 DEBUG: Registered script with {} attributes (including data-* attrs)", script_attrs.len());
                                // Log important attributes for debugging Turnstile
                                for (key, value) in &script_attrs {
                                    if key.starts_with("data-") {
                                        eprintln!("🔍 DEBUG: Script attribute: {} = {}", key, value);
                                    }
                                }
                            }

                            // Set document.currentScript before execution (per HTML spec)
                            if let Err(e) = renderer.set_current_script(&script_entry) {
                                eprintln!("⚠️  WARNING: Failed to set currentScript: {}", e);
                            }
                        }

                        // Execute the fetched script - use trusted execution for challenge providers/pages
                        let exec_result = if is_trusted {
                            self.execute_javascript_trusted(&script_content).await
                        } else {
                            self.execute_javascript(&script_content).await
                        };

                        // Clear document.currentScript after execution (per HTML spec)
                        if let Some(ref mut renderer) = self.renderer {
                            let _ = renderer.clear_current_script();
                        }

                        match exec_result {
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

                // Collect ALL attributes from the HTML element for script registration
                let mut script_attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    script_attrs.insert(key.to_string(), value.to_string());
                }

                // Register the inline script so it shows up in document.scripts
                let script_entry = ScriptEntry {
                    src: None,
                    script_type: script_attrs.get("type").cloned(),
                    async_: is_async,
                    defer: is_defer,
                    text: script_content.clone(),
                    attributes: script_attrs.clone(),
                };

                if let Some(ref mut renderer) = self.renderer {
                    if let Err(e) = renderer.register_script(script_entry.clone()) {
                        eprintln!("⚠️  WARNING: Failed to register inline script: {}", e);
                    }

                    // Set document.currentScript before execution (per HTML spec)
                    if let Err(e) = renderer.set_current_script(&script_entry) {
                        eprintln!("⚠️  WARNING: Failed to set currentScript for inline script: {}", e);
                    }
                }

                // Check if we're on a challenge page - use trusted execution for challenge page scripts
                let is_challenge_page = Self::is_challenge_page(&base_url) || is_challenge_by_content;
                if is_challenge_page {
                    eprintln!("🔓 TRUSTED: Inline script on challenge page");
                }

                // Execute the script through the JavaScript engine
                let exec_result = if is_challenge_page {
                    self.execute_javascript_trusted(&script_content).await
                } else {
                    self.execute_javascript(&script_content).await
                };

                // Clear document.currentScript after execution (per HTML spec)
                if let Some(ref mut renderer) = self.renderer {
                    let _ = renderer.clear_current_script();
                }

                match exec_result {
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

    /// Check if a script URL is from a trusted challenge provider.
    /// These providers need to use advanced JavaScript features (Symbol, Proxy, WebAssembly, etc.)
    /// that would normally be blocked by the security validator.
    ///
    /// Trusted domains:
    /// - challenges.cloudflare.com (Cloudflare Turnstile)
    /// - www.google.com/recaptcha (Google reCAPTCHA)
    /// - www.gstatic.com/recaptcha (Google reCAPTCHA assets)
    /// - hcaptcha.com (hCaptcha)
    fn is_trusted_challenge_script(url: &str) -> bool {
        let trusted_domains = [
            "challenges.cloudflare.com",
            "www.google.com/recaptcha",
            "www.gstatic.com/recaptcha",
            "hcaptcha.com",
            "js.hcaptcha.com",
            "newassets.hcaptcha.com",
            "cdn.jsdelivr.net/npm/turnstile", // CDN-hosted Turnstile
        ];

        for domain in &trusted_domains {
            if url.contains(domain) {
                return true;
            }
        }

        false
    }

    /// Check if the current page URL indicates a challenge page.
    /// Challenge pages need to run their scripts with elevated privileges
    /// because they use advanced JavaScript features.
    fn is_challenge_page(url: &str) -> bool {
        let challenge_indicators = [
            "challenges.cloudflare.com", // Cloudflare challenge page
            "/cdn-cgi/challenge",        // Cloudflare challenge path
            "captcha",                   // Generic captcha path
            "recaptcha",                 // Google reCAPTCHA
            "hcaptcha",                  // hCaptcha
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
    /// This is used when the URL doesn't reveal the challenge (e.g., challenge
    /// served from the target domain).
    fn is_challenge_content(content: &str) -> bool {
        let challenge_markers = [
            "challenges.cloudflare.com",
            "cf-turnstile",
            "data-cf-turnstile",
            "cf_chl_",
            "__cf_chl_",
            "ray_id",
            "cf-please-wait",
            "Checking your browser",
            "Just a moment...",
            "Enable JavaScript and cookies",
        ];

        for marker in &challenge_markers {
            if content.contains(marker) {
                return true;
            }
        }

        false
    }
}
