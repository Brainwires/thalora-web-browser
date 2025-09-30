use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_html5_compatibility() {
    println!("🧪 Testing HTTP client by fetching html5test.com...");

    let browser = HeadlessWebBrowser::new();

    // Test HTML5Test.com - gives score out of 555 points
    let response = browser.lock().unwrap().scrape("http://html5test.com/", true, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ HTML5Test.com request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Look for the HTML5 score
            if let Some(score_start) = scraped_data.content.find("score") {
                let score_section = &scraped_data.content[score_start..score_start.min(scraped_data.content.len()).min(score_start + 500)];
                println!("📊 Score section: {}", score_section);
            }

            // Check for specific HTML5 features
            let features_found = check_html5_features(&scraped_data.content);
            println!("🎯 HTML5 features detected: {}", features_found.len());
            for feature in features_found {
                println!("  ✅ {}", feature);
            }

            assert!(scraped_data.content.len() > 1000, "Should get substantial HTML5 test content");
        }
        Err(e) => {
            panic!("❌ HTML5Test.com request failed: {}", e);
        }
    }
}
