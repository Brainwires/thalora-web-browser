use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_request_timing_tracking() {
    let mut browser = HeadlessWebBrowser::new();
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/track"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Tracked"))
        .mount(&mock_server)
        .await;
    
    // Make several requests to build timing history
    for i in 0..3 {
        let _result = browser.scrape(
            &format!("{}/track", mock_server.uri()),
            false,
            None,
            false,
            false
        ).await.unwrap();
        
        // Small delay between requests
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    
    // Verify timing tracking is working (this is internal, so we mainly test that it doesn't crash)
    // In a real implementation, we might expose methods to inspect the timing history
}
