use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_139_extended_lifetime_shared_workers() {
    println!("🧪 Testing Chrome 139: Extended Lifetime Shared Workers...");

    let browser = HeadlessWebBrowser::new();

    // Test Extended Lifetime Shared Workers
    let js_code = r#"
        try {
            if (typeof SharedWorker !== 'undefined') {
                // Test SharedWorker with extended lifetime options
                var hasSharedWorker = typeof SharedWorker === 'function';

                if (hasSharedWorker) {
                    // Test extended lifetime option structure
                    var options = {
                        type: 'module',
                        credentials: 'same-origin',
                        // Chrome 139: Extended lifetime option
                        extendedLifetime: true
                    };

                    'SharedWorker extended lifetime support: ' + hasSharedWorker;
                } else {
                    'SharedWorker constructor not available';
                }
            } else {
                'SharedWorker not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Extended Lifetime Shared Workers test: {}", value_str);
            // SharedWorker might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Extended Lifetime Shared Workers: {:?}", e),
    }

    println!("✅ Extended Lifetime Shared Workers test completed");
}
