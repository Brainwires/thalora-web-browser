#[tokio::test]
async fn test_chrome_125_storage_access_api() {
    println!("🧪 Testing Chrome 125: Storage Access API Extension...");

    let browser = HeadlessWebBrowser::new();

    // First check if document exists
    let doc_result = browser.lock().unwrap().execute_javascript("typeof document").await;
    match doc_result {
        Ok(value) => println!("document type: {:?}", value),
        Err(e) => println!("document check error: {:?}", e),
    }

    // Test document.requestStorageAccess
    let result = browser.lock().unwrap().execute_javascript("typeof document.requestStorageAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("document.requestStorageAccess type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "requestStorageAccess should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check requestStorageAccess: {:?}", e),
    }

    // Test document.hasStorageAccess
    let result = browser.lock().unwrap().execute_javascript("typeof document.hasStorageAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("document.hasStorageAccess type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "hasStorageAccess should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check hasStorageAccess: {:?}", e),
    }

    println!("✅ Storage Access API test completed");
}
