use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_stealth_config_defaults() {
    let browser = HeadlessWebBrowser::new();
    
    // Test that stealth config has reasonable defaults
    let config = &browser.stealth_config;
    
    assert_eq!(config.viewport_width, 1920);
    assert_eq!(config.viewport_height, 1080);
    assert_eq!(config.device_pixel_ratio, 1.0);
    assert!(!config.languages.is_empty());
    assert!(!config.timezone.is_empty());
    assert!(!config.webgl_vendor.is_empty());
    assert!(!config.webgl_renderer.is_empty());
    assert!(!config.platform.is_empty());
    assert!(config.hardware_concurrency > 0);
    assert!(config.memory > 0);
    assert!(config.random_delays);
}
