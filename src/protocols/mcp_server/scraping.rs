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
    /// Unified scraping function that combines all scraping capabilities
    pub(super) async fn scrape_unified(&mut self, arguments: Value) -> McpResponse {
        let url = arguments["url"].as_str();
        let session_id = arguments.get("session_id").and_then(|v| v.as_str());

        // Validate that we have either URL or session_id
        if url.is_none() && session_id.is_none() {
            return McpResponse::error(-1, "Either 'url' or 'session_id' parameter is required".to_string());
        }

        // Session & Navigation options
        let wait_for_js = arguments.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(false);
        let wait_timeout = arguments.get("wait_timeout").and_then(|v| v.as_u64()).unwrap_or(5000);

        // What to extract (all default to true for comprehensive extraction)
        let extract_basic = arguments.get("extract_basic").and_then(|v| v.as_bool()).unwrap_or(true);
        let extract_readable = arguments.get("extract_readable").and_then(|v| v.as_bool()).unwrap_or(false);
        let extract_structured = arguments.get("extract_structured").and_then(|v| v.as_bool()).unwrap_or(false);
        let extract_by_selectors = arguments.get("selectors").and_then(|v| v.as_object()).cloned();

        // Readability options
        let readability_format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("markdown");
        let include_images = arguments.get("include_images").and_then(|v| v.as_bool()).unwrap_or(true);
        let include_metadata = arguments.get("include_metadata").and_then(|v| v.as_bool()).unwrap_or(true);
        let min_content_score = arguments.get("min_content_score").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32;

        // Structured content options
        let content_types = arguments.get("content_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["tables", "lists", "code_blocks", "metadata"]);

        // Pagination options
        let follow_pagination = arguments.get("follow_pagination").and_then(|v| v.as_bool()).unwrap_or(false);
        let max_pages = arguments.get("max_pages").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        // Navigate to URL if provided (or use existing session)
        let html_content = if let Some(url_str) = url {
            // Create temporary browser or use session
            let temp_browser = if session_id.is_some() {
                // TODO: Get session browser
                crate::engine::browser::HeadlessWebBrowser::new()
            } else {
                crate::engine::browser::HeadlessWebBrowser::new()
            };

            // Navigate to URL
            {
                let mut browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                };

                match browser.navigate_to_with_options(url_str, wait_for_js).await {
                    Ok(_) => {},
                    Err(e) => return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
                }
            }

            // Get HTML content
            let html = {
                let browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                };
                browser.get_current_content()
            };

            html
        } else {
            // Get content from session
            // TODO: Implement session-based content retrieval
            return McpResponse::error(-1, "Session-based scraping not yet implemented in unified scrape".to_string());
        };

        // Build unified response
        let mut result = serde_json::json!({
            "url": url.unwrap_or(""),
            "scraping_options": {
                "extract_basic": extract_basic,
                "extract_readable": extract_readable,
                "extract_structured": extract_structured,
                "has_custom_selectors": extract_by_selectors.is_some()
            }
        });

        // 1. Extract basic content (links, images, metadata)
        if extract_basic {
            let document = Html::parse_document(&html_content);

            // Extract links
            let mut links = Vec::new();
            if let Ok(link_selector) = Selector::parse("a[href]") {
                for element in document.select(&link_selector) {
                    if let Some(href) = element.value().attr("href") {
                        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        links.push(serde_json::json!({
                            "href": href,
                            "text": text
                        }));
                    }
                }
            }

            // Extract images
            let mut images = Vec::new();
            if let Ok(img_selector) = Selector::parse("img[src]") {
                for element in document.select(&img_selector) {
                    if let Some(src) = element.value().attr("src") {
                        let alt = element.value().attr("alt").unwrap_or("");
                        images.push(serde_json::json!({
                            "src": src,
                            "alt": alt
                        }));
                    }
                }
            }

            // Extract basic metadata
            let metadata = self.extract_article_metadata(&html_content);

            result["basic"] = serde_json::json!({
                "links": links,
                "images": images,
                "metadata": metadata
            });
        }

        // 2. Extract content by custom selectors
        if let Some(selectors) = extract_by_selectors {
            let document = Html::parse_document(&html_content);
            let mut selector_results = serde_json::Map::new();

            for (name, selector_str) in selectors {
                if let Some(selector_str) = selector_str.as_str() {
                    if let Ok(selector) = Selector::parse(selector_str) {
                        let mut matches = Vec::new();
                        for element in document.select(&selector) {
                            let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                            matches.push(serde_json::Value::String(text));
                        }
                        selector_results.insert(name, serde_json::Value::Array(matches));
                    } else {
                        selector_results.insert(
                            name,
                            serde_json::Value::String(format!("Invalid selector: {}", selector_str))
                        );
                    }
                }
            }

            result["by_selector"] = serde_json::Value::Object(selector_results);
        }

        // 3. Extract readable content using readability algorithms
        if extract_readable {
            let output_format = match readability_format {
                "markdown" => crate::features::readability::OutputFormat::Markdown,
                "text" => crate::features::readability::OutputFormat::Text,
                "structured" => crate::features::readability::OutputFormat::Structured,
                _ => crate::features::readability::OutputFormat::Markdown,
            };

            let mut extractor = crate::features::readability::ReadabilityExtractor::new();
            let document = scraper::Html::parse_document(&html_content);

            let options = crate::features::readability::ExtractionOptions {
                base_url: url.unwrap_or("").to_string(),
                include_images,
                include_metadata,
                output_format,
                min_content_score,
                max_link_density: 0.25,
                min_paragraph_count: 2,
            };

            match extractor.extract(&document, &options) {
                Ok(extraction_result) => {
                    if extraction_result.success {
                        result["readable"] = serde_json::json!({
                            "content": extraction_result.content.content,
                            "format": readability_format,
                            "metadata": extraction_result.content.metadata,
                            "quality": extraction_result.quality,
                            "processing_time_ms": extraction_result.processing_time_ms
                        });
                    } else {
                        result["readable"] = serde_json::json!({
                            "error": extraction_result.error.unwrap_or("Extraction failed".to_string())
                        });
                    }
                },
                Err(e) => {
                    result["readable"] = serde_json::json!({
                        "error": format!("Readability extraction failed: {}", e)
                    });
                }
            }
        }

        // 4. Extract structured content (tables, lists, code blocks)
        if extract_structured {
            let mut structured = serde_json::json!({});

            for content_type in &content_types {
                match content_type.as_ref() {
                    "tables" => {
                        let tables = self.extract_tables(&html_content);
                        structured["tables"] = serde_json::Value::Array(tables);
                    },
                    "lists" => {
                        let lists = self.extract_lists(&html_content);
                        structured["lists"] = serde_json::Value::Array(lists);
                    },
                    "code_blocks" => {
                        let code_blocks = self.extract_code_blocks(&html_content);
                        structured["code_blocks"] = serde_json::Value::Array(code_blocks);
                    },
                    "metadata" => {
                        let metadata = self.extract_article_metadata(&html_content);
                        structured["metadata"] = metadata;
                    },
                    _ => {}
                }
            }

            // Add summary
            let mut summary = serde_json::json!({
                "total_tables": 0,
                "total_lists": 0,
                "total_code_blocks": 0,
                "has_metadata": false
            });

            if let Some(tables) = structured["tables"].as_array() {
                summary["total_tables"] = serde_json::Value::Number(serde_json::Number::from(tables.len()));
            }
            if let Some(lists) = structured["lists"].as_array() {
                summary["total_lists"] = serde_json::Value::Number(serde_json::Number::from(lists.len()));
            }
            if let Some(code_blocks) = structured["code_blocks"].as_array() {
                summary["total_code_blocks"] = serde_json::Value::Number(serde_json::Number::from(code_blocks.len()));
            }
            if let Some(metadata) = structured["metadata"].as_object() {
                summary["has_metadata"] = serde_json::Value::Bool(!metadata.is_empty());
            }

            structured["summary"] = summary;
            result["structured"] = structured;
        }

        McpResponse::success(result)
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

        // Check for JavaScript challenge/enablejs redirect - but let's try to proceed anyway
        if html.contains("/httpservice/retry/enablejs") || (html.contains("<style>table,div,span,p{display:none}</style>") && html.contains("refresh")) {
            eprintln!("🔍 DEBUG: Google returned JavaScript challenge page, but attempting to parse anyway");
            eprintln!("🔍 DEBUG: Challenge page length: {} chars", html.len());
            // Instead of failing, let's try to follow the redirect or parse what we can

            // Try to extract the redirect URL and follow it
            if let Some(start) = html.find("http-equiv=\"refresh\"") {
                if let Some(content_start) = html[..start].rfind("content=\"") {
                    let content_part = &html[content_start + 9..];
                    if let Some(url_start) = content_part.find("url=") {
                        let url_part = &content_part[url_start + 4..];
                        if let Some(url_end) = url_part.find("\"") {
                            let redirect_url = &url_part[..url_end];
                            eprintln!("🔍 DEBUG: Found redirect URL: {}", redirect_url);

                            // Make a new request to the redirect URL
                            let full_redirect_url = if redirect_url.starts_with("/") {
                                format!("https://www.google.com{}", redirect_url)
                            } else {
                                redirect_url.to_string()
                            };

                            eprintln!("🔍 DEBUG: Following redirect to: {}", full_redirect_url);

                            // Create a new temporary browser for the redirect
                            let redirect_browser = crate::engine::browser::HeadlessWebBrowser::new();
                            {
                                let mut browser = redirect_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock for redirect"))?;
                                browser.navigate_to_with_options(&full_redirect_url, true).await?;
                            }

                            let redirect_html = {
                                let browser = redirect_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock for redirect"))?;
                                browser.get_current_content()
                            };

                            eprintln!("🔍 DEBUG: Redirect response length: {} chars", redirect_html.len());
                            eprintln!("🔍 DEBUG: Redirect response preview: {}",
                                if redirect_html.len() > 500 { &redirect_html[..500] } else { &redirect_html });

                            // Parse the redirect response instead
                            return self.parse_google_results(&redirect_html, query, num_results).await;
                        }
                    }
                }
            }

            // If we can't follow the redirect, just try to parse what we have
            eprintln!("🔍 DEBUG: Could not extract redirect URL, parsing challenge page directly");
        }

        // Let's also check if we got valid search results
        if !html.contains("</html>") || html.len() < 1000 {
            eprintln!("🔍 DEBUG: Got incomplete HTML response: {} chars", html.len());
            eprintln!("🔍 DEBUG: HTML content: {}", if html.len() > 500 { &html[..500] } else { &html });
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
                            existing["code"].as_str() == Some(code_text.as_str())
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

    /// Extract readable content using readability algorithm
    /// This is a dedicated method for the browse_readable_content MCP tool
    pub(super) async fn browse_readable_content(&mut self, arguments: Value) -> McpResponse {
        let url = arguments["url"].as_str();
        let session_id = arguments.get("session_id").and_then(|v| v.as_str());

        // Validate that we have either URL or session_id
        if url.is_none() && session_id.is_none() {
            return McpResponse::error(-1, "Either 'url' or 'session_id' parameter is required".to_string());
        }

        // Navigation options
        let wait_for_js = arguments.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(false);

        // Readability options
        let format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("markdown");
        let include_images = arguments.get("include_images").and_then(|v| v.as_bool()).unwrap_or(true);
        let include_metadata = arguments.get("include_metadata").and_then(|v| v.as_bool()).unwrap_or(true);
        let min_content_score = arguments.get("min_content_score").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32;

        // Navigate to URL if provided (or use existing session)
        let html_content = if let Some(url_str) = url {
            // Create temporary browser
            let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

            // Navigate to URL
            {
                let mut browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                };

                match browser.navigate_to_with_options(url_str, wait_for_js).await {
                    Ok(_) => {},
                    Err(e) => return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e)),
                }
            }

            // Get HTML content
            let html = {
                let browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => return McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                };
                browser.get_current_content()
            };

            html
        } else {
            // Get content from session
            return McpResponse::error(-1, "Session-based readable content extraction not yet implemented".to_string());
        };

        // Extract readable content using readability algorithm
        let output_format = match format {
            "markdown" => crate::features::readability::OutputFormat::Markdown,
            "text" => crate::features::readability::OutputFormat::Text,
            "structured" => crate::features::readability::OutputFormat::Structured,
            _ => crate::features::readability::OutputFormat::Markdown,
        };

        let mut extractor = crate::features::readability::ReadabilityExtractor::new();
        let document = scraper::Html::parse_document(&html_content);

        let options = crate::features::readability::ExtractionOptions {
            base_url: url.unwrap_or("").to_string(),
            include_images,
            include_metadata,
            output_format,
            min_content_score,
            max_link_density: 0.25,
            min_paragraph_count: 2,
        };

        match extractor.extract(&document, &options) {
            Ok(extraction_result) => {
                if extraction_result.success {
                    let result = serde_json::json!({
                        "url": url.unwrap_or(""),
                        "content": extraction_result.content.content,
                        "format": format,
                        "metadata": extraction_result.content.metadata,
                        "quality": extraction_result.quality,
                        "processing_time_ms": extraction_result.processing_time_ms,
                        "success": true
                    });
                    McpResponse::success(result)
                } else {
                    McpResponse::error(-1, extraction_result.error.unwrap_or("Extraction failed".to_string()))
                }
            },
            Err(e) => {
                McpResponse::error(-1, format!("Readability extraction failed: {}", e))
            }
        }
    }
}