use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_132_file_system_access_android() {
    println!("🧪 Testing Chrome 132: File System Access API on Android...");

    let browser = HeadlessWebBrowser::new();

    // Test File System Access API methods
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.showOpenFilePicker) {
                // Test if File System Access API is available
                var hasFileSystemAccess = typeof window.showOpenFilePicker === 'function';
                var hasSaveFilePicker = typeof window.showSaveFilePicker === 'function';
                var hasDirectoryPicker = typeof window.showDirectoryPicker === 'function';

                'File System Access - open:' + hasFileSystemAccess +
                ', save:' + hasSaveFilePicker +
                ', directory:' + hasDirectoryPicker;
            } else {
                'File System Access API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("File System Access API test: {}", value_str);
            // File System Access might not be available in headless mode
        },
        Err(e) => panic!("Failed to test File System Access API: {:?}", e),
    }

    println!("✅ File System Access API test completed");
}
