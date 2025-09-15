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

#[tokio::test]
async fn test_history_api_functionality() {
    println!("🧪 Testing History API functionality...");

    let browser = HeadlessWebBrowser::new();

    // Test pushState
    let result = browser.lock().unwrap().execute_javascript("history.pushState({test: 'data'}, 'Test', '/test'); 'success'").await;
    match result {
        Ok(value) => {
            println!("pushState result: {:?}", value);
            assert!(format!("{:?}", value).contains("success"), "pushState should execute without error");
        },
        Err(e) => panic!("pushState failed: {:?}", e),
    }

    // Test state property
    let result = browser.lock().unwrap().execute_javascript("history.state").await;
    match result {
        Ok(value) => {
            println!("history.state: {:?}", value);
            // Should not be undefined
            assert!(!format!("{:?}", value).contains("undefined"), "history.state should be defined after pushState");
        },
        Err(e) => panic!("Failed to get history.state: {:?}", e),
    }

    println!("✅ History API functionality test completed");
}

#[tokio::test]
async fn test_window_history_vs_global_history() {
    println!("🧪 Testing window.history vs global history...");

    let browser = HeadlessWebBrowser::new();

    // Test window.history
    let result = browser.lock().unwrap().execute_javascript("typeof window.history").await;
    match result {
        Ok(value) => {
            println!("window.history type: {:?}", value);
        },
        Err(e) => println!("window.history error: {:?}", e),
    }

    // Test global history
    let result = browser.lock().unwrap().execute_javascript("typeof history").await;
    match result {
        Ok(value) => {
            println!("global history type: {:?}", value);
        },
        Err(e) => println!("global history error: {:?}", e),
    }

    // Test if they're the same object
    let result = browser.lock().unwrap().execute_javascript("window.history === history").await;
    match result {
        Ok(value) => {
            println!("window.history === history: {:?}", value);
        },
        Err(e) => println!("comparison error: {:?}", e),
    }

    println!("✅ Window vs global history test completed");
}