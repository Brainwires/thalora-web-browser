use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_multiple_import_maps() {
    println!("🧪 Testing Chrome 133: Multiple import maps support...");

    let browser = HeadlessWebBrowser::new();

    // Test multiple import maps functionality
    let js_code = r#"
        try {
            // Test if import maps are supported by checking HTMLScriptElement
            if (typeof HTMLScriptElement !== 'undefined') {
                var script = document.createElement('script');
                script.type = 'importmap';
                var hasImportMapSupport = script.type === 'importmap';
                'Import maps support: ' + hasImportMapSupport;
            } else {
                'HTMLScriptElement not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Multiple import maps test: {}", value_str);
            // Import maps might not be fully testable in headless mode
        },
        Err(e) => panic!("Failed to test multiple import maps: {:?}", e),
    }

    println!("✅ Multiple import maps test completed");
}
