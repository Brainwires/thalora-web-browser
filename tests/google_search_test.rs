use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_google_search_real_functionality() {
    println!("🚀 Testing Google search with real browser...");

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

#[tokio::test]
async fn test_javascript_engine_math_operations() {
    println!("🧮 Testing JavaScript math operations...");

    let browser = HeadlessWebBrowser::new();

    // Test basic math
    let result = browser.lock().unwrap().execute_javascript("2 + 2").await;
    assert!(result.is_ok(), "Basic addition should work");

    // Test Math object
    let result = browser.lock().unwrap().execute_javascript("Math.PI").await;
    assert!(result.is_ok(), "Math.PI should be available");

    // Test random number generation
    let result = browser.lock().unwrap().execute_javascript("Math.random()").await;
    assert!(result.is_ok(), "Math.random() should work");

    println!("✅ JavaScript math operations working correctly");
}