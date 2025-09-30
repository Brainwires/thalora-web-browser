use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_javascript_engine_math_operations() {
    println!("🧮 Testing JavaScript math operations...");

    let browser = HeadlessWebBrowser::new();

    // Test basic math
    let result = browser.lock().unwrap().execute_javascript("2 + 2").await;
    assert!(result.is_ok(), "Basic addition should work");

    // Test Math object
    let result = browser.lock().unwrap().execute_javascript("Math.PI").await;
    assert!(result.is_ok(), "Math.PI should be available");

    // Test random number generation
    let result = browser.lock().unwrap().execute_javascript("Math.random()").await;
    assert!(result.is_ok(), "Math.random() should work");

    println!("✅ JavaScript math operations working correctly");
}
