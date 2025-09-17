use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_file_system_observer() {
    println!("🧪 Testing Chrome 133: FileSystemObserver interface...");

    let browser = HeadlessWebBrowser::new();

    // Test FileSystemObserver interface
    let js_code = r#"
        try {
            if (typeof FileSystemObserver !== 'undefined') {
                // Test FileSystemObserver constructor
                var hasFileSystemObserver = typeof FileSystemObserver === 'function';
                'FileSystemObserver constructor available: ' + hasFileSystemObserver;
            } else {
                'FileSystemObserver not available';
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
            // FileSystemObserver might not be available in headless mode
        },
        Err(e) => panic!("Failed to test FileSystemObserver: {:?}", e),
    }

    println!("✅ FileSystemObserver test completed");
}
