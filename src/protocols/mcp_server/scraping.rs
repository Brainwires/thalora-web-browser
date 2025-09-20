use anyhow::Result;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use url::Url;
use rand::{Rng, thread_rng};

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

    pub(super) async fn web_search(&mut self, arguments: Value) -> McpResponse {
        let query = arguments["query"].as_str().unwrap_or("");
        let num_results = arguments["num_results"].as_u64().unwrap_or(10) as usize;
        let search_engine = arguments["search_engine"].as_str().unwrap_or("duckduckgo");

        if query.is_empty() {
            return McpResponse::error(-1, "Query parameter is required".to_string());
        }

        let num_results = num_results.min(20); // Cap at 20 results

        match self.perform_web_search(query, num_results, search_engine).await {
            Ok(results) => McpResponse::success(serde_json::to_value(results).unwrap_or_default()),
            Err(e) => McpResponse::error(-1, format!("Web search failed: {}", e))
        }
    }

    async fn perform_web_search(&mut self, query: &str, num_results: usize, search_engine: &str) -> Result<SearchResults> {
        match search_engine {
            "duckduckgo" => self.search_duckduckgo(query, num_results).await,
            "bing" => self.search_bing(query, num_results).await,
            "startpage" => self.search_startpage(query, num_results).await,
            "searx" => self.search_searx(query, num_results).await,
            _ => self.search_duckduckgo(query, num_results).await, // Default to DuckDuckGo
        }
    }

    async fn search_duckduckgo(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));

        let browser = self.browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        let mut browser_guard = browser;

        // Navigate with JavaScript support
        browser_guard.navigate_to_with_options(&search_url, true).await?;

        // Get the rendered content
        let html = browser_guard.get_current_content();
        drop(browser_guard);

        self.parse_duckduckgo_results(&html, query, num_results).await
    }

    async fn search_bing(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        // Fall back to DuckDuckGo for now since Bing has very aggressive bot detection
        // This is a temporary workaround until we can implement proper browser automation
        eprintln!("Warning: Bing search is temporarily using DuckDuckGo fallback due to bot detection");
        return self.search_duckduckgo(query, num_results).await;
    }

    async fn search_startpage(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        let search_url = format!("https://www.startpage.com/do/search?query={}", urlencoding::encode(query));

        let browser = self.browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        let mut browser_guard = browser;

        browser_guard.navigate_to_with_options(&search_url, true).await?;
        let html = browser_guard.get_current_content();
        drop(browser_guard);

        self.parse_startpage_results(&html, query, num_results).await
    }

    async fn search_searx(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        // Use public SearX instance
        let search_url = format!("https://searx.be/search?q={}&format=html", urlencoding::encode(query));

        let browser = self.browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        let mut browser_guard = browser;

        browser_guard.navigate_to_with_options(&search_url, true).await?;
        let html = browser_guard.get_current_content();
        drop(browser_guard);

        self.parse_searx_results(&html, query, num_results).await
    }

    async fn parse_duckduckgo_results(&self, html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // DuckDuckGo HTML result selectors
        if let Ok(selector) = Selector::parse(".result__body") {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let title = self.extract_generic_title(&element, &[".result__title a", "h2 a", "h3 a"]);
                let url = self.extract_generic_url(&element, &[".result__title a", "h2 a", "h3 a"]);
                let snippet = self.extract_generic_snippet(&element, &[".result__snippet", ".result__body .snippet"]);

                if !title.is_empty() && !url.is_empty() {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet,
                        position: results.len() + 1,
                    });
                }
            }
        }

        let result_count = results.len();
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_results: Some(format!("{} results", result_count)),
            search_time: None,
        })
    }

    async fn parse_bing_results(&self, html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Multiple Bing result selectors - try various versions
        let selectors = [".b_algo", ".b_algoSlug", ".b_algoheader", ".b_title h2"];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    let title = self.extract_generic_title(&element, &["h2 a", ".b_title a", "h3 a", "a h2", "a"]);
                    let url = self.extract_generic_url(&element, &["h2 a", ".b_title a", "h3 a", "a h2", "a"]);
                    let snippet = self.extract_generic_snippet(&element, &[".b_caption p", ".b_snippet", ".b_paractl", ".b_descript", "p"]);

                    if !title.is_empty() && !url.is_empty() && !results.iter().any(|r: &SearchResult| r.url == url) {
                        results.push(SearchResult {
                            title,
                            url,
                            snippet,
                            position: results.len() + 1,
                        });
                    }
                }
            }

            if results.len() >= num_results {
                break;
            }
        }

        // If we still don't have results, try looking for any links with titles
        if results.is_empty() {
            if let Ok(selector) = Selector::parse("a[href]") {
                for element in document.select(&selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    let url = element.value().attr("href").unwrap_or("").to_string();
                    let title = element.text().collect::<String>().trim().to_string();

                    // Filter for actual search results
                    if url.starts_with("http") && !url.contains("bing.com") && !url.contains("microsoft.com")
                        && title.len() > 5 && title.len() < 200 {
                        results.push(SearchResult {
                            title,
                            url,
                            snippet: String::new(),
                            position: results.len() + 1,
                        });
                    }
                }
            }
        }

        let result_count = results.len();
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_results: Some(format!("{} results", result_count)),
            search_time: None,
        })
    }

    async fn parse_startpage_results(&self, html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Startpage result selectors
        if let Ok(selector) = Selector::parse(".result") {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let title = self.extract_generic_title(&element, &[".result-title a", "h3 a", "h2 a"]);
                let url = self.extract_generic_url(&element, &[".result-title a", "h3 a", "h2 a"]);
                let snippet = self.extract_generic_snippet(&element, &[".result-desc", ".snippet"]);

                if !title.is_empty() && !url.is_empty() {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet,
                        position: results.len() + 1,
                    });
                }
            }
        }

        let result_count = results.len();
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_results: Some(format!("{} results", result_count)),
            search_time: None,
        })
    }

    async fn parse_searx_results(&self, html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // SearX result selectors
        if let Ok(selector) = Selector::parse(".result") {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let title = self.extract_generic_title(&element, &[".result_title a", "h3 a", "h2 a"]);
                let url = self.extract_generic_url(&element, &[".result_title a", "h3 a", "h2 a"]);
                let snippet = self.extract_generic_snippet(&element, &[".result_content", ".content"]);

                if !title.is_empty() && !url.is_empty() {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet,
                        position: results.len() + 1,
                    });
                }
            }
        }

        let result_count = results.len();
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_results: Some(format!("{} results", result_count)),
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

    fn extract_generic_title(&self, element: &scraper::ElementRef, selectors: &[&str]) -> String {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title_element) = element.select(&selector).next() {
                    let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && title.len() > 3 {
                        return title;
                    }
                }
            }
        }

        // Fallback: look for any heading in the element
        let fallback_selectors = ["h1", "h2", "h3", "h4", "h5", "h6"];
        for selector_str in &fallback_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title_element) = element.select(&selector).next() {
                    let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && title.len() > 3 {
                        return title;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_generic_url(&self, element: &scraper::ElementRef, selectors: &[&str]) -> String {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(link_element) = element.select(&selector).next() {
                    if let Some(href) = link_element.value().attr("href") {
                        let cleaned_url = self.clean_url(href);
                        if !cleaned_url.is_empty() && cleaned_url.starts_with("http") {
                            return cleaned_url;
                        }
                    }
                }
            }
        }

        // Fallback: look for any link in the element
        if let Ok(selector) = Selector::parse("a[href]") {
            if let Some(link_element) = element.select(&selector).next() {
                if let Some(href) = link_element.value().attr("href") {
                    let cleaned_url = self.clean_url(href);
                    if !cleaned_url.is_empty() && cleaned_url.starts_with("http") {
                        return cleaned_url;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_generic_snippet(&self, element: &scraper::ElementRef, selectors: &[&str]) -> String {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(snippet_element) = element.select(&selector).next() {
                    let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !snippet.is_empty() && snippet.len() > 10 && snippet.len() < 500 {
                        return snippet;
                    }
                }
            }
        }

        // Fallback: look for paragraph text
        if let Ok(selector) = Selector::parse("p") {
            if let Some(snippet_element) = element.select(&selector).next() {
                let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !snippet.is_empty() && snippet.len() > 10 && snippet.len() < 500 {
                    return snippet;
                }
            }
        }

        String::new()
    }

    fn clean_url(&self, url: &str) -> String {
        // Handle various redirect patterns
        if url.starts_with("/url?q=") {
            // Google-style redirect
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
        } else if url.starts_with("/l/?u=") {
            // Some redirect patterns
            if let Some(u_param) = url.strip_prefix("/l/?u=") {
                return urlencoding::decode(u_param)
                    .unwrap_or_else(|_| u_param.into())
                    .into_owned();
            }
        } else if url.starts_with("http") {
            return url.to_string();
        } else if url.starts_with("//") {
            return format!("https:{}", url);
        } else if url.starts_with("/") {
            // Relative URL - would need base URL to resolve properly
            return String::new();
        }

        url.to_string()
    }
}