//! Web search module - provides search functionality without requiring full MCP server
//!
//! This module exposes search capabilities for use by external crates
//! without pulling in the full browser infrastructure.
//! Uses reqwest with proper browser headers to avoid bot detection.

use anyhow::Result;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, UPGRADE_INSECURE_REQUESTS,
    USER_AGENT,
};
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

/// Create a configured HTTP client with proper browser-like settings
/// This matches what a real Chrome browser sends to avoid TLS/HTTP fingerprinting
fn create_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .cookie_store(true) // Important for session tracking
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

/// Create standard browser headers that match a real Chrome browser
fn create_browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Use Chrome 120 on Windows - this must match sec-ch-ua headers
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ));

    // Chrome-style Accept header
    headers.insert(ACCEPT, HeaderValue::from_static(
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
    ));

    // Chrome language preferences
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

    // Chrome compression support
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    // Chrome client hints - CRITICAL for passing bot detection
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static("\"Chromium\";v=\"120\", \"Not A(Brand\";v=\"99\""),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"Windows\""),
    );

    // Fetch metadata
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));

    // Upgrade insecure requests
    headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

    // DNT header
    headers.insert("dnt", HeaderValue::from_static("1"));

    headers
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
pub async fn perform_search(
    query: &str,
    num_results: usize,
    search_engine: &str,
) -> Result<SearchResults> {
    let client = create_client()?;
    let headers = create_browser_headers();

    match search_engine {
        "duckduckgo" => search_duckduckgo(&client, &headers, query, num_results).await,
        "bing" => search_bing(&client, &headers, query, num_results).await,
        "google" => search_google(&client, &headers, query, num_results).await,
        "startpage" => search_startpage(&client, &headers, query, num_results).await,
        _ => Err(anyhow::anyhow!(
            "Unsupported search engine: {}. Use: duckduckgo, bing, google, or startpage",
            search_engine
        )),
    }
}

async fn search_duckduckgo(
    client: &reqwest::Client,
    headers: &HeaderMap,
    query: &str,
    num_results: usize,
) -> Result<SearchResults> {
    let search_url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let response = client
        .get(&search_url)
        .headers(headers.clone())
        .send()
        .await?;
    let html = response.text().await?;

    // Check for CAPTCHA - only warn if detected
    if html.contains("anomaly-modal") {
        eprintln!(
            "[thalora::web_search] WARNING: DuckDuckGo CAPTCHA detected, results may be limited"
        );
    }

    parse_duckduckgo_results(&html, query, num_results)
}

async fn search_bing(
    client: &reqwest::Client,
    headers: &HeaderMap,
    query: &str,
    num_results: usize,
) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.bing.com/search?q={}&count={}&FORM=QBLH",
        urlencoding::encode(query),
        num_results
    );

    let response = client
        .get(&search_url)
        .headers(headers.clone())
        .send()
        .await?;
    let html = response.text().await?;

    parse_bing_results(&html, query, num_results)
}

async fn search_google(
    client: &reqwest::Client,
    headers: &HeaderMap,
    query: &str,
    num_results: usize,
) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.google.com/search?q={}&num={}&hl=en&gl=us",
        urlencoding::encode(query),
        num_results
    );

    let response = client
        .get(&search_url)
        .headers(headers.clone())
        .send()
        .await?;
    let html = response.text().await?;

    parse_google_results(&html, query, num_results)
}

async fn search_startpage(
    client: &reqwest::Client,
    headers: &HeaderMap,
    query: &str,
    num_results: usize,
) -> Result<SearchResults> {
    let search_url = format!(
        "https://www.startpage.com/do/search?query={}",
        urlencoding::encode(query)
    );

    let response = client
        .get(&search_url)
        .headers(headers.clone())
        .send()
        .await?;
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

                if !title.is_empty()
                    && !url.is_empty()
                    && !url.starts_with("/search")
                    && !results.iter().any(|r: &SearchResult| r.url == url)
                {
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
