// Browser jobs run one at a time on the session's own BrowserThread
// (single-threaded), so holding the browser's MutexGuard across .await
// cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

pub mod bing;
pub mod duckduckgo;
pub mod google;
pub mod startpage;

use anyhow::Result;
use futures::FutureExt;

use super::types::{ImageSearchResults, SearchResults};
use crate::engine::browser::BrowserThread;

/// Start a temporary browser for one search on its own thread (dropped with
/// the returned handle).
pub(crate) fn temporary_browser(engine: &str) -> Result<BrowserThread> {
    BrowserThread::spawn(&format!("search-{engine}"))
}

/// Navigate `browser` to `url` and return the page's HTML (scripts are not
/// executed). `full_load` uses the wait-for-load navigation path.
pub(crate) async fn navigate_and_read(
    browser: &BrowserThread,
    url: String,
    full_load: bool,
) -> Result<String> {
    browser
        .call(move |browser| {
            async move {
                let mut guard = browser
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire browser lock"))?;
                // The response body as fetched (TODO(a3): read the live
                // document once scripts run)
                if full_load {
                    guard.navigate_to_with_js_option(&url, true, true).await
                } else {
                    guard.navigate_to_with_options(&url, true).await
                }
            }
            .boxed_local()
        })
        .await?
}

/// Perform web search using the specified search engine
pub async fn perform_search(
    query: &str,
    num_results: usize,
    search_engine: &str,
) -> Result<SearchResults> {
    tracing::debug!("perform_web_search called with engine: {}", search_engine);
    match search_engine {
        "duckduckgo" => {
            tracing::debug!("Calling search_duckduckgo");
            duckduckgo::search(query, num_results).await
        }
        "bing" => {
            tracing::debug!("Calling search_bing");
            bing::search(query, num_results).await
        }
        "google" => {
            tracing::debug!("Calling search_google");
            google::search(query, num_results).await
        }
        "startpage" => {
            tracing::debug!("Calling search_startpage");
            startpage::search(query, num_results).await
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported search engine: {}. Supported engines: google, bing, duckduckgo, startpage",
            search_engine
        )),
    }
}

/// Perform image search using the specified search engine
pub async fn perform_image_search(
    query: &str,
    num_results: usize,
    search_engine: &str,
) -> Result<ImageSearchResults> {
    eprintln!(
        "🖼️ DEBUG: perform_image_search called with engine: {}",
        search_engine
    );
    match search_engine {
        "duckduckgo" => {
            eprintln!("🖼️ DEBUG: Calling image_search_duckduckgo");
            duckduckgo::image_search(query, num_results).await
        }
        "bing" => {
            eprintln!("🖼️ DEBUG: Calling image_search_bing");
            bing::image_search(query, num_results).await
        }
        "google" => {
            eprintln!("🖼️ DEBUG: Calling image_search_google");
            google::image_search(query, num_results).await
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported image search engine: {}. Supported engines: google, bing, duckduckgo",
            search_engine
        )),
    }
}
