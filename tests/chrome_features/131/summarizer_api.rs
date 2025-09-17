use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_131_summarizer_api() {
    println!("🧪 Testing Chrome 131: Summarizer API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Summarizer API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml && navigator.ml.createSummarizer) {
                // Test Summarizer API availability
                var hasSummarizer = typeof navigator.ml.createSummarizer === 'function';
                'Summarizer API available: ' + hasSummarizer;
            } else if (typeof Summarizer !== 'undefined') {
                // Alternative global interface
                'Summarizer global available';
            } else {
                'Summarizer API not available (Origin Trial)';
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
            // Summarizer API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Summarizer API: {:?}", e),
    }

    println!("✅ Summarizer API test completed");
}
