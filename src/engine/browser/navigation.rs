use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::time::sleep;
use std::time::Duration;
use url::Url;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, CONNECTION, UPGRADE_INSECURE_REQUESTS};
use crate::engine::browser::core::HeadlessWebBrowser;
use crate::engine::browser::types::{ScrapedData, InteractionResponse};

impl HeadlessWebBrowser {
    pub async fn navigate_to(&mut self, url: &str) -> Result<String> {
        self.navigate_to_with_options(url, false).await
    }

    pub async fn navigate_to_with_options(&mut self, url: &str, wait_for_js: bool) -> Result<String> {
        let headers = self.create_standard_browser_headers(url);

        let response = self.client
            .get(url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let content = response.text().await?;

        // Store current state
        self.current_url = Some(url.to_string());
        self.current_content = content.clone();

        // Update the document's HTML content in the JavaScript context
        if let Some(ref mut renderer) = self.renderer {
            let _ = renderer.update_document_html(&content);
        }

        // Add to history
        let title = self.extract_title(&content).unwrap_or_else(|| url.to_string());
        self.add_to_history(url.to_string(), title);

        // Execute JavaScript and wait for dynamic content if requested
        if wait_for_js {
            self.wait_for_page_ready().await?;
        } else {
            // Execute any safe JavaScript on the page
            if let Some(js_code) = self.extract_safe_javascript(&content) {
                if let Some(ref mut renderer) = self.renderer {
                    if renderer.is_safe_javascript(&js_code) {
                        let _ = renderer.evaluate_javascript(&js_code);
                    }
                }
            }
        }

        Ok(self.current_content.clone())
    }

    async fn wait_for_page_ready(&mut self) -> Result<()> {
        // First, extract ALL JavaScript that needs to be executed
        let content_copy = self.current_content.clone();
        let inline_js = self.extract_safe_javascript(&content_copy);

        // Extract external script URLs
        let mut external_scripts = Vec::new();
        if let Ok(selector) = scraper::Selector::parse("script[src]") {
            let document = scraper::Html::parse_document(&content_copy);
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    if self.is_safe_script_url(src) {
                        external_scripts.push(src.to_string());
                    }
                }
            }
        }

        if let Some(ref mut renderer) = self.renderer {
            // Execute inline JavaScript
            if let Some(js_code) = inline_js {
                if renderer.is_safe_javascript(&js_code) {
                    let _ = renderer.evaluate_javascript(&js_code);
                }
            }

            // Load and execute external scripts
            for script_url in external_scripts {
                if let Ok(script_response) = self.client.get(&script_url).send().await {
                    if let Ok(script_content) = script_response.text().await {
                        if renderer.is_safe_javascript(&script_content) {
                            let _ = renderer.evaluate_javascript(&script_content);
                        }
                    }
                }
            }

            // Wait for DOM to be ready with more attempts
            let ready_check = r#"
                (function() {
                    try {
                        return document.readyState === 'complete' ||
                               document.readyState === 'interactive';
                    } catch(e) {
                        return true;
                    }
                })()
            "#;

            for _ in 0..20 { // Try up to 20 times (10 seconds total)
                if let Ok(result) = renderer.evaluate_javascript(ready_check) {
                    if result.contains("true") {
                        break;
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }

            // Give dynamic content more time to load and execute
            sleep(Duration::from_millis(3000)).await;

            // Try to trigger any lazy loading or dynamic content
            let trigger_dynamic = r#"
                (function() {
                    try {
                        // Trigger scroll events that might load content
                        window.dispatchEvent(new Event('scroll'));
                        window.dispatchEvent(new Event('resize'));

                        // Trigger DOMContentLoaded if not already fired
                        if (document.readyState === 'loading') {
                            document.dispatchEvent(new Event('DOMContentLoaded'));
                        }

                        return 'triggered';
                    } catch(e) {
                        return 'error: ' + e.message;
                    }
                })()
            "#;
            let _ = renderer.evaluate_javascript(trigger_dynamic);

            // Wait a bit more after triggering events
            sleep(Duration::from_millis(2000)).await;

            // Update current content with any dynamic changes
            let updated_content = self.get_dynamic_content().await?;
            if !updated_content.is_empty() {
                self.current_content = updated_content;
            }
        }

        Ok(())
    }

    async fn get_dynamic_content(&mut self) -> Result<String> {
        if let Some(ref mut renderer) = self.renderer {
            let get_html_script = r#"
                (function() {
                    try {
                        // Try multiple ways to get the dynamic content
                        var html = '';

                        if (document.documentElement && document.documentElement.outerHTML) {
                            html = document.documentElement.outerHTML;
                        } else if (document.body && document.body.innerHTML) {
                            html = '<html><head></head><body>' + document.body.innerHTML + '</body></html>';
                        } else if (document.getElementsByTagName) {
                            var bodyTags = document.getElementsByTagName('body');
                            if (bodyTags.length > 0) {
                                html = '<html><head></head><body>' + bodyTags[0].innerHTML + '</body></html>';
                            }
                        }

                        // Also try to capture any dynamically created content
                        if (html.length < 1000) { // If we didn't get much, try harder
                            var allElements = document.querySelectorAll('*');
                            var content = '';
                            for (var i = 0; i < allElements.length && i < 100; i++) {
                                if (allElements[i].outerHTML) {
                                    content += allElements[i].outerHTML + '\n';
                                }
                            }
                            if (content.length > html.length) {
                                html = content;
                            }
                        }

                        return html || '';
                    } catch(e) {
                        return 'Error getting dynamic content: ' + e.message;
                    }
                })()
            "#;

            match renderer.evaluate_javascript(get_html_script) {
                Ok(result) => {
                    // Clean up the result - remove "JavaScript result (string): " prefix and handle quotes
                    let cleaned = result
                        .replace("JavaScript result (string): ", "")
                        .trim()
                        .trim_matches('"')
                        .to_string();

                    if cleaned.len() > 100 && (cleaned.contains("<") || cleaned.contains("Error getting dynamic content:")) {
                        return Ok(cleaned);
                    }
                }
                Err(_) => {}
            }
        }

        Ok(String::new())
    }

    pub async fn go_back(&mut self) -> Result<Option<String>> {
        if !self.can_go_back() {
            return Ok(None);
        }
        self.history.current_index -= 1;
        // Clone URL to avoid borrowing self while calling navigate_to which mutably borrows self
        let entry_url = self.history.entries[self.history.current_index].url.clone();
        let content = self.navigate_to(&entry_url).await?;
        Ok(Some(content))
    }

    pub async fn go_forward(&mut self) -> Result<Option<String>> {
        if !self.can_go_forward() {
            return Ok(None);
        }
        self.history.current_index += 1;
        let entry_url = self.history.entries[self.history.current_index].url.clone();
        let content = self.navigate_to(&entry_url).await?;
        Ok(Some(content))
    }

    pub async fn reload(&mut self) -> Result<String> {
        if let Some(url) = &self.current_url.clone() {
            self.navigate_to(url).await
        } else {
            Err(anyhow!("No current URL to reload"))
        }
    }

    pub async fn scrape_current_page(&self) -> Result<ScrapedData> {
        let url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page to scrape"))?;

        self.scraper.scrape_page(&self.current_content, url)
    }

    pub async fn submit_form(&mut self, form_selector: &str, form_data: HashMap<String, String>) -> Result<InteractionResponse> {
        let current_url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page loaded"))?;


        // Parse the current page to find the form
        let document = scraper::Html::parse_document(&self.current_content);
        let forms = self.scraper.extract_forms(&document, current_url)?;

        if forms.is_empty() {
            return Err(anyhow!("No forms found on the current page"));
        }

        // Use the first form for now (in a real implementation, would use form_selector)
        let form = &forms[0];

        // Build form data
        let mut form_params = reqwest::multipart::Form::new();
        for (key, value) in &form_data {
            form_params = form_params.text(key.clone(), value.clone());
        }

        let headers = self.create_standard_browser_headers(&form.action);

        let response = if form.method.to_uppercase() == "POST" {
            self.client
                .post(&form.action)
                .headers(headers)
                .multipart(form_params)
                .send()
                .await?
        } else {
            // For GET forms, convert to query parameters
            let mut url = Url::parse(&form.action)?;
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in &form_data {
                query_pairs.append_pair(key, value);
            }
            drop(query_pairs);

            self.client
                .get(url.as_str())
                .headers(headers)
                .send()
                .await?
        };

        let status_code = response.status();
        let content = response.text().await?;

        if status_code.is_success() {
            // Update current content if successful
            self.current_content = content.clone();

            // Check for redirect
            let redirect_url = if status_code.is_redirection() {
                Some(form.action.clone())
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

    pub async fn click_link(&mut self, link_text: &str) -> Result<InteractionResponse> {
        let current_url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        // Parse current page to find the link
        let scraped_data = self.scraper.scrape_page(&self.current_content, current_url)?;

        let target_link = scraped_data.links.iter()
            .find(|link| link.text.contains(link_text) || link.url.contains(link_text))
            .ok_or_else(|| anyhow!("Link not found: {}", link_text))?;

        // Navigate to the link
        let content = self.navigate_to(&target_link.url).await?;

        Ok(InteractionResponse {
            success: true,
            message: format!("Navigated to: {}", target_link.url),
            redirect_url: Some(target_link.url.clone()),
            new_content: Some(content),
        })
    }

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

    fn extract_safe_javascript(&self, html: &str) -> Option<String> {
        // Extract ALL inline JavaScript from script tags and combine them
        if let Ok(selector) = scraper::Selector::parse("script:not([src])") {
            let document = scraper::Html::parse_document(html);
            let mut all_js = Vec::new();

            for element in document.select(&selector) {
                let js_content = element.text().collect::<String>();
                if !js_content.trim().is_empty() {
                    all_js.push(js_content);
                }
            }

            if !all_js.is_empty() {
                // Join all JavaScript with newlines and proper separation
                return Some(all_js.join("\n;\n"));
            }
        }
        None
    }

    fn is_safe_script_url(&self, url: &str) -> bool {
        // Allow relative URLs
        if !url.starts_with("http") {
            return true;
        }

        // Allow well-known CDNs and common script sources
        let safe_domains = [
            "cdn.jsdelivr.net",
            "cdnjs.cloudflare.com",
            "unpkg.com",
            "ajax.googleapis.com",
            "code.jquery.com",
            "stackpath.bootstrapcdn.com",
            "maxcdn.bootstrapcdn.com",
            "ajax.aspnetcdn.com",
            "cdn.socket.io",
            "d3js.org",
        ];

        // Check if current URL is available to allow same-origin scripts
        if let Some(ref current_url) = self.current_url {
            if let (Ok(current), Ok(script)) = (url::Url::parse(current_url), url::Url::parse(url)) {
                if current.host() == script.host() {
                    return true;
                }
            }
        }

        // Check against safe domains
        safe_domains.iter().any(|domain| url.contains(domain))
    }

}