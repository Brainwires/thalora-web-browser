use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_network_error() {
    let mut scraper = HeadlessWebBrowser::new();
    // Try to connect to a non-existent server
    let result = scraper.scrape("http://localhost:99999", false, None, false, false).await;
    
    assert!(result.is_err());
}
