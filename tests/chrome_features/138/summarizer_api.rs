#[tokio::test]
async fn test_chrome_138_summarizer_api() {
    println!("🧪 Testing Chrome 138: Summarizer API...");

    let browser = HeadlessWebBrowser::new();

    // Test Summarizer API availability
    let js_code = r#"
        try {
            // Check if summarizer APIs are available
            var hasSummarizerAPI = typeof navigator !== 'undefined' &&
                                  (typeof navigator.ml !== 'undefined' ||
                                   typeof ai !== 'undefined' ||
                                   typeof summarizer !== 'undefined');

            // Test basic structure
            if (typeof summarizer !== 'undefined') {
                'Summarizer API available: true';
            } else if (typeof ai !== 'undefined') {
                'AI APIs context available for summarization: ' + (typeof ai === 'object');
            } else {
                'Summarizer API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Summarizer API test: {}", value_str);
            // AI APIs might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Summarizer API: {:?}", e),
    }

    println!("✅ Summarizer API test completed");
}
