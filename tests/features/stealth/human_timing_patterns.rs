use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_human_timing_patterns() {
    let mut browser = HeadlessWebBrowser::new();
    let mock_server = MockServer::start().await;
    
    // Set up a simple endpoint
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;
    
    // Measure timing delays between requests
    let mut timings = Vec::new();
    
    for _ in 0..3 {
        let start = Instant::now();
        let _result = browser.scrape(
            &format!("{}/test", mock_server.uri()),
            false,
            None,
            false,
            false
        ).await;
        let elapsed = start.elapsed();
        timings.push(elapsed);
    }
    
    // Should have some variation in timing (not all identical)
    let has_variation = timings.windows(2).any(|pair| {
        let diff = if pair[0] > pair[1] {
            pair[0] - pair[1]
        } else {
            pair[1] - pair[0]
        };
        diff.as_millis() > 50 // More than 50ms difference
    });
    
    assert!(has_variation, "Request timing should show human-like variation");
}
