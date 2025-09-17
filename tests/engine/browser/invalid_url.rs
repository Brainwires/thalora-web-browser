use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_invalid_url() {
    let mut scraper = HeadlessWebBrowser::new();
    let result = scraper.scrape("not-a-valid-url", false, None, false, false).await;
    
    assert!(result.is_err());
}
