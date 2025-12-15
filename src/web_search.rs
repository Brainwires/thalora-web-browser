//! Web search module - provides search functionality without requiring full MCP server
//!
//! This module exposes search capabilities for use by external crates
//! without pulling in the full browser infrastructure.
//! Uses simple HTTP requests which are Send + Sync friendly.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: usize,
}

/// Collection of search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: Option<String>,
    pub search_time: Option<String>,
}

/// Default user agent for web requests - using a common browser UA to avoid bot detection
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Create a configured HTTP client
fn create_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

/// Perform a web search using the specified search engine
///
/// # Arguments
/// * `query` - The search query
/// * `num_results` - Maximum number of results to return
/// * `search_engine` - Search engine to use: "duckduckgo", "bing", "google", or "startpage"
///
/// # Returns
/// SearchResults containing the search results
pub async fn perform_search(query: &str, num_results: usize, search_engine: &str) -> Result<SearchResults> {
    let client = create_client()?;

    match search_engine {
        "duckduckgo" => search_duckduckgo(&client, query, num_results).await,
        "bing" => search_bing(&client, query, num_results).await,
        "google" => search_google(&client, query, num_results).await,
        "startpage" => search_startpage(&client, query, num_results).await,
        _ => Err(anyhow::anyhow!("Unsupported search engine: {}. Use: duckduckgo, bing, google, or startpage", search_engine)),
    }
}

async fn search_duckduckgo(client: &reqwest::Client, query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));

    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;

    parse_duckduckgo_results(&html, query, num_results)
}

async fn search_bing(client: &reqwest::Client, query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.bing.com/search?q={}&count={}&FORM=QBLH",
        urlencoding::encode(query),
        num_results
    );

    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;

    parse_bing_results(&html, query, num_results)
}

async fn search_google(client: &reqwest::Client, query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.google.com/search?q={}&num={}&hl=en&gl=us",
        urlencoding::encode(query),
        num_results
    );

    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;

    parse_google_results(&html, query, num_results)
}

async fn search_startpage(client: &reqwest::Client, query: &str, num_results: usize) -> Result<SearchResults> {
    let search_url = format!("https://www.startpage.com/do/search?query={}", urlencoding::encode(query));

    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;

    parse_startpage_results(&html, query, num_results)
}

fn parse_duckduckgo_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    if let Ok(selector) = scraper::Selector::parse(".result__body") {
        for (i, element) in document.select(&selector).enumerate() {
            if i >= num_results {
                break;
            }

            let title = extract_text(&element, ".result__title a, .result__a");
            let url = extract_href(&element, ".result__title a, .result__a");
            let snippet = extract_text(&element, ".result__snippet");

            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url: clean_duckduckgo_url(&url),
                    snippet,
                    position: i + 1,
                });
            }
        }
    }

    Ok(SearchResults {
        query: query.to_string(),
        results,
        total_results: None,
        search_time: None,
    })
}

fn parse_bing_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    if let Ok(selector) = scraper::Selector::parse("li.b_algo") {
        for (i, element) in document.select(&selector).enumerate() {
            if i >= num_results {
                break;
            }

            let title = extract_text(&element, "h2 a");
            let url = extract_href(&element, "h2 a");
            let snippet = extract_text(&element, ".b_caption p, .b_algoSlug");

            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url,
                    snippet,
                    position: i + 1,
                });
            }
        }
    }

    Ok(SearchResults {
        query: query.to_string(),
        results,
        total_results: None,
        search_time: None,
    })
}

fn parse_google_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    // Try multiple selectors as Google changes their HTML frequently
    let selectors = ["div.g", ".tF2Cxc", ".Gx5Zad"];

    for sel_str in &selectors {
        if let Ok(selector) = scraper::Selector::parse(sel_str) {
            for element in document.select(&selector) {
                if results.len() >= num_results {
                    break;
                }

                let title = extract_text(&element, "h3");
                let url = extract_href(&element, "a");
                let snippet = extract_text(&element, ".VwiC3b, .IsZvec, .st");

                if !title.is_empty() && !url.is_empty()
                    && !url.starts_with("/search")
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
        if !results.is_empty() {
            break;
        }
    }

    Ok(SearchResults {
        query: query.to_string(),
        results,
        total_results: None,
        search_time: None,
    })
}

fn parse_startpage_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    let document = scraper::Html::parse_document(html);
    let mut results = Vec::new();

    if let Ok(selector) = scraper::Selector::parse(".w-gl__result") {
        for (i, element) in document.select(&selector).enumerate() {
            if i >= num_results {
                break;
            }

            let title = extract_text(&element, ".w-gl__result-title");
            let url = extract_href(&element, "a.w-gl__result-url");
            let snippet = extract_text(&element, ".w-gl__description");

            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url,
                    snippet,
                    position: i + 1,
                });
            }
        }
    }

    Ok(SearchResults {
        query: query.to_string(),
        results,
        total_results: None,
        search_time: None,
    })
}

fn extract_text(element: &scraper::ElementRef, selector: &str) -> String {
    if let Ok(sel) = scraper::Selector::parse(selector) {
        if let Some(el) = element.select(&sel).next() {
            return el.text().collect::<Vec<_>>().join(" ").trim().to_string();
        }
    }
    String::new()
}

fn extract_href(element: &scraper::ElementRef, selector: &str) -> String {
    if let Ok(sel) = scraper::Selector::parse(selector) {
        if let Some(el) = element.select(&sel).next() {
            if let Some(href) = el.value().attr("href") {
                return href.to_string();
            }
        }
    }
    String::new()
}

fn clean_duckduckgo_url(url: &str) -> String {
    // DuckDuckGo wraps URLs in redirects
    if url.starts_with("//duckduckgo.com/l/?uddg=") {
        if let Some(start) = url.find("uddg=") {
            let encoded = &url[start + 5..];
            if let Some(end) = encoded.find('&') {
                return urlencoding::decode(&encoded[..end])
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| url.to_string());
            }
            return urlencoding::decode(encoded)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| url.to_string());
        }
    }
    url.to_string()
}
