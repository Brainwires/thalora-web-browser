use thalora::HeadlessWebBrowser;

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
