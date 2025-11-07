use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{sleep, Duration};
use rand;

use crate::engine::browser::{InteractionResponse, types::ScrapedData};

impl super::HeadlessWebBrowser {
    /// Navigate to URL with options for JavaScript execution
    pub async fn navigate_to_with_options(&mut self, url: &str, wait_for_load: bool) -> Result<String> {
        self.navigate_to_with_js_option(url, wait_for_load, false).await
    }

    /// Navigate to URL with full control over waiting behavior
    pub async fn navigate_to_with_js_option(&mut self, url: &str, wait_for_load: bool, wait_for_js: bool) -> Result<String> {
        eprintln!("🔍 DEBUG: navigate_to_with_js_option - URL: {}, wait_for_load: {}, wait_for_js: {}", url, wait_for_load, wait_for_js);

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

        // Add human-like navigation delay
        let navigation_delay = 1000 + (rand::random::<u64>() % 2000); // 1-3 seconds
        eprintln!("🔍 DEBUG: Adding human-like navigation delay: {}ms", navigation_delay);
        sleep(Duration::from_millis(navigation_delay)).await;

        // Add processing delay to simulate page loading
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

    /// Extract page title from HTML
    fn extract_title(&self, html: &str) -> Option<String> {
        if let Ok(selector) = scraper::Selector::parse("title") {
            let document = scraper::Html::parse_document(html);
            document.select(&selector)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|t| !t.is_empty())
        } else {
            None
        }
    }

    pub async fn click_link(&mut self, link_text: &str) -> Result<InteractionResponse> {
        let current_url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        // Parse current page to find the link
        let scraped_data = self.scraper.scrape_page(&self.current_content, current_url)?;

        let target_link = scraped_data.links.iter()
            .find(|link| link.text.contains(link_text) || link.url.contains(link_text))
            .ok_or_else(|| anyhow!("Link not found: {}", link_text))?;

        // Navigate to the link
        let content = self.navigate_to_with_options(&target_link.url, false).await?;

        Ok(InteractionResponse {
            success: true,
            message: format!("Navigated to: {}", target_link.url),
            redirect_url: Some(target_link.url.clone()),
            new_content: Some(content),
        })
    }

    /// Type text into a form input element identified by CSS selector
    pub async fn type_text_into_element(&mut self, selector: &str, text: &str, clear_first: bool) -> Result<InteractionResponse> {
        // Debug logging for session state
        eprintln!("🔍 DEBUG: type_text_into_element - current_content length: {}", self.current_content.len());
        eprintln!("🔍 DEBUG: type_text_into_element - current_url: {:?}", self.current_url);

        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        // Use JavaScript-based form field manipulation for better compatibility
        // Escape quotes in CSS selector for proper JavaScript string interpolation
        let escaped_selector = selector.replace("\"", "\\\"");
        let escaped_text = text.replace("\"", "\\\"");


        let js_code = format!(r#"
(function() {{
    try {{
        // Check if document.querySelector is available
        if (typeof document === 'undefined') {{
            return JSON.stringify({{
                success: false,
                message: "Error typing text: document is undefined",
                error: "document_undefined"
            }});
        }}

        if (typeof document.querySelector !== 'function') {{
            return JSON.stringify({{
                success: false,
                message: "Error typing text: document.querySelector is not a function",
                error: "querySelector_not_function"
            }});
        }}

        var element = document.querySelector("{}");
        if (element) {{
            var elementType = element.tagName.toLowerCase();
            var isInput = (elementType === 'input' || elementType === 'textarea');

            if (isInput) {{
                // Handle input/textarea elements
                try {{
                    // Step 1: Set the value
                    if ({}) {{
                        element.value = '';
                    }}
                    element.value = "{}";

                    // Step 2: Check Event constructor availability
                    if (typeof Event === 'undefined') {{
                        return JSON.stringify({{
                            success: false,
                            message: "Error typing text: Event constructor is undefined",
                            error: "event_constructor_undefined"
                        }});
                    }}

                    if (typeof Event !== 'function') {{
                        return JSON.stringify({{
                            success: false,
                            message: "Error typing text: Event is not a function, type is: " + typeof Event,
                            error: "event_constructor_not_function"
                        }});
                    }}

                    // Step 3: Test Event constructor
                    var inputEvent;
                    try {{
                        inputEvent = new Event('input', {{ bubbles: true }});
                    }} catch (eventError) {{
                        return JSON.stringify({{
                            success: false,
                            message: "Error typing text: Event constructor failed: " + eventError.message,
                            error: "event_constructor_failed"
                        }});
                    }}

                    // Trigger input events to simulate real user interaction
                    var inputEvent = new Event('input', {{ bubbles: true }});
                    element.dispatchEvent(inputEvent);

                    var changeEvent = new Event('change', {{ bubbles: true }});
                    element.dispatchEvent(changeEvent);

                    return JSON.stringify({{
                        success: true,
                        message: "Text entered into " + elementType + " element: " + (element.name || element.id || "unnamed") + " = " + element.value,
                        element_type: elementType,
                        element_name: element.name,
                        element_value: element.value
                    }});
                }} catch (inputError) {{
                    return JSON.stringify({{
                        success: false,
                        message: "Error typing text: input handling failed: " + inputError.message,
                        error: "input_handling_failed"
                    }});
                }}
            }} else {{
                // Handle non-input elements (h1, p, div, etc.) - set textContent
                if ({}) {{
                    element.textContent = '';
                }}
                element.textContent = "{}";

                return JSON.stringify({{
                    success: true,
                    message: "Text set for " + elementType + " element: " + (element.id || element.className || "unnamed") + " = " + element.textContent,
                    element_type: elementType,
                    element_id: element.id,
                    element_value: element.textContent
                }});
            }}
        }} else {{
            return JSON.stringify({{
                success: false,
                message: "Element not found: {}",
                error: "selector_not_found"
            }});
        }}
    }} catch (error) {{
        return JSON.stringify({{
            success: false,
            message: "Error typing text: " + error.message,
            error: error.toString()
        }});
    }}
}})();
"#, escaped_selector, if clear_first { "true" } else { "false" }, escaped_text, if clear_first { "true" } else { "false" }, escaped_text, escaped_selector);

        // Execute the JavaScript in the browser engine
        if let Some(ref mut renderer) = self.renderer {
            match renderer.evaluate_javascript_direct(&js_code) {
                Ok(result) => {
                    // Try to parse the result as JSON
                    if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(&result) {
                        let success = json_result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                        let message = json_result.get("message").and_then(|v| v.as_str()).unwrap_or("Text entered");

                        Ok(InteractionResponse {
                            success,
                            message: message.to_string(),
                            redirect_url: None,
                            new_content: None,
                        })
                    } else {
                        // Fallback if result is not JSON
                        Ok(InteractionResponse {
                            success: !result.contains("error") && !result.contains("Error"),
                            message: format!("Text input result: {}", result),
                            redirect_url: None,
                            new_content: None,
                        })
                    }
                }
                Err(e) => Err(anyhow!("Failed to execute text input JavaScript: {}", e))
            }
        } else {
            Err(anyhow!("No JavaScript renderer available"))
        }
    }

    /// Submit a form with the provided field data
    pub async fn submit_form(&mut self, form_selector: &str, form_data: HashMap<String, String>) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        // Parse HTML to find the form
        let document = scraper::Html::parse_document(&self.current_content);
        let form_selector = scraper::Selector::parse(form_selector)
            .map_err(|_| anyhow!("Invalid form selector"))?;

        let form_element = document.select(&form_selector).next()
            .ok_or_else(|| anyhow!("Form not found"))?;

        let action = form_element.value().attr("action").unwrap_or("");
        let method = form_element.value().attr("method").unwrap_or("get").to_lowercase();

        let current_url = self.current_url.as_ref().unwrap();
        let form_url = if action.starts_with("http") {
            action.to_string()
        } else if action.starts_with('/') {
            let base_url = url::Url::parse(current_url)?;
            format!("{}://{}{}", base_url.scheme(), base_url.host_str().unwrap_or(""), action)
        } else {
            format!("{}/{}", current_url.trim_end_matches('/'), action.trim_start_matches('/'))
        };

        // Build form data
        let mut form_params = Vec::new();
        for (key, value) in form_data {
            form_params.push((key, value));
        }

        // Submit the form
        let response = if method == "post" {
            self.client.post(&form_url).form(&form_params).send().await?
        } else {
            self.client.get(&form_url).query(&form_params).send().await?
        };

        let status_code = response.status();
        let content = response.text().await?;

        // Update current content if successful
        if status_code.is_success() {
            self.current_content = content.clone();
            self.current_url = Some(form_url.clone());

            // Check for redirect
            let redirect_url = if status_code.is_redirection() {
                Some(form_url)
            } else {
                None
            };

            Ok(InteractionResponse {
                success: true,
                message: "Form submitted successfully".to_string(),
                redirect_url,
                new_content: Some(content),
            })
        } else {
            Ok(InteractionResponse {
                success: false,
                message: format!("Form submission failed: {}", status_code),
                redirect_url: None,
                new_content: Some(content),
            })
        }
    }

    /// Click on a form element (checkbox, submit button, etc.) using CSS selector
    pub async fn click_element(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        eprintln!("🔍 DEBUG: click_element - attempting to click selector: {}", selector);
        eprintln!("🔍 DEBUG: click_element - current_content length: {}", self.current_content.len());

        // Wait for element to appear in DOM (5 second timeout)
        let element_found = self.wait_for_element(selector, 5000).await?;
        if !element_found {
            return Err(anyhow!("Element not found after waiting: {}", selector));
        }

        // Use JavaScript-based element interaction for better compatibility
        // Escape quotes in CSS selector for proper JavaScript string interpolation
        let escaped_selector = selector.replace("\"", "\\\"");

        let js_code = format!(r#"
(function() {{
    try {{
        var element = document.querySelector("{}");
        if (element) {{
            // Handle different element types
            if (element.type === 'checkbox' || element.type === 'radio') {{
                // Toggle checkbox/radio state
                element.checked = !element.checked;

                // Try to dispatch change event, fallback gracefully
                var eventDispatchSuccessful = false;
                var eventErrors = [];

                try {{
                    if (typeof element.dispatchEvent === 'function') {{
                        var changeEvent = new Event('change', {{ bubbles: true }});
                        element.dispatchEvent(changeEvent);
                        eventDispatchSuccessful = true;
                    }} else {{
                        eventErrors.push("dispatchEvent not available (type: " + typeof element.dispatchEvent + ")");
                    }}
                }} catch (eventError) {{
                    eventErrors.push("Event dispatch failed: " + eventError.message);
                }}

                // Fallback: try onchange property
                if (!eventDispatchSuccessful) {{
                    try {{
                        if (typeof element.onchange === 'function') {{
                            element.onchange();
                            eventDispatchSuccessful = true;
                        }}
                    }} catch (propError) {{
                        eventErrors.push("onchange property trigger failed: " + propError.message);
                    }}
                }}

                var message = "Clicked " + element.type + " element: " + (element.name || "unnamed") + ", checked: " + element.checked;
                if (!eventDispatchSuccessful && eventErrors.length > 0) {{
                    message += " (Note: Event dispatching failed - " + eventErrors.join(", ") + ")";
                }}

                return JSON.stringify({{
                    success: true,
                    message: message,
                    element_type: element.type,
                    element_name: element.name,
                    element_checked: element.checked,
                    event_dispatch_successful: eventDispatchSuccessful,
                    event_errors: eventErrors
                }});
            }} else if (element.type === 'submit' || element.tagName.toLowerCase() === 'button') {{
                // For submit buttons and regular buttons
                var eventDispatchSuccessful = false;
                var eventErrors = [];

                // Try to dispatch click event
                try {{
                    if (typeof element.dispatchEvent === 'function') {{
                        var clickEvent = new Event('click', {{ bubbles: true }});
                        element.dispatchEvent(clickEvent);
                        eventDispatchSuccessful = true;
                    }} else {{
                        eventErrors.push("dispatchEvent not available (type: " + typeof element.dispatchEvent + ")");
                    }}
                }} catch (eventError) {{
                    eventErrors.push("Event dispatch failed: " + eventError.message);
                }}

                // Fallback: try onclick property
                if (!eventDispatchSuccessful) {{
                    try {{
                        if (typeof element.onclick === 'function') {{
                            element.onclick();
                            eventDispatchSuccessful = true;
                        }}
                    }} catch (propError) {{
                        eventErrors.push("onclick property trigger failed: " + propError.message);
                    }}
                }}

                // If it's a submit button, also trigger form submission
                if (element.type === 'submit' && element.form) {{
                    var message = "Clicked submit button: " + (element.value || "unnamed") + ", form will be submitted";
                    if (!eventDispatchSuccessful && eventErrors.length > 0) {{
                        message += " (Note: Event dispatching failed - " + eventErrors.join(", ") + ")";
                    }}

                    return JSON.stringify({{
                        success: true,
                        message: message,
                        element_type: element.type,
                        element_value: element.value,
                        form_action: element.form.action,
                        form_method: element.form.method,
                        submit_triggered: true,
                        event_dispatch_successful: eventDispatchSuccessful,
                        event_errors: eventErrors
                    }});
                }} else {{
                    var message = "Clicked button element: " + (element.value || element.textContent || "unnamed");
                    if (!eventDispatchSuccessful && eventErrors.length > 0) {{
                        message += " (Note: Event dispatching failed - " + eventErrors.join(", ") + ")";
                    }}

                    return JSON.stringify({{
                        success: true,
                        message: message,
                        element_type: element.type || "button",
                        element_value: element.value || element.textContent,
                        event_dispatch_successful: eventDispatchSuccessful,
                        event_errors: eventErrors
                    }});
                }}
            }} else {{
                // Generic element click
                var eventDispatchSuccessful = false;
                var eventErrors = [];

                // Try to dispatch click event
                try {{
                    if (typeof element.dispatchEvent === 'function') {{
                        var clickEvent = new Event('click', {{ bubbles: true }});
                        element.dispatchEvent(clickEvent);
                        eventDispatchSuccessful = true;
                    }} else {{
                        eventErrors.push("dispatchEvent not available (type: " + typeof element.dispatchEvent + ")");
                    }}
                }} catch (eventError) {{
                    eventErrors.push("Event dispatch failed: " + eventError.message);
                }}

                // Fallback: try onclick property
                if (!eventDispatchSuccessful) {{
                    try {{
                        if (typeof element.onclick === 'function') {{
                            element.onclick();
                            eventDispatchSuccessful = true;
                        }}
                    }} catch (propError) {{
                        eventErrors.push("onclick property trigger failed: " + propError.message);
                    }}
                }}

                var message = "Clicked element: " + element.tagName + (element.name ? " (name: " + element.name + ")" : "");
                if (!eventDispatchSuccessful && eventErrors.length > 0) {{
                    message += " (Note: Event dispatching failed - " + eventErrors.join(", ") + ")";
                }}

                return JSON.stringify({{
                    success: true,
                    message: message,
                    element_type: element.tagName.toLowerCase(),
                    element_name: element.name || null,
                    event_dispatch_successful: eventDispatchSuccessful,
                    event_errors: eventErrors
                }});
            }}
        }} else {{
            return JSON.stringify({{
                success: false,
                message: "Element not found: {}",
                error: "selector_not_found"
            }});
        }}
    }} catch (error) {{
        return JSON.stringify({{
            success: false,
            message: "Error clicking element: " + error.message,
            error: error.toString()
        }});
    }}
}})();
"#, escaped_selector, escaped_selector);

        // Execute the JavaScript in the browser engine
        if let Some(ref mut renderer) = self.renderer {
            eprintln!("🔍 DEBUG: click_element - executing JavaScript to click element");
            match renderer.evaluate_javascript_direct(&js_code) {
                Ok(result) => {
                    eprintln!("🔍 DEBUG: click_element - JavaScript result: {}", result);

                    // Try to parse the result as JSON
                    if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(&result) {
                        let success = json_result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                        let message = json_result.get("message").and_then(|v| v.as_str()).unwrap_or("Element clicked");
                        let submit_triggered = json_result.get("submit_triggered").and_then(|v| v.as_bool()).unwrap_or(false);

                        if success && submit_triggered {
                            // Handle form submission if submit button was clicked
                            eprintln!("🔍 DEBUG: click_element - submit button triggered form submission");
                            // Note: In a real browser, this would navigate to the form action URL
                            // For now, we'll return the click result and let the caller handle navigation
                        }

                        Ok(InteractionResponse {
                            success,
                            message: message.to_string(),
                            redirect_url: None,
                            new_content: None,
                        })
                    } else {
                        // Fallback if result is not JSON
                        Ok(InteractionResponse {
                            success: !result.contains("error") && !result.contains("Error"),
                            message: format!("Element interaction result: {}", result),
                            redirect_url: None,
                            new_content: None,
                        })
                    }
                }
                Err(e) => {
                    eprintln!("🔍 DEBUG: click_element - JavaScript execution error: {}", e);
                    Err(anyhow!("Failed to execute element click JavaScript: {}", e))
                }
            }
        } else {
            Err(anyhow!("No JavaScript renderer available"))
        }
    }

    /// Navigate to URL (convenience method)
    pub async fn navigate_to(&mut self, url: &str) -> Result<String> {
        self.navigate_to_with_options(url, false).await
    }

    /// Scrape the current page content
    pub async fn scrape_current_page(&mut self) -> Result<ScrapedData> {
        let current_url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page loaded"))?;
        self.scraper.scrape_page(&self.current_content, current_url)
    }

    /// Go back in navigation history
    pub async fn go_back(&mut self) -> Result<Option<String>> {
        // Simplified implementation - return None since we don't maintain history
        Ok(None)
    }

    /// Go forward in navigation history
    pub async fn go_forward(&mut self) -> Result<Option<String>> {
        // Simplified implementation - return None since we don't maintain history
        Ok(None)
    }

    /// Reload the current page
    pub async fn reload(&mut self) -> Result<String> {
        if let Some(ref url) = self.current_url.clone() {
            self.navigate_to_with_options(url, false).await
        } else {
            Err(anyhow!("No current page to reload"))
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
    async fn fire_dom_content_loaded(&mut self) -> Result<()> {
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

    /// Resolve a script URL relative to the base URL
    fn resolve_script_url(&self, base_url: &str, src: &str) -> Result<String> {
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
    async fn fetch_external_script(&self, url: &str) -> Result<String> {
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