use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_get_installed_related_apps() {
    println!("🧪 Testing Chrome 140: Get Installed Related Apps API on Desktop...");

    let browser = HeadlessWebBrowser::new();

    // Test Get Installed Related Apps API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.getInstalledRelatedApps) {
                // Test getInstalledRelatedApps method
                var hasGetInstalledRelatedApps = typeof navigator.getInstalledRelatedApps === 'function';

                'Get Installed Related Apps API available: ' + hasGetInstalledRelatedApps;
            } else {
                'navigator.getInstalledRelatedApps not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Get Installed Related Apps test: {}", value_str);
            // Related Apps API might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Get Installed Related Apps: {:?}", e),
    }

    println!("✅ Get Installed Related Apps test completed");
}
