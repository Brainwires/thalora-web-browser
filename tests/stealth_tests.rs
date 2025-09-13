use synaptic::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_user_agent_rotation() {
    let browser = HeadlessWebBrowser::new();
    
    // Generate multiple user agents and verify they're different
    let user_agents: Vec<String> = (0..10)
        .map(|_| browser.get_random_user_agent())
        .collect();
    
    // Should have some variety in user agents (not all the same)
    let unique_agents: std::collections::HashSet<&String> = user_agents.iter().collect();
    assert!(unique_agents.len() > 1, "User agents should vary");
    
    // All should be realistic browser user agents
    for agent in &user_agents {
        assert!(
            agent.contains("Chrome") || agent.contains("Firefox") || 
            agent.contains("Safari") || agent.contains("Edge"),
            "User agent should be realistic: {}", agent
        );
    }
}

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