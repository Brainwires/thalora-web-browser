use anyhow::{Result, Context};
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{SearchResult, SearchResults};
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

fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
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
