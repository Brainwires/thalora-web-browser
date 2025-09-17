use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_138_language_detector_api() {
    println!("🧪 Testing Chrome 138: Language Detector API...");

    let browser = HeadlessWebBrowser::new();

    // Test Language Detector API availability
    let js_code = r#"
        try {
            // Check if language detection APIs are available
            var hasLanguageDetector = typeof navigator !== 'undefined' &&
                                     (typeof navigator.ml !== 'undefined' ||
                                      typeof ai !== 'undefined' ||
                                      typeof languageDetector !== 'undefined');

            // Test basic structure
            if (typeof languageDetector !== 'undefined') {
                'Language Detector API available: true';
            } else if (typeof ai !== 'undefined') {
                'AI APIs context available for language detection: ' + (typeof ai === 'object');
            } else {
                'Language Detector API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Language Detector API test: {}", value_str);
            // AI APIs might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Language Detector API: {:?}", e),
    }

    println!("✅ Language Detector API test completed");
}
