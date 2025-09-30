use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_canvas_fingerprinting() {
    let browser = HeadlessWebBrowser::new();
    
    // Generate multiple canvas fingerprints
    let fingerprints: Vec<String> = (0..5)
        .map(|_| browser.simulate_canvas_fingerprint())
        .collect();
    
    // Fingerprints should have some variation
    let unique_prints: std::collections::HashSet<&String> = fingerprints.iter().collect();
    assert!(unique_prints.len() > 1, "Canvas fingerprints should vary");
    
    // All should be valid fingerprint format
    for print in &fingerprints {
        assert!(!print.is_empty(), "Canvas fingerprint should not be empty");
        assert!(print.len() > 10, "Canvas fingerprint should be substantial");
    }
}
