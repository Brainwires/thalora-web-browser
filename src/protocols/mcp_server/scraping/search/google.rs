use anyhow::Result;
use scraper::{Html, Selector};

use crate::protocols::mcp_server::scraping::types::{
    ImageSearchResult, ImageSearchResults, SearchResult, SearchResults,
};
use crate::protocols::mcp_server::scraping::utils::{
    extract_generic_snippet, extract_generic_title, extract_generic_url,
};

pub async fn search(query: &str, num_results: usize) -> Result<SearchResults> {
    tracing::debug!("search_google started");
    let search_url = format!(
        "https://www.google.com/search?q={}&num={}&hl=en&gl=us",
        urlencoding::encode(query),
        num_results
    );
    tracing::debug!("Google search URL: {}", search_url);

    // Temporary browser on its own thread for this stateless search.
    // Google requires JavaScript execution to display search results.
    let temp_browser = super::temporary_browser("google")?;
    let html = super::navigate_and_read(&temp_browser, search_url.clone(), true).await?;
    tracing::debug!("Content retrieved");

    // Check for Google's bot detection challenges
    if html.contains("Our systems have detected unusual traffic")
        || html.contains("why did this happen")
    {
        return Err(anyhow::anyhow!("Google returned bot detection challenge"));
    }

    // Check for reCAPTCHA challenge
    if html.contains("recaptcha") && html.contains("challenge") {
        return Err(anyhow::anyhow!("Google returned reCAPTCHA challenge"));
    }

    // Check for JavaScript challenge/enablejs redirect - but let's try to proceed anyway
    if html.contains("/httpservice/retry/enablejs")
        || (html.contains("<style>table,div,span,p{display:none}</style>")
            && html.contains("refresh"))
    {
        tracing::debug!(
            "Google returned JavaScript challenge page, but attempting to parse anyway"
        );
        tracing::debug!("Challenge page length: {} chars", html.len());
        // Instead of failing, let's try to follow the redirect or parse what we can

        // Try to extract the redirect URL and follow it
        if let Some(start) = html.find("http-equiv=\"refresh\"")
            && let Some(content_start) = html[..start].rfind("content=\"")
        {
            let content_part = &html[content_start + 9..];
            if let Some(url_start) = content_part.find("url=") {
                let url_part = &content_part[url_start + 4..];
                if let Some(url_end) = url_part.find("\"") {
                    let redirect_url = &url_part[..url_end];
                    tracing::debug!("Found redirect URL: {}", redirect_url);

                    // Make a new request to the redirect URL
                    let full_redirect_url = if redirect_url.starts_with("/") {
                        format!("https://www.google.com{}", redirect_url)
                    } else {
                        redirect_url.to_string()
                    };

                    tracing::debug!("Following redirect to: {}", full_redirect_url);

                    // Reuse the same browser to follow the redirect (keeps cookies)
                    let redirect_html =
                        super::navigate_and_read(&temp_browser, full_redirect_url.clone(), true)
                            .await?;

                    tracing::debug!("Redirect response length: {} chars", redirect_html.len());
                    tracing::debug!(
                        "Redirect response preview: {}",
                        redirect_html
                            .char_indices()
                            .nth(500)
                            .map_or(redirect_html.as_str(), |(i, _)| &redirect_html[..i])
                    );

                    // Explicitly drop browser to ensure cleanup
                    drop(temp_browser);

                    // Parse the redirect response instead
                    return parse_results(&redirect_html, query, num_results);
                }
            }
        }

        // If we can't follow the redirect, just try to parse what we have
        tracing::debug!("Could not extract redirect URL, parsing challenge page directly");
    }

    // Let's also check if we got valid search results
    if !html.contains("</html>") || html.len() < 1000 {
        tracing::debug!("Got incomplete HTML response: {} chars", html.len());
        tracing::debug!(
            "HTML content: {}",
            html.char_indices()
                .nth(500)
                .map_or(html.as_str(), |(i, _)| &html[..i])
        );
    }

    // Explicitly drop browser to ensure cleanup
    drop(temp_browser);

    parse_results(&html, query, num_results)
}

pub fn parse_results(html: &str, query: &str, num_results: usize) -> Result<SearchResults> {
    tracing::debug!("Google HTML length: {}", html.len());
    tracing::debug!(
        "Google HTML contains .g class: {}",
        html.contains("class=\"g\"")
    );
    tracing::debug!("Google HTML contains .tF2Cxc: {}", html.contains("tF2Cxc"));
    tracing::debug!("First 500 chars: {}", &html[..html.len().min(500)]);

    let document = Html::parse_document(html);
    let mut results = Vec::new();

    // Google result selectors - multiple approaches since Google changes frequently
    let main_selectors = [
        ".g",                       // Classic Google result container
        "[data-sokoban-container]", // Modern Google result container
        ".tF2Cxc",                  // Current Google search result container
        ".rc",                      // Legacy Google result container
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
                    ".yuRUbf h3 a",
                ];

                // Google URL selectors
                let url_selectors = ["a[href]", ".yuRUbf a", ".r a", "h3 a"];

                // Google snippet selectors
                let snippet_selectors = [".VwiC3b", ".s", ".st", ".IsZvec", "span[data-ved]"];

                let title = extract_generic_title(&element, &title_selectors);
                let mut url = extract_generic_url(&element, &url_selectors);
                let snippet = extract_generic_snippet(&element, &snippet_selectors);

                // Clean up Google redirect URLs
                if url.starts_with("/url?q=")
                    && let Some(actual_url) = url.strip_prefix("/url?q=")
                    && let Some(clean_url) = actual_url.split('&').next()
                {
                    url = urlencoding::decode(clean_url)
                        .unwrap_or_default()
                        .to_string();
                }

                // Make relative URLs absolute
                if url.starts_with("/") {
                    url = format!("https://www.google.com{}", url);
                }

                // Only add if we have valid title and URL, and it's not a Google internal URL
                if !title.is_empty()
                    && !url.is_empty()
                    && url.starts_with("http")
                    && !url.contains("google.com")
                    && !url.contains("youtube.com")
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
    eprintln!(
        "🔍🖼️ Google Images: Searching for '{}' at URL: {}",
        query, search_url
    );

    let temp_browser = super::temporary_browser("google-images")?;
    let html = super::navigate_and_read(&temp_browser, search_url.clone(), true).await?;

    drop(temp_browser);

    parse_image_results(&html, query, num_results)
}

pub fn parse_image_results(
    html: &str,
    query: &str,
    num_results: usize,
) -> Result<ImageSearchResults> {
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

                let image_url = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                    .unwrap_or("");

                // Skip base64 encoded thumbnails and tracking pixels
                if image_url.is_empty()
                    || image_url.starts_with("data:")
                    || image_url.contains("1x1")
                {
                    continue;
                }

                let title = element
                    .value()
                    .attr("alt")
                    .or_else(|| element.value().attr("title"))
                    .unwrap_or("")
                    .to_string();

                results.push(ImageSearchResult {
                    title: if title.is_empty() {
                        format!("Image {}", results.len() + 1)
                    } else {
                        title
                    },
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
