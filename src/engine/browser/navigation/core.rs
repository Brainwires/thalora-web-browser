use anyhow::{Result, anyhow};
use tokio::time::{sleep, Duration};
use rand;

use crate::engine::browser::{InteractionResponse, types::ScrapedData};

impl super::super::HeadlessWebBrowser {
    /// Navigate to URL (convenience method)
    pub async fn navigate_to(&mut self, url: &str) -> Result<String> {
        self.navigate_to_with_options(url, false).await
    }

    /// Navigate to URL with options for JavaScript execution
    pub async fn navigate_to_with_options(&mut self, url: &str, wait_for_js: bool) -> Result<String> {
        self.navigate_to_with_js_option(url, wait_for_js, wait_for_js).await
    }

    /// Navigate to URL with explicit timeout for pending async jobs
    pub async fn navigate_to_with_timeout(&mut self, url: &str, wait_for_js: bool, wait_timeout_ms: u64) -> Result<String> {
        // Store the timeout so navigate_to_with_js_option can use it
        self.pending_jobs_timeout_ms = Some(wait_timeout_ms);
        let result = self.navigate_to_with_js_option(url, wait_for_js, wait_for_js).await;
        self.pending_jobs_timeout_ms = None;
        result
    }

    /// Extract page title from HTML
    pub(super) fn extract_title(&self, html: &str) -> Option<String> {
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

    /// Scrape the current page content
    pub async fn scrape_current_page(&mut self) -> Result<ScrapedData> {
        let current_url = self.current_url.as_ref()
            .ok_or_else(|| anyhow!("No current page loaded"))?;
        self.scraper.scrape_page(&self.current_content, current_url)
    }

    /// Reload the current page
    pub async fn reload(&mut self) -> Result<String> {
        if let Some(ref url) = self.current_url.clone() {
            self.navigate_to_with_options(url, false).await
        } else {
            Err(anyhow!("No current page to reload"))
        }
    }

    /// Click a link on the current page
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

}
