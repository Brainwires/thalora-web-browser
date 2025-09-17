use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_history_api_availability() {
    println!("🧪 Testing History API availability...");

    let browser = HeadlessWebBrowser::new();

    // Test if history is available
    let result = browser.lock().unwrap().execute_javascript("typeof history").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("history type: {}", value_str);
            assert!(!value_str.contains("undefined"), "History API should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to execute JavaScript: {:?}", e),
    }

    // Test history.length property
    let result = browser.lock().unwrap().execute_javascript("history.length").await;
    match result {
        Ok(value) => {
            println!("history.length: {:?}", value);
            // Should be a number
            assert!(!format!("{:?}", value).contains("undefined"), "history.length should be defined");
        },
        Err(e) => panic!("Failed to get history.length: {:?}", e),
    }

    // Test history methods exist
    let methods = vec!["back", "forward", "go", "pushState", "replaceState"];
    for method in methods {
        let js_code = format!("typeof history.{}", method);
        let result = browser.lock().unwrap().execute_javascript(&js_code).await;
        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                println!("history.{} type: {}", method, value_str);
                assert!(value_str.contains("function"), "history.{} should be a function, got: {}", method, value_str);
            },
            Err(e) => panic!("Failed to check history.{}: {:?}", method, e),
        }
    }

    println!("✅ History API test completed");
}
