use anyhow::Result;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use url::Url;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: Option<String>,
    pub search_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: usize,
}

impl McpServer {
    pub(super) async fn scrape_url(&mut self, arguments: Value) -> McpResponse {
        self.browser_tools.handle_scrape_url(arguments).await
    }

    pub(super) async fn google_search(&mut self, arguments: Value) -> McpResponse {
        let query = arguments["query"].as_str().unwrap_or("");
        let num_results = arguments["num_results"].as_u64().unwrap_or(10) as usize;

        if query.is_empty() {
            return McpResponse::error(-1, "Query parameter is required".to_string());
        }

        let num_results = num_results.min(20); // Cap at 20 results

        match self.perform_google_search(query, num_results).await {
            Ok(results) => McpResponse::success(serde_json::to_value(results).unwrap_or_default()),
            Err(e) => McpResponse::error(-1, format!("Google search failed: {}", e))
        }
    }

    async fn perform_google_search(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        // Try multiple search approaches to avoid detection
        for attempt in 0..3 {
            match self.try_google_search_approach(query, num_results, attempt).await {
                Ok(results) if !results.results.is_empty() => return Ok(results),
                Ok(_) => continue, // Empty results, try next approach
                Err(_) if attempt < 2 => continue, // Error, try next approach
                Err(e) => return Err(e), // Final attempt failed
            }
        }

        // If all approaches fail, return empty results
        Ok(SearchResults {
            query: query.to_string(),
            results: vec![],
            total_results: Some("0 results".to_string()),
            search_time: None,
        })
    }

    async fn try_google_search_approach(&mut self, query: &str, num_results: usize, approach: usize) -> Result<SearchResults> {
        let search_url = match approach {
            0 => format!("https://www.google.com/search?q={}&num={}",
                        urlencoding::encode(query), num_results),
            1 => format!("https://www.google.com/search?q={}&num={}&hl=en",
                        urlencoding::encode(query), num_results),
            _ => format!("https://www.google.com/search?q={}&start=0&num={}",
                        urlencoding::encode(query), num_results),
        };

        // Enhanced anti-detection headers
        let user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ];

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(user_agents[approach % user_agents.len()])
            .build()?;

        // Add random delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(500 + (approach * 200) as u64)).await;

        let mut request = client.get(&search_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "max-age=0")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1");

        // Add browser-specific headers based on approach
        if approach == 1 {
            request = request.header("Sec-Ch-Ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
                           .header("Sec-Ch-Ua-Mobile", "?0")
                           .header("Sec-Ch-Ua-Platform", "\"Windows\"");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Google search failed with status: {}", response.status()));
        }

        let html = response.text().await?;

        // Check if we got blocked by JavaScript detection
        if html.contains("enablejs") || html.contains("Please click") || html.contains("redirect") {
            return Err(anyhow::anyhow!("Blocked by Google bot detection"));
        }

        self.parse_google_search_results(&html, num_results).await
    }

    async fn parse_google_search_results(&self, html: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Modern Google search result selectors (2024/2025)
        let result_selectors = [
            "div[data-ved] h3",           // Most common modern selector
            ".g h3",                      // Traditional selector
            ".tF2Cxc",                    // Container for search results
            ".yuRUbf",                    // Title container
            "div.g div[data-ved]",        // Nested data-ved elements
            "[data-ved] > div > div > div > div > div > span > a", // Deep nested structure
        ];

        // First try to find results using modern selectors
        for selector_str in &result_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    let title = self.extract_modern_search_result_title(&element);
                    let url = self.extract_modern_search_result_url(&element);
                    let snippet = self.extract_modern_search_result_snippet(&element);

                    if !title.is_empty() && !url.is_empty() && !url.contains("google.com") {
                        results.push(SearchResult {
                            title,
                            url,
                            snippet,
                            position: results.len() + 1,
                        });
                    }
                }

                if !results.is_empty() {
                    break;
                }
            }
        }

        // Fallback: broader search for links with useful content
        if results.is_empty() {
            if let Ok(link_selector) = Selector::parse("a[href*='/url?q='], a[href^='http']:not([href*='google.com'])") {
                for element in document.select(&link_selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    if let Some(href) = element.value().attr("href") {
                        let url = self.clean_google_url(href);
                        let title = self.extract_link_text(&element);

                        if !title.is_empty() && !url.is_empty() && !url.contains("google.com") {
                            // Look for snippet in nearby elements
                            let snippet = self.extract_nearby_snippet(&element);

                            results.push(SearchResult {
                                title,
                                url,
                                snippet,
                                position: results.len() + 1,
                            });
                        }
                    }
                }
            }
        }

        let total_count = results.len();
        Ok(SearchResults {
            query: "search query".to_string(),
            results,
            total_results: Some(format!("{} results", total_count)),
            search_time: None,
        })
    }

    fn extract_search_result_title(&self, element: &scraper::ElementRef) -> String {
        let title_selectors = ["h3", ".LC20lb", ".DKV0Md", "a h3"];

        for selector_str in &title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title_element) = element.select(&selector).next() {
                    let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() {
                        return title;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_search_result_url(&self, element: &scraper::ElementRef) -> String {
        let link_selectors = ["a[href]", "h3 a", ".yuRUbf a"];

        for selector_str in &link_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(link_element) = element.select(&selector).next() {
                    if let Some(href) = link_element.value().attr("href") {
                        return self.clean_google_url(href);
                    }
                }
            }
        }

        String::new()
    }

    fn extract_search_result_snippet(&self, element: &scraper::ElementRef) -> String {
        let snippet_selectors = [".VwiC3b", ".s", ".st", "span.aCOpRe"];

        for selector_str in &snippet_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(snippet_element) = element.select(&selector).next() {
                    let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !snippet.is_empty() {
                        return snippet;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_modern_search_result_title(&self, element: &scraper::ElementRef) -> String {
        // Modern Google search title extraction
        let title_selectors = [
            "h3",                    // Direct h3 elements
            "a h3",                  // h3 inside links
            "[role='heading']",      // ARIA heading role
            ".LC20lb",              // Classic Google title class
            ".DKV0Md",              // Alternative title class
            "span[dir='ltr']",      // Direction-specific spans
        ];

        for selector_str in &title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title_element) = element.select(&selector).next() {
                    let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && title.len() > 3 {
                        return title;
                    }
                }
            }
        }

        // Fallback: look for any text content in the element
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if text.len() > 3 && text.len() < 200 {
            text
        } else {
            String::new()
        }
    }

    fn extract_modern_search_result_url(&self, element: &scraper::ElementRef) -> String {
        // Modern Google search URL extraction
        let link_selectors = [
            "a[href]",              // Direct links
            "a[href*='/url?q=']",   // Google redirect links
            "[data-ved] a[href]",   // Links in data-ved containers
        ];

        for selector_str in &link_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(link_element) = element.select(&selector).next() {
                    if let Some(href) = link_element.value().attr("href") {
                        let cleaned_url = self.clean_google_url(href);
                        if !cleaned_url.is_empty() && !cleaned_url.contains("google.com") {
                            return cleaned_url;
                        }
                    }
                }
            }
        }

        // Look in parent elements for links
        if let Some(parent) = element.parent() {
            if let Some(parent_element) = parent.value().as_element() {
                let parent_ref = scraper::ElementRef::wrap(parent).unwrap();
                return self.extract_modern_search_result_url(&parent_ref);
            }
        }

        String::new()
    }

    fn extract_modern_search_result_snippet(&self, element: &scraper::ElementRef) -> String {
        // Modern Google search snippet extraction
        let snippet_selectors = [
            ".VwiC3b",              // Modern snippet class
            ".s",                   // Classic snippet class
            ".st",                  // Alternative snippet class
            "span[style*='color']", // Colored text spans
            ".aCOpRe",             // Another snippet class
            "[data-content-feature='1']", // Content feature elements
        ];

        for selector_str in &snippet_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(snippet_element) = element.select(&selector).next() {
                    let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !snippet.is_empty() && snippet.len() > 10 {
                        return snippet;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_link_text(&self, element: &scraper::ElementRef) -> String {
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if text.len() > 3 && text.len() < 200 {
            text
        } else {
            String::new()
        }
    }

    fn extract_nearby_snippet(&self, element: &scraper::ElementRef) -> String {
        // Look for snippet text in sibling elements
        if let Some(parent) = element.parent() {
            if let Some(parent_element) = parent.value().as_element() {
                let parent_ref = scraper::ElementRef::wrap(parent).unwrap();

                // Check siblings for snippet-like content
                for sibling in parent_ref.children() {
                    if let Some(sibling_element) = sibling.value().as_element() {
                        let sibling_ref = scraper::ElementRef::wrap(sibling).unwrap();
                        let text = sibling_ref.text().collect::<Vec<_>>().join(" ").trim().to_string();

                        // Look for text that seems like a snippet (longer than title, but not too long)
                        if text.len() > 20 && text.len() < 300 && !text.starts_with("http") {
                            return text;
                        }
                    }
                }
            }
        }

        String::new()
    }

    fn clean_google_url(&self, url: &str) -> String {
        if url.starts_with("/url?q=") {
            // Extract the actual URL from Google's redirect URL
            if let Ok(parsed_url) = Url::parse(&format!("https://google.com{}", url)) {
                if let Some(query) = parsed_url.query() {
                    for pair in query.split('&') {
                        if let Some(q_url) = pair.strip_prefix("q=") {
                            return urlencoding::decode(q_url)
                                .unwrap_or_else(|_| q_url.into())
                                .into_owned();
                        }
                    }
                }
            }
        } else if url.starts_with("http") {
            return url.to_string();
        }

        url.to_string()
    }
}