use anyhow::{Result, anyhow};
use rand;
use tokio::time::{Duration, sleep};

use crate::engine::browser::{InteractionResponse, types::ScrapedData};

impl super::super::HeadlessWebBrowser {
    /// Navigate to URL (convenience method)
    pub async fn navigate_to(&mut self, url: &str) -> Result<String> {
        self.navigate_to_with_options(url, false).await
    }

    /// Navigate to URL.
    /// Page scripts run; `wait_for_js` lets the page's timers and requests
    /// run for up to 5 s (instead of 1 s) before returning.
    /// `wait_for_load` is always true (we always want to wait for the HTTP response).
    pub async fn navigate_to_with_options(
        &mut self,
        url: &str,
        wait_for_js: bool,
    ) -> Result<String> {
        self.navigate_to_with_js_option(url, true, wait_for_js)
            .await
    }

    /// The current page's title (`<title>` text, trimmed; `None` if empty).
    pub(super) fn page_title(&self) -> Option<String> {
        self.page.with_doc(|doc| {
            let title = doc.query_selector(doc.document(), "title").ok()??;
            let text = crate::engine::browser::scraper::joined_text(doc, title);
            Some(text.trim().to_string()).filter(|t| !t.is_empty())
        })
    }

    /// Scrape the current page content
    pub async fn scrape_current_page(&mut self) -> Result<ScrapedData> {
        let current_url = self
            .page
            .url()
            .ok_or_else(|| anyhow!("No current page loaded"))?;
        self.page
            .with_doc(|doc| self.scraper.scrape_page(doc, current_url))
    }

    /// Reload the current page, clearing the resource cache to force re-fetching.
    pub async fn reload(&mut self) -> Result<String> {
        if let Some(ref url) = self.get_current_url() {
            self.resource_cache.clear();
            self.bypass_cache = true;
            let result = self.navigate_to_with_options(url, false).await;
            self.bypass_cache = false;
            result
        } else {
            Err(anyhow!("No current page to reload"))
        }
    }

    /// Click a link on the current page
    pub async fn click_link(&mut self, link_text: &str) -> Result<InteractionResponse> {
        let current_url = self
            .page
            .url()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        // Scrape the current page to find the link
        let scraped_data = self
            .page
            .with_doc(|doc| self.scraper.scrape_page(doc, current_url))?;

        let target_link = scraped_data
            .links
            .iter()
            .find(|link| link.text.contains(link_text) || link.url.contains(link_text))
            .ok_or_else(|| anyhow!("Link not found: {}", link_text))?;

        // Navigate to the link
        let content = self
            .navigate_to_with_options(&target_link.url, false)
            .await?;

        Ok(InteractionResponse {
            success: true,
            message: format!("Navigated to: {}", target_link.url),
            redirect_url: Some(target_link.url.clone()),
            new_content: Some(content),
        })
    }
}
