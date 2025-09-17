use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_http_client_functionality() {
    println!("🔧 Testing HTTP client with httpbin.org...");

    let browser = HeadlessWebBrowser::new();

    // Test simple HTTP request
    let response = browser.lock().unwrap().scrape("https://httpbin.org/html", false, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ HTTP request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Verify we got substantial content
            assert!(scraped_data.content.len() > 100, "Response should be substantial");

            // Check for HTML or meaningful content (httpbin sometimes returns different formats)
            let has_meaningful_content = scraped_data.content.contains("<html") ||
                                        scraped_data.content.contains("Herman Melville") ||
                                        scraped_data.content.contains("httpbin") ||
                                        scraped_data.content.len() > 1000;

            if !has_meaningful_content {
                println!("⚠️  Unexpected response format - showing first 200 chars:");
                println!("{}", &scraped_data.content[..scraped_data.content.len().min(200)]);
            }

            assert!(has_meaningful_content, "Response should contain meaningful content");

            println!("✅ HTTP client working correctly");
        }
        Err(e) => {
            panic!("❌ HTTP request failed: {}", e);
        }
    }
}
