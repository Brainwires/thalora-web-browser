use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test] 
async fn test_stealth_headers() {
    let browser = HeadlessWebBrowser::new();
    
    // Generate multiple header sets
    let header_sets: Vec<reqwest::header::HeaderMap> = (0..5)
        .map(|_| browser.create_stealth_headers("https://example.com"))
        .collect();
    
    // All should contain essential headers
    for headers in &header_sets {
        assert!(headers.contains_key("accept"), "Should have Accept header");
        assert!(headers.contains_key("accept-language"), "Should have Accept-Language header");
        assert!(headers.contains_key("accept-encoding"), "Should have Accept-Encoding header");
        assert!(headers.contains_key("sec-ch-ua"), "Should have Sec-Ch-Ua header");
    }
    
    // Should show some variation in Accept-Language
    let languages: Vec<String> = header_sets.iter()
        .map(|h| h.get("accept-language").unwrap().to_str().unwrap().to_string())
        .collect();
    
    let unique_languages: std::collections::HashSet<&String> = languages.iter().collect();
    assert!(unique_languages.len() > 1, "Accept-Language headers should vary");
}
