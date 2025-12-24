use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: Option<String>,
    pub search_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: usize,
}

/// Image search result
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSearchResult {
    /// Title/alt text of the image
    pub title: String,
    /// URL of the full-size image
    pub image_url: String,
    /// URL of the thumbnail
    pub thumbnail_url: Option<String>,
    /// URL of the source page
    pub source_url: String,
    /// Width of the image (if available)
    pub width: Option<u32>,
    /// Height of the image (if available)
    pub height: Option<u32>,
    /// Position in search results
    pub position: usize,
}

/// Image search results collection
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSearchResults {
    pub query: String,
    pub results: Vec<ImageSearchResult>,
    pub total_results: Option<String>,
}
