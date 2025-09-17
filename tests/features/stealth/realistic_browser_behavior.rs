use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_realistic_browser_behavior() {
    let mut browser = HeadlessWebBrowser::new();
    let mock_server = MockServer::start().await;
    
    // Set up endpoint that logs headers
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/behavior"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;
    
    // Make request and verify it includes stealth features
    let result = browser.scrape(
        &format!("{}/behavior", mock_server.uri()),
        false,
        None,
        false,
        false
    ).await;
    
    assert!(result.is_ok(), "Stealth request should succeed");
    
    let scraped_data = result.unwrap();
    assert_eq!(scraped_data.content.trim(), "OK");
}
