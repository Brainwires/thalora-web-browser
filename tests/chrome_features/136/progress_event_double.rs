use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_progress_event_double() {
    println!("🧪 Testing Chrome 136: ProgressEvent double type...");

    let browser = HeadlessWebBrowser::new();

    // Test ProgressEvent with double type for loaded and total
    let js_code = r#"
        try {
            if (typeof ProgressEvent !== 'undefined') {
                // Test ProgressEvent constructor
                var progressEvent = new ProgressEvent('progress', {
                    lengthComputable: true,
                    loaded: 50.5,  // Chrome 136: now supports double
                    total: 100.0   // Chrome 136: now supports double
                });

                var hasLoadedTotal = 'loaded' in progressEvent && 'total' in progressEvent;
                var loadedValue = progressEvent.loaded;
                var totalValue = progressEvent.total;

                'ProgressEvent double type support: ' + (hasLoadedTotal && loadedValue === 50.5 && totalValue === 100);
            } else {
                'ProgressEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ProgressEvent double type test: {}", value_str);
            assert!(!value_str.contains("error:"), "ProgressEvent double type should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test ProgressEvent double type: {:?}", e),
    }

    println!("✅ ProgressEvent double type test completed");
}
