use anyhow::Result;
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{SearchResult, SearchResults, ImageSearchResult, ImageSearchResults};
use crate::protocols::mcp_server::scraping::utils::{extract_generic_snippet, extract_generic_title, extract_generic_url};

pub async fn search(query: &str, num_results: usize) -> Result<SearchResults> {
    eprintln!("🔍 DEBUG: search_google started");
    let search_url = format!("https://www.google.com/search?q={}&num={}&hl=en&gl=us",
                            urlencoding::encode(query), num_results);
    eprintln!("🔍 DEBUG: Google search URL: {}", search_url);

    // Create temporary browser for stateless search
    eprintln!("🔍 DEBUG: Creating temporary browser");
    let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();
    eprintln!("🔍 DEBUG: Temporary browser created, about to navigate");

    // Navigate using the browser's full navigation system which includes stealth features
    // Google requires JavaScript execution to display search results
    {
        let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        browser.navigate_to_with_js_option(&search_url, true, true).await?;
    }
    eprintln!("🔍 DEBUG: Navigation completed, getting content");

    let html = {
        let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        browser.get_current_content()
    };
    eprintln!("🔍 DEBUG: Content retrieved");

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

                        // Reuse the existing browser to follow the redirect (avoid IndexedDB lock conflict)
                        {
                            let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock for redirect"))?;
                            browser.navigate_to_with_js_option(&full_redirect_url, true, true).await?;
                        }

                        let redirect_html = {
                            let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock for redirect"))?;
                            browser.get_current_content()
                        };

                        eprintln!("🔍 DEBUG: Redirect response length: {} chars", redirect_html.len());
                        eprintln!("🔍 DEBUG: Redirect response preview: {}",
                            if redirect_html.len() > 500 { &redirect_html[..500] } else { &redirect_html });

                        // Explicitly drop browser to ensure cleanup
                        drop(temp_browser);

                        // Parse the redirect response instead
                        return parse_results(&redirect_html, query, num_results);
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

    // Explicitly drop browser to ensure cleanup
    drop(temp_browser);

    parse_results(&html, query, num_results)
}

pub fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    eprintln!("🔍 DEBUG: Google HTML length: {}", html.len());
    eprintln!("🔍 DEBUG: Google HTML contains .g class: {}", html.contains("class=\"g\""));
    eprintln!("🔍 DEBUG: Google HTML contains .tF2Cxc: {}", html.contains("tF2Cxc"));
    eprintln!("🔍 DEBUG: First 500 chars: {}", &html[..html.len().min(500)]);

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

                let title = extract_generic_title(&element, &title_selectors);
                let mut url = extract_generic_url(&element, &url_selectors);
                let snippet = extract_generic_snippet(&element, &snippet_selectors);

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

/// Perform Google image search
pub async fn image_search(query: &str, num_results: usize) -> Result<ImageSearchResults> {
    let search_url = format!(
        "https://www.google.com/search?q={}&tbm=isch&hl=en&gl=us",
        urlencoding::encode(query)
    );
    eprintln!("🔍🖼️ Google Images: Searching for '{}' at URL: {}", query, search_url);

    let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

    {
        let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        browser.navigate_to_with_js_option(&search_url, true, true).await?;
    }

    let html = {
        let browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        browser.get_current_content()
    };

    drop(temp_browser);

    parse_image_results(&html, query, num_results)
}

pub fn parse_image_results(html: &str, query: &str, num_results: usize) -> Result<ImageSearchResults> {
    let document = Html::parse_document(html);
    let mut results = Vec::new();

    // Google Images selectors
    let selectors = [
        "div.rg_bx img",
        "img.rg_i",
        "div.isv-r img",
        "a[data-ved] img",
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let image_url = element.value().attr("src")
                    .or_else(|| element.value().attr("data-src"))
                    .unwrap_or("");

                // Skip base64 encoded thumbnails and tracking pixels
                if image_url.is_empty()
                    || image_url.starts_with("data:")
                    || image_url.contains("1x1")
                {
                    continue;
                }

                let title = element.value().attr("alt")
                    .or_else(|| element.value().attr("title"))
                    .unwrap_or("")
                    .to_string();

                results.push(ImageSearchResult {
                    title: if title.is_empty() { format!("Image {}", results.len() + 1) } else { title },
                    image_url: image_url.to_string(),
                    thumbnail_url: Some(image_url.to_string()),
                    source_url: String::new(),
                    width: None,
                    height: None,
                    position: results.len() + 1,
                });
            }
        }
        if results.len() >= num_results {
            break;
        }
    }

    let result_count = results.len();
    Ok(ImageSearchResults {
        query: query.to_string(),
        results,
        total_results: Some(format!("{} results", result_count)),
    })
}
