use anyhow::Result;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use url::Url;
use base64::{engine::general_purpose, Engine as _};

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
        eprintln!("🔍 DEBUG: Starting web_search function");
        let query = arguments["query"].as_str().unwrap_or("");
        let num_results = arguments["num_results"].as_u64().unwrap_or(10) as usize;
        let search_engine = arguments["search_engine"].as_str().unwrap_or("google");
        eprintln!("🔍 DEBUG: Parameters - query: {}, num_results: {}, engine: {}", query, num_results, search_engine);

        if query.is_empty() {
            return McpResponse::error(-1, "Query parameter is required".to_string());
        }

        let num_results = num_results.min(20); // Cap at 20 results
        eprintln!("🔍 DEBUG: About to call perform_web_search");

        match self.perform_web_search(query, num_results, search_engine).await {
            Ok(results) => {
                eprintln!("🔍 DEBUG: perform_web_search succeeded");
                McpResponse::success(serde_json::to_value(results).unwrap_or_default())
            },
            Err(e) => {
                eprintln!("🔍 DEBUG: perform_web_search failed: {}", e);
                McpResponse::error(-1, format!("Web search failed: {}", e))
            }
        }
    }

    async fn perform_web_search(&mut self, query: &str, num_results: usize, search_engine: &str) -> Result<SearchResults> {
        eprintln!("🔍 DEBUG: perform_web_search called with engine: {}", search_engine);
        match search_engine {
            "duckduckgo" => {
                eprintln!("🔍 DEBUG: Calling search_duckduckgo");
                self.search_duckduckgo(query, num_results).await
            },
            "bing" => {
                eprintln!("🔍 DEBUG: Calling search_bing");
                self.search_bing(query, num_results).await
            },
            "google" => {
                eprintln!("🔍 DEBUG: Calling search_google");
                self.search_google(query, num_results).await
            },
            "startpage" => {
                eprintln!("🔍 DEBUG: Calling search_startpage");
                self.search_startpage(query, num_results).await
            },
            "searx" => {
                eprintln!("🔍 DEBUG: Calling search_searx");
                self.search_searx(query, num_results).await
            },
            _ => Err(anyhow::anyhow!("Unsupported search engine: {}. Supported engines: google, bing, duckduckgo, startpage, searx", search_engine)),
        }
    }

    async fn search_duckduckgo(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));

        // Create temporary browser for stateless search
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

        // Navigate with JavaScript support
        {
            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.navigate_to_with_options(&search_url, true).await?;
        }

        // Get the rendered content
        let html = {
            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.get_current_content()
        };
        // Browser dropped here automatically

        self.parse_duckduckgo_results(&html, query, num_results).await
    }

    async fn search_bing(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        let search_url = format!("https://www.bing.com/search?q={}&count={}&FORM=QBLH",
                                urlencoding::encode(query), num_results);

        // Create temporary browser for stateless search
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

        // Navigate using the browser's full navigation system which includes stealth features
        {
            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.navigate_to_with_options(&search_url, true).await?;
        }

        let html = {
            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.get_current_content()
        };
        // Browser dropped here automatically

        // Check for actual Cloudflare challenge (not just JS that mentions cloudflare)
        if html.contains("challenges.cloudflare.com") && html.contains("cf-browser-verification") {
            return Err(anyhow::anyhow!("Bing returned Cloudflare challenge - need enhanced stealth"));
        }

        self.parse_bing_results(&html, query, num_results).await
    }

    async fn search_google(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        eprintln!("🔍 DEBUG: search_google started");
        let search_url = format!("https://www.google.com/search?q={}&num={}&hl=en&gl=us",
                                urlencoding::encode(query), num_results);
        eprintln!("🔍 DEBUG: Google search URL: {}", search_url);

        // Create temporary browser for stateless search
        eprintln!("🔍 DEBUG: Creating temporary browser");
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();
        eprintln!("🔍 DEBUG: Temporary browser created, about to navigate");

        // Navigate using the browser's full navigation system which includes stealth features
        {
            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.navigate_to_with_options(&search_url, true).await?;
        }
        eprintln!("🔍 DEBUG: Navigation completed, getting content");

        let html = {
            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.get_current_content()
        };
        eprintln!("🔍 DEBUG: Content retrieved");
        // Browser dropped here automatically

        // Check for Google's bot detection challenges
        if html.contains("Our systems have detected unusual traffic") || html.contains("why did this happen") {
            return Err(anyhow::anyhow!("Google returned bot detection challenge"));
        }

        // Check for reCAPTCHA challenge
        if html.contains("recaptcha") && html.contains("challenge") {
            return Err(anyhow::anyhow!("Google returned reCAPTCHA challenge"));
        }

        // Check for JavaScript challenge/enablejs redirect
        if html.contains("/httpservice/retry/enablejs") || (html.contains("<style>table,div,span,p{display:none}</style>") && html.contains("refresh")) {
            return Err(anyhow::anyhow!("Google returned JavaScript challenge - requires browser automation with JS execution"));
        }

        self.parse_google_results(&html, query, num_results).await
    }

    async fn search_startpage(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        let search_url = format!("https://www.startpage.com/do/search?query={}", urlencoding::encode(query));

        // Create temporary browser for stateless search
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

        {
            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.navigate_to_with_options(&search_url, true).await?;
        }

        let html = {
            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.get_current_content()
        };
        // Browser dropped here automatically

        self.parse_startpage_results(&html, query, num_results).await
    }

    async fn search_searx(&mut self, query: &str, num_results: usize) -> Result<SearchResults> {
        // Use public SearX instance
        let search_url = format!("https://searx.be/search?q={}&format=html", urlencoding::encode(query));

        // Create temporary browser for stateless search
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

        {
            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.navigate_to_with_options(&search_url, true).await?;
        }

        let html = {
            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
            browser.get_current_content()
        };
        // Browser dropped here automatically

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

        // Modern Bing result selectors - updated for current Bing structure
        let main_selectors = [
            ".b_algo",           // Main result container
            ".b_algoSlug",       // Alternative result container
            "li.b_algo",         // List item with class
            "[data-feedback]",   // Modern Bing feedback-enabled results
        ];

        for selector_str in &main_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    // Try multiple title selectors for Bing
                    let title_selectors = [
                        "h2 a",
                        ".b_title a",
                        ".b_algoheader a",
                        ".b_topTitle a",
                        "a[href] h2",
                        "a[href] h3"
                    ];

                    let url_selectors = [
                        "h2 a",
                        ".b_title a",
                        ".b_algoheader a",
                        ".b_topTitle a",
                        "a[href]:first-of-type"
                    ];

                    let snippet_selectors = [
                        ".b_caption p",
                        ".b_snippet",
                        ".b_paractl",
                        ".b_descript",
                        ".b_lineclamp2",
                        ".b_lineclamp3",
                        ".b_lineclamp4",
                        "p"
                    ];

                    let title = self.extract_generic_title(&element, &title_selectors);
                    let mut url = self.extract_generic_url(&element, &url_selectors);
                    let snippet = self.extract_generic_snippet(&element, &snippet_selectors);

                    // Clean up Bing redirect URLs
                    if url.starts_with("https://www.bing.com/ck/a?") || url.contains("&u=a1aHR0") {
                        // Extract actual URL from Bing redirect
                            if let Some(u_param) = url.split("&u=a1").nth(1) {
                            let param_value = u_param.chars().take_while(|&c| c != '&').collect::<String>();
                            if let Ok(decoded) = general_purpose::STANDARD.decode(&param_value) {
                                if let Ok(decoded_url) = String::from_utf8(decoded) {
                                    url = decoded_url;
                                }
                            }
                        }
                    }

                    // Only add if we have valid title and URL, and it's not a duplicate
                    if !title.is_empty() && !url.is_empty() && url.starts_with("http")
                        && !url.contains("bing.com") && !url.contains("microsoft.com")
                        && !results.iter().any(|r: &SearchResult| r.url == url) {
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

        let result_count = results.len();
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_results: Some(format!("{} results", result_count)),
            search_time: None,
        })
    }

    async fn parse_google_results(&self, html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Google result selectors - multiple approaches since Google changes frequently
        let main_selectors = [
            ".g",                    // Classic Google result container
            "[data-sokoban-container]", // Modern Google result container
            ".tF2Cxc",              // Current Google search result container
            ".rc",                  // Legacy Google result container
        ];

        for selector_str in &main_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if results.len() >= num_results {
                        break;
                    }

                    // Google title selectors
                    let title_selectors = [
                        "h3",
                        ".LC20lb",
                        ".DKV0Md",
                        "a h3",
                        ".r h3 a",
                        ".yuRUbf h3 a"
                    ];

                    // Google URL selectors
                    let url_selectors = [
                        "a[href]",
                        ".yuRUbf a",
                        ".r a",
                        "h3 a"
                    ];

                    // Google snippet selectors
                    let snippet_selectors = [
                        ".VwiC3b",
                        ".s",
                        ".st",
                        ".IsZvec",
                        "span[data-ved]"
                    ];

                    let title = self.extract_generic_title(&element, &title_selectors);
                    let mut url = self.extract_generic_url(&element, &url_selectors);
                    let snippet = self.extract_generic_snippet(&element, &snippet_selectors);

                    // Clean up Google redirect URLs
                    if url.starts_with("/url?q=") {
                        if let Some(actual_url) = url.strip_prefix("/url?q=") {
                            if let Some(clean_url) = actual_url.split('&').next() {
                                url = urlencoding::decode(clean_url).unwrap_or_default().to_string();
                            }
                        }
                    }

                    // Make relative URLs absolute
                    if url.starts_with("/") {
                        url = format!("https://www.google.com{}", url);
                    }

                    // Only add if we have valid title and URL, and it's not a Google internal URL
                    if !title.is_empty() && !url.is_empty() && url.starts_with("http")
                        && !url.contains("google.com") && !url.contains("youtube.com")
                        && !results.iter().any(|r: &SearchResult| r.url == url) {
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
            if let Some(_parent_element) = parent.value().as_element() {
                let parent_ref = scraper::ElementRef::wrap(parent).unwrap();
                return self.extract_modern_search_result_url(&parent_ref);
            }
        }

        String::new()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn extract_link_text(&self, element: &scraper::ElementRef) -> String {
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if text.len() > 3 && text.len() < 200 {
            text
        } else {
            String::new()
        }
    }

    #[allow(dead_code)]
    fn extract_nearby_snippet(&self, element: &scraper::ElementRef) -> String {
        // Look for snippet text in sibling elements
        if let Some(parent) = element.parent() {
            if let Some(_parent_element) = parent.value().as_element() {
                let parent_ref = scraper::ElementRef::wrap(parent).unwrap();

                // Check siblings for snippet-like content
                for sibling in parent_ref.children() {
                    if let Some(_sibling_element) = sibling.value().as_element() {
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

    /// Extract clean, readable content from a webpage using advanced readability algorithms
    pub(super) async fn scrape_readable_content(&mut self, arguments: Value) -> McpResponse {
        let url = match arguments["url"].as_str() {
            Some(url) => url,
            None => return McpResponse::error(-1, "URL parameter is required".to_string()),
        };

        // Parse optional parameters
        let format = arguments["format"].as_str().unwrap_or("markdown");
        let include_images = arguments["include_images"].as_bool().unwrap_or(true);
        let include_metadata = arguments["include_metadata"].as_bool().unwrap_or(true);
        let min_content_score = arguments["min_content_score"].as_f64().unwrap_or(0.3) as f32;

        // Validate format parameter
        let output_format = match format {
            "markdown" => crate::features::readability::OutputFormat::Markdown,
            "text" => crate::features::readability::OutputFormat::Text,
            "structured" => crate::features::readability::OutputFormat::Structured,
            _ => return McpResponse::error(-1, "Invalid format. Must be 'markdown', 'text', or 'structured'".to_string()),
        };

        // Fetch the webpage
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        let html_content = match client.get(url).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => text,
                    Err(e) => return McpResponse::error(-1, format!("Failed to read response body: {}", e)),
                }
            },
            Err(e) => return McpResponse::error(-1, format!("Failed to fetch URL: {}", e)),
        };

        // Use the readability engine to extract content
        let mut extractor = crate::features::readability::ReadabilityExtractor::new();
        let document = scraper::Html::parse_document(&html_content);

        let options = crate::features::readability::ExtractionOptions {
            base_url: url.to_string(),
            include_images,
            include_metadata,
            output_format,
            min_content_score,
            max_link_density: 0.25,
            min_paragraph_count: 2,
        };

        match extractor.extract(&document, &options) {
            Ok(result) => {
                if result.success {
                    let response_data = serde_json::json!({
                        "content": result.content.content,
                        "format": format,
                        "metadata": result.content.metadata,
                        "quality": result.quality,
                        "processing_time_ms": result.processing_time_ms
                    });
                    McpResponse::success(response_data)
                } else {
                    McpResponse::error(-1, result.error.unwrap_or("Extraction failed".to_string()))
                }
            },
            Err(e) => McpResponse::error(-1, format!("Content extraction failed: {}", e)),
        }
    }

    /// Extract content from multi-page articles with session support and automatic pagination handling
    pub(super) async fn browse_readable_content(&mut self, arguments: Value) -> McpResponse {
        let url = match arguments["url"].as_str() {
            Some(url) => url,
            None => return McpResponse::error(-1, "URL parameter is required".to_string()),
        };

        // Parse optional parameters
        let format = arguments["format"].as_str().unwrap_or("markdown");
        let follow_pagination = arguments["follow_pagination"].as_bool().unwrap_or(true);
        let max_pages = arguments["max_pages"].as_u64().unwrap_or(10) as usize;
        let wait_for_js = arguments["wait_for_js"].as_bool().unwrap_or(false);
        let include_images = arguments["include_images"].as_bool().unwrap_or(true);
        let session_id = arguments["session_id"].as_str();

        // Validate format parameter
        let output_format = match format {
            "markdown" => crate::features::readability::OutputFormat::Markdown,
            "text" => crate::features::readability::OutputFormat::Text,
            "structured" => crate::features::readability::OutputFormat::Structured,
            _ => return McpResponse::error(-1, "Invalid format. Must be 'markdown', 'text', or 'structured'".to_string()),
        };

        // For now, implement as single-page extraction
        // TODO: Add actual multi-page session support and pagination detection
        if session_id.is_some() {
            eprintln!("Warning: Session-based browsing not yet implemented, falling back to single-page extraction");
        }

        if follow_pagination && max_pages > 1 {
            eprintln!("Warning: Pagination following not yet implemented, extracting single page only");
        }

        if wait_for_js {
            eprintln!("Warning: JavaScript execution not yet implemented for readability extraction");
        }

        // For now, use the same logic as scrape_readable_content
        // but with a slightly lower quality threshold for multi-page content
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true)  // Enable cookies for session-like behavior
            .build()
            .unwrap();

        let html_content = match client.get(url).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => text,
                    Err(e) => return McpResponse::error(-1, format!("Failed to read response body: {}", e)),
                }
            },
            Err(e) => return McpResponse::error(-1, format!("Failed to fetch URL: {}", e)),
        };

        // Use the readability engine with slightly more permissive settings for multi-page content
        let mut extractor = crate::features::readability::ReadabilityExtractor::new();
        let document = scraper::Html::parse_document(&html_content);

        let options = crate::features::readability::ExtractionOptions {
            base_url: url.to_string(),
            include_images,
            include_metadata: true,
            output_format,
            min_content_score: 0.25,  // Slightly lower threshold for multi-page content
            max_link_density: 0.30,  // Allow slightly more links for multi-page articles
            min_paragraph_count: 1,  // Lower minimum for partial content
        };

        match extractor.extract(&document, &options) {
            Ok(result) => {
                if result.success {
                    let response_data = serde_json::json!({
                        "content": result.content.content,
                        "format": format,
                        "metadata": result.content.metadata,
                        "quality": result.quality,
                        "processing_time_ms": result.processing_time_ms,
                        "pages_processed": 1,
                        "session_used": session_id.is_some(),
                        "pagination_followed": false  // Not yet implemented
                    });
                    McpResponse::success(response_data)
                } else {
                    McpResponse::error(-1, result.error.unwrap_or("Content extraction failed".to_string()))
                }
            },
            Err(e) => McpResponse::error(-1, format!("Content extraction failed: {}", e)),
        }
    }

    /// Extract tables from HTML content
    pub(super) fn extract_tables(&self, html: &str) -> Vec<serde_json::Value> {
        let document = Html::parse_document(html);
        let mut tables = Vec::new();

        // Select all tables
        if let Ok(table_selector) = Selector::parse("table") {
            for table in document.select(&table_selector) {
                let mut table_data = serde_json::json!({
                    "headers": [],
                    "rows": [],
                    "caption": null
                });

                // Extract caption if present
                if let Ok(caption_selector) = Selector::parse("caption") {
                    if let Some(caption) = table.select(&caption_selector).next() {
                        table_data["caption"] = serde_json::Value::String(
                            caption.text().collect::<Vec<_>>().join(" ").trim().to_string()
                        );
                    }
                }

                // Extract headers from thead or first tr
                if let Ok(thead_selector) = Selector::parse("thead tr th, thead tr td") {
                    let headers: Vec<String> = table.select(&thead_selector)
                        .map(|th| th.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .collect();

                    if !headers.is_empty() {
                        table_data["headers"] = serde_json::Value::Array(
                            headers.into_iter().map(serde_json::Value::String).collect()
                        );
                    }
                }

                // If no headers in thead, try first tr
                if table_data["headers"].as_array().map_or(true, |h| h.is_empty()) {
                    if let Ok(first_row_selector) = Selector::parse("tr:first-child th, tr:first-child td") {
                        let headers: Vec<String> = table.select(&first_row_selector)
                            .map(|th| th.text().collect::<Vec<_>>().join(" ").trim().to_string())
                            .collect();

                        if !headers.is_empty() {
                            table_data["headers"] = serde_json::Value::Array(
                                headers.into_iter().map(serde_json::Value::String).collect()
                            );
                        }
                    }
                }

                // Extract data rows
                if let Ok(row_selector) = Selector::parse("tbody tr, tr") {
                    let mut rows = Vec::new();
                    let mut skip_first = table_data["headers"].as_array().map_or(false, |h| !h.is_empty());

                    for row in table.select(&row_selector) {
                        if skip_first {
                            skip_first = false;
                            continue;
                        }

                        if let Ok(cell_selector) = Selector::parse("td, th") {
                            let cells: Vec<String> = row.select(&cell_selector)
                                .map(|td| td.text().collect::<Vec<_>>().join(" ").trim().to_string())
                                .collect();

                            if !cells.is_empty() && !cells.iter().all(|c| c.is_empty()) {
                                rows.push(serde_json::Value::Array(
                                    cells.into_iter().map(serde_json::Value::String).collect()
                                ));
                            }
                        }
                    }

                    table_data["rows"] = serde_json::Value::Array(rows);
                }

                // Only include tables with meaningful content
                if table_data["rows"].as_array().map_or(false, |rows| !rows.is_empty()) {
                    tables.push(table_data);
                }
            }
        }

        tables
    }

    /// Extract lists (ul, ol) from HTML content
    pub(super) fn extract_lists(&self, html: &str) -> Vec<serde_json::Value> {
        let document = Html::parse_document(html);
        let mut lists = Vec::new();

        // Select all lists
        if let Ok(list_selector) = Selector::parse("ul, ol") {
            for list in document.select(&list_selector) {
                let list_type = list.value().name();
                let mut list_data = serde_json::json!({
                    "type": list_type,
                    "items": []
                });

                // Extract list items
                if let Ok(item_selector) = Selector::parse("li") {
                    let items: Vec<String> = list.select(&item_selector)
                        .map(|li| {
                            // Get text content, handling nested lists
                            let mut text = String::new();
                            for node in li.children() {
                                if let Some(element) = node.value().as_element() {
                                    if element.name() != "ul" && element.name() != "ol" {
                                        text.push_str(&scraper::ElementRef::wrap(node)
                                            .unwrap()
                                            .text()
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                            .trim());
                                        text.push(' ');
                                    }
                                } else if let Some(text_node) = node.value().as_text() {
                                    text.push_str(text_node.trim());
                                    text.push(' ');
                                }
                            }
                            text.trim().to_string()
                        })
                        .filter(|item| !item.is_empty())
                        .collect();

                    if !items.is_empty() {
                        list_data["items"] = serde_json::Value::Array(
                            items.into_iter().map(serde_json::Value::String).collect()
                        );
                        lists.push(list_data);
                    }
                }
            }
        }

        lists
    }

    /// Extract code blocks from HTML content
    pub(super) fn extract_code_blocks(&self, html: &str) -> Vec<serde_json::Value> {
        let document = Html::parse_document(html);
        let mut code_blocks = Vec::new();

        // Extract pre/code blocks
        let selectors = [
            "pre", "code", "pre code", ".highlight", ".code",
            ".sourceCode", ".language-*", "[class*='lang-']"
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let code_text = element.text().collect::<Vec<_>>().join("\n").trim().to_string();

                    if !code_text.is_empty() && code_text.len() > 10 {
                        // Try to detect language from class attributes
                        let mut language = None;
                        if let Some(class_attr) = element.value().attr("class") {
                            for class in class_attr.split_whitespace() {
                                if class.starts_with("language-") {
                                    language = Some(class.strip_prefix("language-").unwrap().to_string());
                                    break;
                                } else if class.starts_with("lang-") {
                                    language = Some(class.strip_prefix("lang-").unwrap().to_string());
                                    break;
                                }
                            }
                        }

                        let code_block = serde_json::json!({
                            "code": code_text,
                            "language": language,
                            "element_type": element.value().name()
                        });

                        // Avoid duplicates by checking if we already have this exact code
                        let is_duplicate = code_blocks.iter().any(|existing: &serde_json::Value| {
                            existing["code"].as_str() == Some(&code_text)
                        });

                        if !is_duplicate {
                            code_blocks.push(code_block);
                        }
                    }
                }
            }
        }

        code_blocks
    }

    /// Extract article metadata (author, publish date, tags) from HTML content
    pub(super) fn extract_article_metadata(&self, html: &str) -> serde_json::Value {
        let document = Html::parse_document(html);
        let mut metadata = serde_json::json!({
            "title": null,
            "author": null,
            "publish_date": null,
            "tags": [],
            "description": null,
            "canonical_url": null
        });

        // Extract title
        if let Ok(title_selector) = Selector::parse("title, h1, .title, .article-title, [property='og:title'], [name='twitter:title']") {
            if let Some(title_element) = document.select(&title_selector).next() {
                let title = if title_element.value().name() == "meta" {
                    title_element.value().attr("content").unwrap_or("").to_string()
                } else {
                    title_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };
                if !title.is_empty() {
                    metadata["title"] = serde_json::Value::String(title);
                }
            }
        }

        // Extract author
        let author_selectors = [
            "[name='author']", "[property='article:author']", "[rel='author']",
            ".author", ".byline", ".article-author", ".post-author"
        ];

        for selector_str in &author_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(author_element) = document.select(&selector).next() {
                    let author = if author_element.value().name() == "meta" {
                        author_element.value().attr("content").unwrap_or("").to_string()
                    } else {
                        author_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    };
                    if !author.is_empty() {
                        metadata["author"] = serde_json::Value::String(author);
                        break;
                    }
                }
            }
        }

        // Extract publish date
        let date_selectors = [
            "[property='article:published_time']", "[name='publish_date']",
            "[name='date']", "time[datetime]", ".publish-date", ".date",
            ".article-date", ".post-date"
        ];

        for selector_str in &date_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(date_element) = document.select(&selector).next() {
                    let date = if let Some(datetime) = date_element.value().attr("datetime") {
                        datetime.to_string()
                    } else if let Some(content) = date_element.value().attr("content") {
                        content.to_string()
                    } else {
                        date_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    };
                    if !date.is_empty() {
                        metadata["publish_date"] = serde_json::Value::String(date);
                        break;
                    }
                }
            }
        }

        // Extract tags/keywords
        let tag_selectors = [
            "[name='keywords']", "[property='article:tag']",
            ".tags a", ".tag", ".article-tags a", ".post-tags a"
        ];

        let mut tags = Vec::new();
        for selector_str in &tag_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for tag_element in document.select(&selector) {
                    let tag = if tag_element.value().name() == "meta" {
                        tag_element.value().attr("content").unwrap_or("").to_string()
                    } else {
                        tag_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    };

                    if !tag.is_empty() {
                        // Split comma-separated keywords
                        if tag.contains(',') {
                            for t in tag.split(',') {
                                let clean_tag = t.trim().to_string();
                                if !clean_tag.is_empty() && !tags.contains(&clean_tag) {
                                    tags.push(clean_tag);
                                }
                            }
                        } else if !tags.contains(&tag) {
                            tags.push(tag);
                        }
                    }
                }
            }
        }

        if !tags.is_empty() {
            metadata["tags"] = serde_json::Value::Array(
                tags.into_iter().map(serde_json::Value::String).collect()
            );
        }

        // Extract description
        let desc_selectors = [
            "[name='description']", "[property='og:description']",
            "[name='twitter:description']", ".description", ".summary"
        ];

        for selector_str in &desc_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(desc_element) = document.select(&selector).next() {
                    let description = if desc_element.value().name() == "meta" {
                        desc_element.value().attr("content").unwrap_or("").to_string()
                    } else {
                        desc_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    };
                    if !description.is_empty() {
                        metadata["description"] = serde_json::Value::String(description);
                        break;
                    }
                }
            }
        }

        // Extract canonical URL
        if let Ok(canonical_selector) = Selector::parse("[rel='canonical']") {
            if let Some(canonical_element) = document.select(&canonical_selector).next() {
                if let Some(href) = canonical_element.value().attr("href") {
                    metadata["canonical_url"] = serde_json::Value::String(href.to_string());
                }
            }
        }

        metadata
    }

    /// Extract structured content from a webpage (stateless operation)
    pub(super) async fn extract_structured_content(&mut self, arguments: Value) -> McpResponse {
        let url = match arguments["url"].as_str() {
            Some(url) => url,
            None => return McpResponse::error(-1, "URL parameter is required".to_string()),
        };

        // Parse optional parameters
        let content_types = arguments["content_types"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["tables", "lists", "code_blocks", "metadata"]);

        // Create temporary browser for stateless operation
        let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

        // Navigate and get content
        {
            let mut browser = match temp_browser.lock() {
                Ok(b) => b,
                Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
            };
            match browser.navigate_to_with_options(url, true).await {
                Ok(_) => {},
                Err(e) => return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
            }
        }

        let html = {
            let browser = match temp_browser.lock() {
                Ok(b) => b,
                Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
            };
            browser.get_current_content()
        };
        // Browser dropped here automatically for stateless operation

        // Extract requested content types
        let mut result = serde_json::json!({
            "url": url,
            "content_types_requested": content_types,
            "extracted_content": {}
        });

        for content_type in &content_types {
            match content_type.as_ref() {
                "tables" => {
                    let tables = self.extract_tables(&html);
                    result["extracted_content"]["tables"] = serde_json::Value::Array(tables);
                },
                "lists" => {
                    let lists = self.extract_lists(&html);
                    result["extracted_content"]["lists"] = serde_json::Value::Array(lists);
                },
                "code_blocks" => {
                    let code_blocks = self.extract_code_blocks(&html);
                    result["extracted_content"]["code_blocks"] = serde_json::Value::Array(code_blocks);
                },
                "metadata" => {
                    let metadata = self.extract_article_metadata(&html);
                    result["extracted_content"]["metadata"] = metadata;
                },
                _ => {
                    // Unknown content type - skip it
                    continue;
                }
            }
        }

        // Add summary information
        let mut summary = serde_json::json!({
            "total_tables": 0,
            "total_lists": 0,
            "total_code_blocks": 0,
            "has_metadata": false
        });

        if let Some(tables) = result["extracted_content"]["tables"].as_array() {
            summary["total_tables"] = serde_json::Value::Number(serde_json::Number::from(tables.len()));
        }

        if let Some(lists) = result["extracted_content"]["lists"].as_array() {
            summary["total_lists"] = serde_json::Value::Number(serde_json::Number::from(lists.len()));
        }

        if let Some(code_blocks) = result["extracted_content"]["code_blocks"].as_array() {
            summary["total_code_blocks"] = serde_json::Value::Number(serde_json::Number::from(code_blocks.len()));
        }

        if let Some(metadata) = result["extracted_content"]["metadata"].as_object() {
            summary["has_metadata"] = serde_json::Value::Bool(!metadata.is_empty());
        }

        result["summary"] = summary;

        McpResponse::success(result)
    }
}