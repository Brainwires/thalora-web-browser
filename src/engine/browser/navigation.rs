use anyhow::{Result, anyhow};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use rand;

use crate::engine::browser::{InteractionResponse, types::ScrapedData};

impl super::HeadlessWebBrowser {
    /// Navigate to URL with options for JavaScript execution
    pub async fn navigate_to_with_options(&mut self, url: &str, wait_for_load: bool) -> Result<String> {
        eprintln!("🔍 DEBUG: navigate_to_with_options - URL: {}, wait_for_load: {}", url, wait_for_load);

        // Fetch the page content
        let response = self.client.get(url).send().await?;
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

        // Headless browser behavior: load HTML and make it ready for interaction
        // No JavaScript extraction or execution from the page
        eprintln!("🔍 DEBUG: HTML content loaded, ready for direct DOM interaction");

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
}