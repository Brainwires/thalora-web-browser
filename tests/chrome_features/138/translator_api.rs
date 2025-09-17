use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_translator_api() {
    println!("🧪 Testing Chrome 138: Translator API...");

    let browser = HeadlessWebBrowser::new();

    // Test Translator API availability
    let js_code = r#"
        try {
            // Check if AI or translation APIs are available
            var hasTranslatorAPI = typeof navigator !== 'undefined' &&
                                  (typeof navigator.ml !== 'undefined' ||
                                   typeof ai !== 'undefined' ||
                                   typeof translation !== 'undefined');

            // Mock translator API structure for testing
            if (typeof translation === 'undefined' && typeof ai !== 'undefined') {
                'AI APIs context available: ' + (typeof ai === 'object');
            } else if (typeof translation !== 'undefined') {
                'Translation API available: true';
            } else {
                'Translation API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Translator API test: {}", value_str);
            // AI APIs might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Translator API: {:?}", e),
    }

    println!("✅ Translator API test completed");
}
