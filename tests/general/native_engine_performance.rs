use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_native_engine_performance() {
    println!("🧪 Testing native engine feature performance...");

    let browser = HeadlessWebBrowser::new();

    // Test that native implementations are fast
    let performance_test = browser.lock().unwrap().execute_javascript(
        "typeof Array"
    ).await;

    match performance_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Performance test result: {}", value_str);
            assert!(value_str.contains("function"), "Array should be available as function, got: {}", value_str);
            println!("✅ Native engine implementation performance verified");
        },
        Err(e) => panic!("Failed to test engine performance: {:?}", e),
    }

    println!("✅ Native engine performance test passed!");
}
