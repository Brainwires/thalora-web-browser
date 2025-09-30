use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_es6_compatibility() {
    println!("🧪 Testing HTTP client by fetching Kangax ES6 table...");

    let browser = HeadlessWebBrowser::new();

    // Test Kangax ES6 compatibility table
    let response = browser.lock().unwrap().scrape("https://compat-table.github.io/compat-table/es6/", true, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ Kangax ES6 table request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Look for ES6 feature support indicators
            let es6_features = check_es6_features(&scraped_data.content);
            println!("🎯 ES6 features found in table: {}", es6_features.len());

            // Check for specific modern JavaScript features
            let modern_features = vec![
                "arrow functions",
                "template literals",
                "destructuring",
                "default parameters",
                "rest parameters",
                "spread operator",
                "classes",
                "promises",
                "async functions",
                "modules"
            ];

            for feature in modern_features {
                if scraped_data.content.to_lowercase().contains(feature) {
                    println!("  ✅ Found: {}", feature);
                } else {
                    println!("  ❌ Missing: {}", feature);
                }
            }

            assert!(scraped_data.content.len() > 5000, "Should get substantial ES6 compatibility table");
        }
        Err(e) => {
            panic!("❌ Kangax ES6 table request failed: {}", e);
        }
    }
}
