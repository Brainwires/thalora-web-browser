use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_can_i_use_features() {
    println!("🧪 Testing HTTP client by fetching Can I Use websites...");

    let browser = HeadlessWebBrowser::new();

    // Test key modern web features
    let features_to_test = vec![
        ("WebAssembly", "https://caniuse.com/wasm"),
        ("WebRTC", "https://caniuse.com/rtcpeerconnection"),
        ("Service Workers", "https://caniuse.com/serviceworkers"),
        ("WebGL", "https://caniuse.com/webgl"),
        ("Geolocation", "https://caniuse.com/geolocation"),
        ("WebSocket", "https://caniuse.com/websockets"),
    ];

    let mut compatibility_results = HashMap::new();

    for (feature_name, url) in features_to_test {
        println!("🔍 Testing {} support...", feature_name);

        let response = browser.lock().unwrap().scrape(url, true, None, false, false).await;

        match response {
            Ok(scraped_data) => {
                println!("✅ {} page loaded ({} chars)", feature_name, scraped_data.content.len());

                // Analyze support indicators
                let support_level = analyze_caniuse_support(&scraped_data.content);
                compatibility_results.insert(feature_name.to_string(), support_level);

                assert!(scraped_data.content.len() > 1000, "Should get substantial Can I Use content for {}", feature_name);
            }
            Err(e) => {
                println!("❌ {} test failed: {}", feature_name, e);
                compatibility_results.insert(feature_name.to_string(), "Failed to load".to_string());
            }
        }

        // Small delay between requests to be respectful
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    // Print compatibility summary
    println!("\n📊 Browser Compatibility Summary:");
    for (feature, support) in compatibility_results {
        println!("  {}: {}", feature, support);
    }
}
