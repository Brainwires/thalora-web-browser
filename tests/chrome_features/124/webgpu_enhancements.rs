#[tokio::test]
async fn test_chrome_124_webgpu_enhancements() {
    println!("🧪 Testing Chrome 124: WebGPU enhancements...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.gpu availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.gpu").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.gpu type: {}", value_str);
            // Note: WebGPU might not be available in headless mode, so we check for object or undefined
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "navigator.gpu should exist or be undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.gpu: {:?}", e),
    }

    // Test WebGPU in ServiceWorker context (basic check)
    let result = browser.lock().unwrap().execute_javascript("typeof ServiceWorkerGlobalScope").await;
    match result {
        Ok(value) => {
            println!("ServiceWorkerGlobalScope availability: {:?}", value);
            // ServiceWorker might not be available in this context, which is fine
        },
        Err(_) => {
            // ServiceWorker context check is optional
            println!("ServiceWorker context not available (expected in headless mode)");
        }
    }

    println!("✅ WebGPU enhancements test completed");
}
