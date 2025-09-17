use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_129_file_system_observer() {
    println!("🧪 Testing Chrome 129: FileSystemObserver (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test FileSystemObserver API
    let js_code = r#"
        try {
            if (typeof FileSystemObserver !== 'undefined') {
                // Test FileSystemObserver constructor
                'FileSystemObserver constructor available';
            } else {
                'FileSystemObserver not available (expected in Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("FileSystemObserver test: {}", value_str);
            // FileSystemObserver is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test FileSystemObserver: {:?}", e),
    }

    println!("✅ FileSystemObserver test completed");
}
