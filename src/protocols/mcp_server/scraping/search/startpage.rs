use anyhow::Result;
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{SearchResult, SearchResults};
use crate::protocols::mcp_server::scraping::utils::{
    extract_generic_snippet, extract_generic_title, extract_generic_url,
};

pub async fn search(query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.startpage.com/do/search?query={}",
        urlencoding::encode(query)
    );

    // Temporary browser on its own thread for this stateless search
    let temp_browser = super::temporary_browser("startpage")?;
    let html = super::navigate_and_read(&temp_browser, search_url, false).await?;

    // Explicitly drop browser to ensure cleanup
    drop(temp_browser);

    parse_results(&html, query, num_results)
}

pub fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = Html::parse_document(html);
    let mut results = Vec::new();

    // Startpage result selectors
    if let Ok(selector) = Selector::parse(".result") {
        for element in document.select(&selector) {
            if results.len() >= num_results {
                break;
            }

            let title = extract_generic_title(&element, &[".result-title a", "h3 a", "h2 a"]);
            let url = extract_generic_url(&element, &[".result-title a", "h3 a", "h2 a"]);
            let snippet = extract_generic_snippet(&element, &[".result-desc", ".snippet"]);

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
