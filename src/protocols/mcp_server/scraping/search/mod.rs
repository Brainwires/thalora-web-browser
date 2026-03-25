pub mod bing;
pub mod duckduckgo;
pub mod google;
pub mod startpage;

use anyhow::Result;

use super::types::{ImageSearchResults, SearchResults};

/// Perform web search using the specified search engine
pub async fn perform_search(
    query: &str,
    num_results: usize,
    search_engine: &str,
) -> Result<SearchResults> {
    eprintln!(
        "🔍 DEBUG: perform_web_search called with engine: {}",
        search_engine
    );
    match search_engine {
        "duckduckgo" => {
            eprintln!("🔍 DEBUG: Calling search_duckduckgo");
            duckduckgo::search(query, num_results).await
        }
        "bing" => {
            eprintln!("🔍 DEBUG: Calling search_bing");
            bing::search(query, num_results).await
        }
        "google" => {
            eprintln!("🔍 DEBUG: Calling search_google");
            google::search(query, num_results).await
        }
        "startpage" => {
            eprintln!("🔍 DEBUG: Calling search_startpage");
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
