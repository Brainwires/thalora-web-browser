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
        let search_url = format!("https://www.google.com/search?q={}&num={}",
                                urlencoding::encode(query),
                                num_results);

        // Simple direct search - same approach that works in our tests
        let response = reqwest::get(&search_url).await?;
        let html = response.text().await?;

        self.parse_google_search_results(&html, num_results).await
    }

    async fn parse_google_search_results(&self, html: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Try multiple selectors for Google search results
        let result_selectors = [
            "div.g",
            ".g",
            "div[data-ved]",
            ".tF2Cxc",
        ];

        for selector_str in &result_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for (index, element) in document.select(&selector).enumerate() {
                    if results.len() >= num_results {
                        break;
                    }

                    // Try to extract title and URL
                    let title = self.extract_search_result_title(&element);
                    let url = self.extract_search_result_url(&element);
                    let snippet = self.extract_search_result_snippet(&element);

                    if !title.is_empty() && !url.is_empty() {
                        results.push(SearchResult {
                            title,
                            url,
                            snippet,
                            position: results.len() + 1,
                        });
                    }
                }

                if !results.is_empty() {
                    break; // Found results with this selector
                }
            }
        }

        // If no results found with standard selectors, try a more general approach
        if results.is_empty() {
            if let Ok(link_selector) = Selector::parse("a[href]") {
                for (index, element) in document.select(&link_selector).enumerate() {
                    if results.len() >= num_results {
                        break;
                    }

                    if let Some(href) = element.value().attr("href") {
                        if href.starts_with("/url?q=") || href.starts_with("http") {
                            let url = self.clean_google_url(href);
                            let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();

                            if !title.is_empty() && !url.is_empty() && !url.contains("google.com") {
                                results.push(SearchResult {
                                    title: title.clone(),
                                    url: url.clone(),
                                    snippet: String::new(),
                                    position: results.len() + 1,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(SearchResults {
            query: "search query".to_string(),
            results,
            total_results: None,
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