use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_engine_error_handling() {
    println!("🧪 Testing engine-level error handling...");

    let browser = HeadlessWebBrowser::new();

    // Test proper error handling for edge cases
    let error_test = browser.lock().unwrap().execute_javascript(r#"
        try {
            // Test basic error handling
            var result1 = new RegExp('valid');
            var result2 = new RegExp('');

            // All should work without throwing errors
            [
                result1 instanceof RegExp,
                result2 instanceof RegExp,
                true
            ]
        } catch (e) {
            ['error', e.message]
        }
    "#).await;

    match error_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Error handling test: {}", value_str);
            assert!(!value_str.contains("error"), "Engine should handle basic cases gracefully, got: {}", value_str);
            println!("✅ Engine error handling verified");
        },
        Err(e) => panic!("Failed to test engine error handling: {:?}", e),
    }

    println!("✅ Engine error handling test passed!");
}
