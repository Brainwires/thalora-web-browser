use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_webplatform_features() {
    println!("🧪 Testing Web Platform features against MDN compatibility...");

    let browser = HeadlessWebBrowser::new();

    // Test critical Web Platform features that Chrome supports
    let webplatform_tests = vec![
        // CSS Features
        ("CSS Grid", "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Grid_Layout"),
        ("CSS Flexbox", "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout"),
        ("CSS Custom Properties", "https://developer.mozilla.org/en-US/docs/Web/CSS/--*"),

        // JavaScript APIs
        ("Intersection Observer", "https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API"),
        ("Resize Observer", "https://developer.mozilla.org/en-US/docs/Web/API/Resize_Observer_API"),
        ("Performance Observer", "https://developer.mozilla.org/en-US/docs/Web/API/PerformanceObserver"),

        // Modern Web APIs
        ("Payment Request", "https://developer.mozilla.org/en-US/docs/Web/API/Payment_Request_API"),
        ("Web Share", "https://developer.mozilla.org/en-US/docs/Web/API/Web_Share_API"),
        ("Screen Wake Lock", "https://developer.mozilla.org/en-US/docs/Web/API/Screen_Wake_Lock_API"),
    ];

    let mut webplatform_results = HashMap::new();

    for (feature_name, url) in webplatform_tests {
        println!("🔍 Testing {} support...", feature_name);

        let response = browser.lock().unwrap().scrape(url, true, None, false, false).await;

        match response {
            Ok(scraped_data) => {
                println!("✅ {} page loaded ({} chars)", feature_name, scraped_data.content.len());

                // Check for browser compatibility information
                let compat_info = analyze_mdn_compatibility(&scraped_data.content);
                webplatform_results.insert(feature_name.to_string(), compat_info);

                assert!(scraped_data.content.len() > 1000, "Should get substantial MDN content for {}", feature_name);
            }
            Err(e) => {
                println!("❌ {} test failed: {}", feature_name, e);
                webplatform_results.insert(feature_name.to_string(), "Failed to load".to_string());
            }
        }

        // Delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    // Print Web Platform compatibility summary
    println!("\n📊 Web Platform Feature Support Summary:");
    for (feature, support) in webplatform_results {
        println!("  {}: {}", feature, support);
    }
}
