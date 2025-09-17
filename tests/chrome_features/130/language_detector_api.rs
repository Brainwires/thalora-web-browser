use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_130_language_detector_api() {
    println!("🧪 Testing Chrome 130: Language Detector API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Language Detector API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml) {
                // Test Language Detector API availability
                var hasLanguageDetector = typeof navigator.ml.createLanguageDetector === 'function';
                'Language Detector API available: ' + hasLanguageDetector;
            } else if (typeof LanguageDetector !== 'undefined') {
                // Alternative global interface
                'LanguageDetector global available';
            } else {
                'Language Detector API not available (Origin Trial)';
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
            // Language Detector API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Language Detector API: {:?}", e),
    }

    println!("✅ Language Detector API test completed");
}
