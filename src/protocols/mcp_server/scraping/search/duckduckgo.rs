use anyhow::{Result, Context};
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{SearchResult, SearchResults, ImageSearchResult, ImageSearchResults};
use crate::protocols::mcp_server::scraping::utils::{extract_generic_snippet, extract_generic_title, extract_generic_url};

pub async fn search(query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    eprintln!("🦆 DuckDuckGo: Searching for '{}' at URL: {}", query, search_url);

    // Create temporary browser for stateless search with error context
    eprintln!("🦆 DuckDuckGo: Creating temporary browser instance...");
    let temp_browser = match std::panic::catch_unwind(|| {
        crate::engine::browser::HeadlessWebBrowser::new()
    }) {
        Ok(browser) => browser,
        Err(panic_payload) => {
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during browser creation".to_string()
            };
            eprintln!("❌ DuckDuckGo: Browser creation panicked: {}", panic_msg);
            return Err(anyhow::anyhow!("Failed to create browser: {}", panic_msg));
        }
    };
    eprintln!("🦆 DuckDuckGo: Browser instance created successfully");

    // Navigate with JavaScript support
    eprintln!("🦆 DuckDuckGo: Acquiring browser lock for navigation...");
    {
        let mut browser = temp_browser.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire browser lock: mutex poisoned ({})", e))?;
        eprintln!("🦆 DuckDuckGo: Navigating to search URL...");
        browser.navigate_to_with_options(&search_url, true).await
            .context("Failed to navigate to DuckDuckGo search URL")?;
        eprintln!("🦆 DuckDuckGo: Navigation completed");
    }

    // Get the rendered content
    eprintln!("🦆 DuckDuckGo: Acquiring browser lock for content extraction...");
    let html = {
        let browser = temp_browser.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire browser lock for content: mutex poisoned ({})", e))?;
        let content = browser.get_current_content();
        eprintln!("🦆 DuckDuckGo: Retrieved {} bytes of HTML content", content.len());
        content
    };

    // Explicitly drop browser to ensure cleanup
    eprintln!("🦆 DuckDuckGo: Cleaning up browser instance...");
    drop(temp_browser);
    eprintln!("🦆 DuckDuckGo: Browser cleanup complete, parsing results...");

    let results = parse_results(&html, query, num_results)
        .context("Failed to parse DuckDuckGo search results")?;

    eprintln!("🦆 DuckDuckGo: Parsed {} results successfully", results.results.len());
    Ok(results)
}

pub fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = Html::parse_document(html);
    let mut results = Vec::new();

    // DuckDuckGo HTML result selectors
    if let Ok(selector) = Selector::parse(".result__body") {
        for element in document.select(&selector) {
            if results.len() >= num_results {
                break;
            }

            let title = extract_generic_title(&element, &[".result__title a", "h2 a", "h3 a"]);
            let url = extract_generic_url(&element, &[".result__title a", "h2 a", "h3 a"]);
            let snippet = extract_generic_snippet(&element, &[".result__snippet", ".result__body .snippet"]);

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

/// Perform DuckDuckGo image search
pub async fn image_search(query: &str, num_results: usize) -> Result<ImageSearchResults> {
    // DuckDuckGo images use a JavaScript-based interface, but we can use the lite version
    // For images, we'll scrape the HTML results and look for image links
    let search_url = format!(
        "https://duckduckgo.com/?q={}&t=h_&iax=images&ia=images",
        urlencoding::encode(query)
    );
    eprintln!("🦆🖼️ DuckDuckGo Images: Searching for '{}' at URL: {}", query, search_url);

    // Create temporary browser for stateless search
    let temp_browser = match std::panic::catch_unwind(|| {
        crate::engine::browser::HeadlessWebBrowser::new()
    }) {
        Ok(browser) => browser,
        Err(panic_payload) => {
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic during browser creation".to_string()
            };
            return Err(anyhow::anyhow!("Failed to create browser: {}", panic_msg));
        }
    };

    // Navigate with JavaScript support
    {
        let mut browser = temp_browser.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire browser lock: mutex poisoned ({})", e))?;
        browser.navigate_to_with_options(&search_url, true).await
            .context("Failed to navigate to DuckDuckGo image search URL")?;
    }

    // Get the rendered content
    let html = {
        let browser = temp_browser.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire browser lock for content: mutex poisoned ({})", e))?;
        browser.get_current_content()
    };

    drop(temp_browser);

    let results = parse_image_results(&html, query, num_results)?;
    eprintln!("🦆🖼️ DuckDuckGo Images: Parsed {} results", results.results.len());
    Ok(results)
}

pub fn parse_image_results(html: &str, query: &str, num_results: usize) -> Result<ImageSearchResults> {
    let document = Html::parse_document(html);
    let mut results = Vec::new();

    // DuckDuckGo images are loaded dynamically, but we can try to parse any image links
    // Try multiple selectors for different page structures
    let selectors = [
        ".tile--img img",
        ".tile--img a",
        "img[data-src]",
        ".result__image",
        "a.result__a img",
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

                let title = element.value().attr("alt")
                    .or_else(|| element.value().attr("title"))
                    .unwrap_or("")
                    .to_string();

                let source_url = element.value().attr("href")
                    .or_else(|| {
                        // Try to get parent link
                        element.parent()
                            .and_then(|p| p.value().as_element())
                            .and_then(|e| e.attr("href"))
                    })
                    .unwrap_or("")
                    .to_string();

                if !image_url.is_empty() && image_url.starts_with("http") {
                    results.push(ImageSearchResult {
                        title: if title.is_empty() { query.to_string() } else { title },
                        image_url: image_url.to_string(),
                        thumbnail_url: Some(image_url.to_string()),
                        source_url,
                        width: None,
                        height: None,
                        position: results.len() + 1,
                    });
                }
            }
        }
        if results.len() >= num_results {
            break;
        }
    }

    // If we didn't find any images in structured elements, try to find any img tags
    if results.is_empty() {
        if let Ok(selector) = Selector::parse("img") {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let image_url = element.value().attr("src")
                    .or_else(|| element.value().attr("data-src"))
                    .unwrap_or("");

                // Skip small images and tracking pixels
                if image_url.is_empty()
                    || !image_url.starts_with("http")
                    || image_url.contains("1x1")
                    || image_url.contains("pixel")
                    || image_url.contains("tracking")
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
    }

    let result_count = results.len();
    Ok(ImageSearchResults {
        query: query.to_string(),
        results,
        total_results: Some(format!("{} results", result_count)),
    })
}
