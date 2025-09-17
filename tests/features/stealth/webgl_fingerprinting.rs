use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_webgl_fingerprinting() {
    let browser = HeadlessWebBrowser::new();
    
    // Test WebGL fingerprint generation
    let (vendor, renderer) = browser.simulate_webgl_fingerprint();
    
    // Should return realistic WebGL info
    assert!(!vendor.is_empty(), "WebGL vendor should not be empty");
    assert!(!renderer.is_empty(), "WebGL renderer should not be empty");
    
    // Should contain expected patterns
    assert!(
        vendor.contains("Google Inc.") || vendor.contains("NVIDIA") || vendor.contains("Intel"),
        "WebGL vendor should be realistic: {}", vendor
    );
    
    assert!(
        renderer.contains("ANGLE") || renderer.contains("OpenGL") || renderer.contains("Direct3D"),
        "WebGL renderer should be realistic: {}", renderer
    );
}
