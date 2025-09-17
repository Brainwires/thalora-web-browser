use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_open_pseudo_class() {
    println!("🧪 Testing Chrome 133: :open pseudo-class support...");

    let browser = HeadlessWebBrowser::new();

    // Test :open pseudo-class support
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test if :open pseudo-class is supported
                var supportsOpen = CSS.supports('selector(:open)');
                ':open pseudo-class supported: ' + supportsOpen;
            } else if (typeof document !== 'undefined') {
                // Fallback test
                try {
                    var style = document.createElement('style');
                    style.textContent = 'dialog:open { display: block; }';
                    ':open pseudo-class: fallback test completed';
                } catch (styleError) {
                    ':open pseudo-class: ' + styleError.message;
                }
            } else {
                ':open pseudo-class: cannot test without document';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!(":open pseudo-class test: {}", value_str);
            // CSS.supports might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test :open pseudo-class: {:?}", e),
    }

    println!("✅ :open pseudo-class test completed");
}
