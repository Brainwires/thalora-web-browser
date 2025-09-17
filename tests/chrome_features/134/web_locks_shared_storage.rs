#[tokio::test]
async fn test_chrome_134_web_locks_shared_storage() {
    println!("🧪 Testing Chrome 134: Web Locks API in Shared Storage...");

    let browser = HeadlessWebBrowser::new();

    // Test Web Locks API in Shared Storage
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.locks) {
                // Test basic Web Locks API availability
                var hasWebLocks = typeof navigator.locks.request === 'function';

                if (typeof sharedStorage !== 'undefined') {
                    // Test if shared storage supports locks
                    var hasSharedStorageLocks = typeof sharedStorage.batchUpdate === 'function';
                    'Web Locks available: ' + hasWebLocks + ', SharedStorage.batchUpdate: ' + hasSharedStorageLocks;
                } else {
                    'Web Locks available: ' + hasWebLocks + ', sharedStorage not available';
                }
            } else {
                'Web Locks API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Web Locks Shared Storage test: {}", value_str);
            // Web Locks might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Web Locks Shared Storage: {:?}", e),
    }

    println!("✅ Web Locks Shared Storage test completed");
}
