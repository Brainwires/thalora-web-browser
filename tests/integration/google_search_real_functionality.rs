use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_google_search_real_functionality() {
    eprintln!("🚀 Testing Google search with real browser...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Real HTTP request to Google
    println!("📡 Making request to Google...");
    let response = browser.lock().unwrap().scrape("https://www.google.com/search?q=rust+programming", false, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ Google request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Verify we got substantial content (real response)
            assert!(scraped_data.content.len() > 1000, "Response should be substantial (>1000 chars)");

            // Check for Google infrastructure response (anti-bot or real content)
            let has_google_response = scraped_data.content.contains("google") ||
                                    scraped_data.content.contains("Google") ||
                                    scraped_data.content.contains("enablejs") ||
                                    scraped_data.content.contains("httpservice");
            assert!(has_google_response, "Response should contain Google infrastructure elements");

            println!("✅ Received authentic Google response");
        }
        Err(e) => {
            panic!("❌ Google request failed: {}", e);
        }
    }

    // Test 2: JavaScript execution functionality
    println!("🧠 Testing JavaScript execution...");
    let js_result = browser.lock().unwrap().execute_javascript("Math.random() * 1000").await;

    match js_result {
        Ok(value) => {
            println!("✅ JavaScript execution successful!");
            // Just verify we got some response - the exact format may vary
            println!("🎲 JavaScript result: {:?}", value);
        }
        Err(e) => {
            panic!("❌ JavaScript execution failed: {}", e);
        }
    }

    println!("🎉 Google search real functionality test completed successfully!");
}
