use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::engine::browser::InteractionResponse;

impl super::super::HeadlessWebBrowser {
    /// Type text into a form input element identified by CSS selector
    pub async fn type_text_into_element(
        &mut self,
        selector: &str,
        text: &str,
        clear_first: bool,
    ) -> Result<InteractionResponse> {
        // Debug logging for session state
        eprintln!(
            "🔍 DEBUG: type_text_into_element - current_content length: {}",
            self.current_content.len()
        );
        eprintln!(
            "🔍 DEBUG: type_text_into_element - current_url: {:?}",
            self.current_url
        );

        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        // Use JavaScript-based form field manipulation for better compatibility.
        // Encode as JS string literals so quotes, backslashes and newlines in
        // the selector or text can't break out of the string.
        let selector_js = js_string_literal(selector);
        let text_js = js_string_literal(text);

        let js_code = format!(
            r#"
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

        var element = document.querySelector({});
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
                    element.value = {};

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
                element.textContent = {};

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
                message: "Element not found: " + {},
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
"#,
            selector_js,
            if clear_first { "true" } else { "false" },
            text_js,
            if clear_first { "true" } else { "false" },
            text_js,
            selector_js
        );

        // Execute the JavaScript in the browser engine
        if let Some(ref mut renderer) = self.renderer {
            match renderer.evaluate_javascript_direct(&js_code) {
                Ok(result) => {
                    // Try to parse the result as JSON
                    if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(&result) {
                        let success = json_result
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let message = json_result
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Text entered");

                        // The JS DOM does not yet persist values across
                        // queries, so remember filled values on the Rust side
                        // for a later submit_form on this page.
                        if success {
                            let name = json_result
                                .get("element_name")
                                .and_then(|v| v.as_str())
                                .filter(|n| !n.is_empty())
                                .map(str::to_string)
                                .or_else(|| {
                                    first_match_attr(&self.current_content, selector, "name")
                                });
                            if let Some(name) = name {
                                self.record_filled_value(&name, text);
                            }
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
                            message: format!("Text input result: {}", result),
                            redirect_url: None,
                            new_content: None,
                        })
                    }
                }
                Err(e) => Err(anyhow!("Failed to execute text input JavaScript: {}", e)),
            }
        } else {
            Err(anyhow!("No JavaScript renderer available"))
        }
    }

    /// Submit a form with the provided field data.
    ///
    /// The submitted entry list follows the browser's behaviour: every
    /// successful control of the form (including hidden inputs such as CSRF
    /// tokens and pre-filled defaults) is sent, overlaid with values entered
    /// via [`type_text_into_element`](Self::type_text_into_element) on this
    /// page and finally with `form_data`.
    pub async fn submit_form(
        &mut self,
        form_selector: &str,
        form_data: HashMap<String, String>,
    ) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form_index = form_index_for_selector(&self.current_content, form_selector)?;
        self.submit_form_at_index(form_index, form_data, None).await
    }

    /// Submit the form that contains the element matching `selector`, with
    /// every field of that form (including values typed on this page).
    pub async fn submit_form_containing(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form_index = form_index_containing(&self.current_content, selector)?;
        self.submit_form_at_index(form_index, HashMap::new(), None)
            .await
    }

    /// Submit the `form_index`-th `<form>` of the current page, optionally on
    /// behalf of a submit button `(name, value)`.
    pub(crate) async fn submit_form_at_index(
        &mut self,
        form_index: usize,
        form_data: HashMap<String, String>,
        submitter: Option<(String, String)>,
    ) -> Result<InteractionResponse> {
        let current_url = self
            .current_url
            .clone()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        let (action, method, mut entries) = {
            let document = scraper::Html::parse_document(&self.current_content);
            let form_element = document
                .select(&FORM_SELECTOR)
                .nth(form_index)
                .ok_or_else(|| anyhow!("Form not found"))?;
            (
                form_element
                    .value()
                    .attr("action")
                    .unwrap_or("")
                    .to_string(),
                form_element
                    .value()
                    .attr("method")
                    .unwrap_or("get")
                    .to_lowercase(),
                collect_form_entries(form_element),
            )
        };

        // Resolve the action against the page URL (empty action = this page).
        let mut form_url = url::Url::parse(&current_url)?.join(&action)?;
        form_url.set_fragment(None);

        let filled = self.filled_values_for_current_page();
        merge_entries(&mut entries, filled);
        merge_entries(&mut entries, form_data);
        if let Some(submitter) = submitter {
            entries.push(submitter);
        }

        // Submit the form
        let response = if method == "post" {
            self.client
                .post(form_url.as_str())
                .form(&entries)
                .send()
                .await?
        } else {
            // GET replaces the action's query string with the form data
            form_url.set_query(None);
            self.client
                .get(form_url.as_str())
                .query(&entries)
                .send()
                .await?
        };

        let status_code = response.status();
        let final_url = response.url().to_string();
        let content = response.text().await?;

        // Update current content if successful
        if status_code.is_success() {
            self.current_content = content.clone();
            self.current_url = Some(final_url.clone());
            self.filled_values.clear();

            // Report where we ended up if the server redirected us
            let redirect_url = if final_url != form_url.as_str() {
                Some(final_url)
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

    /// Remember a value typed into the field `name` on the current page.
    pub(crate) fn record_filled_value(&mut self, name: &str, value: &str) {
        if self.filled_values_url != self.current_url {
            self.filled_values.clear();
            self.filled_values_url = self.current_url.clone();
        }
        self.filled_values
            .insert(name.to_string(), value.to_string());
    }

    /// Values typed into fields of the current page (empty after navigation).
    fn filled_values_for_current_page(&self) -> HashMap<String, String> {
        if self.filled_values_url == self.current_url {
            self.filled_values.clone()
        } else {
            HashMap::new()
        }
    }

    /// Click on a form element (checkbox, submit button, etc.) using CSS selector
    pub async fn click_element(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        eprintln!(
            "🔍 DEBUG: click_element - attempting to click selector: {}",
            selector
        );
        eprintln!(
            "🔍 DEBUG: click_element - current_content length: {}",
            self.current_content.len()
        );

        // Wait for element to appear in DOM (5 second timeout)
        let element_found = self.wait_for_element(selector, 5000).await?;
        if !element_found {
            return Err(anyhow!("Element not found after waiting: {}", selector));
        }

        // Use JavaScript-based element interaction for better compatibility
        let selector_js = js_string_literal(selector);

        let js_code = format!(
            r#"
(function() {{
    try {{
        var defaultPrevented = false;
        var element = document.querySelector({});
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
                        var clickEvent = new Event('click', {{ bubbles: true, cancelable: true }});
                        if (element.dispatchEvent(clickEvent) === false || clickEvent.defaultPrevented === true) {{
                            defaultPrevented = true;
                        }}
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
                        default_prevented: defaultPrevented,
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
                        default_prevented: defaultPrevented,
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
                        var clickEvent = new Event('click', {{ bubbles: true, cancelable: true }});
                        if (element.dispatchEvent(clickEvent) === false || clickEvent.defaultPrevented === true) {{
                            defaultPrevented = true;
                        }}
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
                    default_prevented: defaultPrevented,
                    event_dispatch_successful: eventDispatchSuccessful,
                    event_errors: eventErrors
                }});
            }}
        }} else {{
            return JSON.stringify({{
                success: false,
                message: "Element not found: " + {},
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
"#,
            selector_js, selector_js
        );

        // Execute the JavaScript in the browser engine
        if let Some(ref mut renderer) = self.renderer {
            eprintln!("🔍 DEBUG: click_element - executing JavaScript to click element");
            match renderer.evaluate_javascript_direct(&js_code) {
                Ok(result) => {
                    eprintln!("🔍 DEBUG: click_element - JavaScript result: {}", result);

                    // Try to parse the result as JSON
                    if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(&result) {
                        let success = json_result
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let message = json_result
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Element clicked");
                        let default_prevented = json_result
                            .get("default_prevented")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        // Perform the click's default action (form submission
                        // or link navigation) unless a handler prevented it.
                        if success && !default_prevented {
                            match click_default_action(
                                &self.current_content,
                                self.current_url.as_deref(),
                                selector,
                            ) {
                                ClickDefaultAction::SubmitForm {
                                    form_index,
                                    submitter,
                                } => {
                                    let mut resp = self
                                        .submit_form_at_index(form_index, HashMap::new(), submitter)
                                        .await?;
                                    resp.message = format!("{}; {}", message, resp.message);
                                    return Ok(resp);
                                }
                                ClickDefaultAction::FollowLink(url) => {
                                    let content =
                                        self.navigate_to_with_js_option(&url, false, true).await?;
                                    return Ok(InteractionResponse {
                                        success: true,
                                        message: format!("{}; navigated to {}", message, url),
                                        redirect_url: Some(url),
                                        new_content: Some(content),
                                    });
                                }
                                ClickDefaultAction::None => {}
                            }
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
                    eprintln!(
                        "🔍 DEBUG: click_element - JavaScript execution error: {}",
                        e
                    );
                    Err(anyhow!("Failed to execute element click JavaScript: {}", e))
                }
            }
        } else {
            Err(anyhow!("No JavaScript renderer available"))
        }
    }
}

static FORM_SELECTOR: std::sync::LazyLock<scraper::Selector> =
    std::sync::LazyLock::new(|| scraper::Selector::parse("form").expect("valid selector"));
static CONTROL_SELECTOR: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(|| {
    scraper::Selector::parse("input, select, textarea").expect("valid selector")
});
static OPTION_SELECTOR: std::sync::LazyLock<scraper::Selector> =
    std::sync::LazyLock::new(|| scraper::Selector::parse("option").expect("valid selector"));

/// Encode `s` as a JavaScript string literal (a JSON string is a valid JS
/// string literal), so it can be interpolated into generated scripts safely.
pub(super) fn js_string_literal(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// Attribute `attr` of the first element matching `selector` in `html`.
fn first_match_attr(html: &str, selector: &str, attr: &str) -> Option<String> {
    let selector = scraper::Selector::parse(selector).ok()?;
    let document = scraper::Html::parse_document(html);
    document
        .select(&selector)
        .next()?
        .value()
        .attr(attr)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// Index (among all `<form>` elements) of the first form matching `selector`.
fn form_index_for_selector(html: &str, selector: &str) -> Result<usize> {
    let document = scraper::Html::parse_document(html);
    let selector =
        scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid form selector"))?;
    let target = document
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow!("Form not found"))?;
    document
        .select(&FORM_SELECTOR)
        .position(|form| form == target)
        .ok_or_else(|| anyhow!("Form not found: selector does not match a <form> element"))
}

/// Index (among all `<form>` elements) of the form containing the first
/// element matching `selector`.
fn form_index_containing(html: &str, selector: &str) -> Result<usize> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid selector"))?;
    let target = document
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow!("Element not found"))?;
    let form = target
        .ancestors()
        .filter_map(scraper::ElementRef::wrap)
        .find(|a| a.value().name() == "form")
        .ok_or_else(|| anyhow!("Element is not inside a <form>"))?;
    document
        .select(&FORM_SELECTOR)
        .position(|f| f == form)
        .ok_or_else(|| anyhow!("Form not found"))
}

/// Collect a form's successful controls with their values from the HTML,
/// approximating the HTML spec's "constructing the entry list": disabled
/// controls, buttons and unchecked checkboxes/radios are skipped.
pub(crate) fn collect_form_entries(form: scraper::ElementRef) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for control in form.select(&CONTROL_SELECTOR) {
        let el = control.value();
        let Some(name) = el.attr("name").filter(|n| !n.is_empty()) else {
            continue;
        };
        if el.attr("disabled").is_some() {
            continue;
        }
        match el.name() {
            "input" => {
                let input_type = el.attr("type").unwrap_or("text").to_ascii_lowercase();
                match input_type.as_str() {
                    "submit" | "image" | "button" | "reset" | "file" => {}
                    "checkbox" | "radio" => {
                        if el.attr("checked").is_some() {
                            let value = el.attr("value").unwrap_or("on");
                            entries.push((name.to_string(), value.to_string()));
                        }
                    }
                    _ => {
                        let value = el.attr("value").unwrap_or("");
                        entries.push((name.to_string(), value.to_string()));
                    }
                }
            }
            "textarea" => entries.push((name.to_string(), control.text().collect())),
            "select" => {
                let options: Vec<_> = control.select(&OPTION_SELECTOR).collect();
                let multiple = el.attr("multiple").is_some();
                let selected: Vec<_> = options
                    .iter()
                    .filter(|o| o.value().attr("selected").is_some())
                    .collect();
                let chosen: Vec<_> = if selected.is_empty() && !multiple {
                    options.first().into_iter().collect()
                } else if multiple {
                    selected
                } else {
                    selected.into_iter().take(1).collect()
                };
                for option in chosen {
                    let value = option
                        .value()
                        .attr("value")
                        .map(str::to_string)
                        .unwrap_or_else(|| option.text().collect::<String>().trim().to_string());
                    entries.push((name.to_string(), value));
                }
            }
            _ => {}
        }
    }
    entries
}

/// Overlay `overrides` onto `entries`: an existing entry with the same name
/// takes the new value, otherwise the pair is appended.
pub(crate) fn merge_entries(
    entries: &mut Vec<(String, String)>,
    overrides: impl IntoIterator<Item = (String, String)>,
) {
    for (name, value) in overrides {
        if let Some(entry) = entries.iter_mut().find(|(n, _)| *n == name) {
            entry.1 = value;
        } else {
            entries.push((name, value));
        }
    }
}

/// What a click on an element does by default, absent `preventDefault()`.
#[derive(Debug, PartialEq)]
pub(crate) enum ClickDefaultAction {
    /// Submit the `form_index`-th form, with the button's name/value if any.
    SubmitForm {
        form_index: usize,
        submitter: Option<(String, String)>,
    },
    /// Navigate to an absolute http(s) URL.
    FollowLink(String),
    None,
}

/// Work out the default action of clicking the first element matching
/// `selector` in `html`: submit buttons submit their form, links navigate.
pub(crate) fn click_default_action(
    html: &str,
    page_url: Option<&str>,
    selector: &str,
) -> ClickDefaultAction {
    let Ok(selector) = scraper::Selector::parse(selector) else {
        return ClickDefaultAction::None;
    };
    let document = scraper::Html::parse_document(html);
    let Some(target) = document.select(&selector).next() else {
        return ClickDefaultAction::None;
    };

    // Walk from the target up through its ancestors: the first submit button
    // or link found decides (e.g. a <span> inside an <a> follows the link).
    let mut node = Some(target);
    while let Some(el) = node {
        let v = el.value();
        let is_submit = match v.name() {
            "button" => v
                .attr("type")
                .is_none_or(|t| t.eq_ignore_ascii_case("submit")),
            "input" => v.attr("type").is_some_and(|t| {
                t.eq_ignore_ascii_case("submit") || t.eq_ignore_ascii_case("image")
            }),
            _ => false,
        };
        if is_submit {
            if v.attr("disabled").is_some() {
                return ClickDefaultAction::None;
            }
            let form = el
                .ancestors()
                .filter_map(scraper::ElementRef::wrap)
                .find(|a| a.value().name() == "form");
            let Some(form) = form else {
                return ClickDefaultAction::None;
            };
            let Some(form_index) = document.select(&FORM_SELECTOR).position(|f| f == form) else {
                return ClickDefaultAction::None;
            };
            let submitter = v
                .attr("name")
                .filter(|n| !n.is_empty())
                .map(|name| (name.to_string(), v.attr("value").unwrap_or("").to_string()));
            return ClickDefaultAction::SubmitForm {
                form_index,
                submitter,
            };
        }
        if v.name() == "a"
            && let Some(href) = v.attr("href")
        {
            let href = href.trim();
            if href.is_empty()
                || href.starts_with('#')
                || href.to_ascii_lowercase().starts_with("javascript:")
            {
                return ClickDefaultAction::None;
            }
            let resolved = match page_url.map(url::Url::parse) {
                Some(Ok(base)) => base.join(href),
                _ => url::Url::parse(href),
            };
            return match resolved {
                Ok(u) if u.scheme() == "http" || u.scheme() == "https" => {
                    ClickDefaultAction::FollowLink(u.to_string())
                }
                _ => ClickDefaultAction::None,
            };
        }
        node = el.parent().and_then(scraper::ElementRef::wrap);
    }
    ClickDefaultAction::None
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOGIN: &str = r##"
        <form id="search" action="/search"><input name="q"></form>
        <form id="login" action="/login" method="post">
            <input type="hidden" name="csrf" value="tok123">
            <input name="user" value="prefilled">
            <input type="password" name="pass">
            <input type="checkbox" name="remember" value="yes" checked>
            <input type="checkbox" name="news">
            <input name="off" value="x" disabled>
            <select name="lang"><option value="en">English</option><option value="fr" selected>French</option></select>
            <textarea name="bio">hello</textarea>
            <button name="action" value="signin" id="go"><span id="label">Sign in</span></button>
            <button type="button" id="noop">Help</button>
        </form>
        <a href="/about" id="about"><b id="bold">About</b></a>
        <a href="javascript:void(0)" id="js">JS</a>
        <a href="#top" id="frag">Top</a>
    "##;

    fn login_form_entries() -> Vec<(String, String)> {
        let doc = scraper::Html::parse_document(LOGIN);
        let form = doc.select(&FORM_SELECTOR).nth(1).unwrap();
        collect_form_entries(form)
    }

    #[test]
    fn collects_successful_controls() {
        let entries = login_form_entries();
        let pairs: Vec<(&str, &str)> = entries
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("csrf", "tok123"),
                ("user", "prefilled"),
                ("pass", ""),
                ("remember", "yes"),
                ("lang", "fr"),
                ("bio", "hello"),
            ]
        );
    }

    #[test]
    fn merge_overrides_and_appends() {
        let mut entries = login_form_entries();
        merge_entries(
            &mut entries,
            [
                ("pass".to_string(), "s3cret".to_string()),
                ("extra".to_string(), "1".to_string()),
            ],
        );
        assert!(entries.contains(&("pass".to_string(), "s3cret".to_string())));
        assert_eq!(
            entries.last().unwrap(),
            &("extra".to_string(), "1".to_string())
        );
        assert!(entries.contains(&("csrf".to_string(), "tok123".to_string())));
    }

    #[test]
    fn form_index_containing_lookup() {
        assert_eq!(form_index_containing(LOGIN, "input[name=pass]").unwrap(), 1);
        assert_eq!(form_index_containing(LOGIN, "input[name=q]").unwrap(), 0);
        assert!(form_index_containing(LOGIN, "#about").is_err());
    }

    #[test]
    fn form_index_lookup() {
        assert_eq!(form_index_for_selector(LOGIN, "#login").unwrap(), 1);
        assert_eq!(form_index_for_selector(LOGIN, "form").unwrap(), 0);
        assert!(form_index_for_selector(LOGIN, "#about").is_err());
        assert!(form_index_for_selector(LOGIN, "#missing").is_err());
    }

    #[test]
    fn click_default_actions() {
        let url = Some("https://example.com/dir/page");
        assert_eq!(
            click_default_action(LOGIN, url, "#label"),
            ClickDefaultAction::SubmitForm {
                form_index: 1,
                submitter: Some(("action".to_string(), "signin".to_string())),
            }
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#noop"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#bold"),
            ClickDefaultAction::FollowLink("https://example.com/about".to_string())
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#js"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#frag"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#missing"),
            ClickDefaultAction::None
        );
    }

    #[test]
    fn js_string_literal_escapes() {
        assert_eq!(js_string_literal(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(js_string_literal("line\nbreak"), r#""line\nbreak""#);
    }
}
