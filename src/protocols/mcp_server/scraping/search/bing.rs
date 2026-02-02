use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{SearchResult, SearchResults, ImageSearchResult, ImageSearchResults};
use crate::protocols::mcp_server::scraping::utils::{extract_generic_snippet, extract_generic_title, extract_generic_url};

pub async fn search(query: &str, num_results: usize) -> Result<SearchResults> {
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

    // Explicitly drop browser to ensure cleanup
    drop(temp_browser);

    // Check if we got a Cloudflare challenge that wasn't resolved
    // The navigation layer should have handled this, but check just in case
    if html.contains("challenges.cloudflare.com") && html.contains("cf-browser-verification") {
        eprintln!("⚠️ WARNING: Cloudflare challenge still present after navigation - this may indicate a bypass failure");
        // Continue anyway - the parse_results will return empty results if needed
    }

    parse_results(&html, query, num_results)
}

pub fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    eprintln!("🔍 DEBUG: Bing HTML length: {}", html.len());
    eprintln!("🔍 DEBUG: Bing HTML contains .b_algo: {}", html.contains(".b_algo"));
    eprintln!("🔍 DEBUG: Bing HTML contains cloudflare: {}", html.contains("cloudflare"));
    eprintln!("🔍 DEBUG: First 500 chars: {}", &html[..html.len().min(500)]);

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

                let title = extract_generic_title(&element, &title_selectors);
                let mut url = extract_generic_url(&element, &url_selectors);
                let snippet = extract_generic_snippet(&element, &snippet_selectors);

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

/// Perform Bing image search
pub async fn image_search(query: &str, num_results: usize) -> Result<ImageSearchResults> {
    let search_url = format!(
        "https://www.bing.com/images/search?q={}&first=1&count={}",
        urlencoding::encode(query), num_results
    );
    eprintln!("🔍🖼️ Bing Images: Searching for '{}' at URL: {}", query, search_url);

    let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

    {
        let mut browser = temp_browser.lock().map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
        browser.navigate_to_with_options(&search_url, true).await?;
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

    // Bing Images selectors
    let selectors = [
        ".iusc img",
        ".mimg",
        "a.iusc img",
        ".imgpt img",
        "img[src*='th.bing.com']",
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

                // Skip empty and placeholder images
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
