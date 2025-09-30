use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_automation_detection_evasion() {
    let browser = HeadlessWebBrowser::new();
    
    // Test various automation detection patterns
    let test_cases = vec![
        "<html><body>webdriver detected</body></html>",
        "<script>if (navigator.webdriver) alert('bot');</script>",
        "<html>selenium automation detected</html>",
        "<div>challenge-platform verification</div>",
        "<html>cloudflare protection active</html>",
    ];
    
    for html in test_cases {
        let needs_evasion = browser.detect_automation_evasion_needed(html);
        assert!(needs_evasion, "Should detect automation patterns in: {}", html);
    }
    
    // Test clean HTML that shouldn't trigger detection
    let clean_html = "<html><body><h1>Welcome</h1><p>Normal content</p></body></html>";
    let needs_evasion = browser.detect_automation_evasion_needed(clean_html);
    assert!(!needs_evasion, "Should not detect automation in clean HTML");
}
