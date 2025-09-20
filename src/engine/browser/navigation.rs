use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::time::sleep;
use std::time::Duration;
use url::Url;
use crate::engine::browser::core::HeadlessWebBrowser;
use crate::engine::browser::types::{ScrapedData, InteractionResponse};

impl HeadlessWebBrowser {
    pub async fn navigate_to(&mut self, url: &str) -> Result<String> {
        self.navigate_to_with_options(url, false).await
    }

    pub async fn navigate_to_with_options(&mut self, url: &str, wait_for_js: bool) -> Result<String> {
        self.stealth_manager.apply_random_delay().await;

        let headers = self.stealth_manager.create_stealth_headers(url);

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
        // Execute initialization scripts first
        let content_copy = self.current_content.clone();
        if let Some(js_code) = self.extract_safe_javascript(&content_copy) {
            if let Some(ref mut renderer) = self.renderer {
                if renderer.is_safe_javascript(&js_code) {
                    let _ = renderer.evaluate_javascript(&js_code);
                }
            }
        }

        if let Some(ref mut renderer) = self.renderer {

            // Wait for DOM to be ready
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

            for _ in 0..10 { // Try up to 10 times (5 seconds total)
                if let Ok(result) = renderer.evaluate_javascript(ready_check) {
                    if result.contains("true") {
                        break;
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }

            // Give dynamic content time to load
            sleep(Duration::from_millis(1000)).await;

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
                        return document.documentElement.outerHTML ||
                               document.body.innerHTML || '';
                    } catch(e) {
                        return '';
                    }
                })()
            "#;

            match renderer.evaluate_javascript(get_html_script) {
                Ok(result) => {
                    // Clean up the result - remove "JavaScript result (string): " prefix
                    let cleaned = result.replace("JavaScript result (string): ", "").trim().to_string();
                    if cleaned.len() > 100 && cleaned.contains("<") {
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

        self.stealth_manager.apply_random_delay().await;

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

        let headers = self.stealth_manager.create_stealth_headers(&form.action);

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
        // Extract inline JavaScript from script tags
        if let Ok(selector) = scraper::Selector::parse("script:not([src])") {
            let document = scraper::Html::parse_document(html);
            for element in document.select(&selector) {
                let js_content = element.text().collect::<String>();
                if !js_content.trim().is_empty() {
                    return Some(js_content);
                }
            }
        }
        None
    }
}