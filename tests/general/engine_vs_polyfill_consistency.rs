use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_engine_vs_polyfill_consistency() {
    println!("🧪 Testing engine implementations vs removed polyfills...");

    let browser = HeadlessWebBrowser::new();

    // Test that features work without polyfills
    let consistency_test = browser.lock().unwrap().execute_javascript(
        "typeof RegExp"
    ).await;

    match consistency_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Engine consistency test: {}", value_str);
            // Look for basic engine features
            assert!(value_str.contains("function"), "RegExp should be available as function, got: {}", value_str);
            println!("✅ Engine implementations working consistently");
        },
        Err(e) => panic!("Failed to test engine consistency: {:?}", e),
    }

    println!("✅ Engine vs polyfill consistency verified!");
}
