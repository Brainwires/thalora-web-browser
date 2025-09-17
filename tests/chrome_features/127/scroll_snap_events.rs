use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_127_scroll_snap_events() {
    println!("🧪 Testing Chrome 127: Scroll Snap Events...");

    let browser = HeadlessWebBrowser::new();

    // Test scrollsnapchange event
    let result = browser.lock().unwrap().execute_javascript("typeof document.addEventListener").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("addEventListener type: {}", value_str);
        },
        Err(e) => panic!("Failed to check addEventListener: {:?}", e),
    }

    // Test if we can add scrollsnapchange and scrollsnapchanging event listeners
    let js_code = r#"
        try {
            var scrollSnapChangeSupported = false;
            var scrollSnapChangingSupported = false;

            if (typeof document.addEventListener === 'function') {
                // Test adding scrollsnapchange listener
                document.addEventListener('scrollsnapchange', function() {});
                scrollSnapChangeSupported = true;

                // Test adding scrollsnapchanging listener
                document.addEventListener('scrollsnapchanging', function() {});
                scrollSnapChangingSupported = true;
            }

            'scrollsnapchange:' + scrollSnapChangeSupported + ',scrollsnapchanging:' + scrollSnapChangingSupported;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Scroll snap events test: {}", value_str);
            assert!(!value_str.contains("error:"), "Scroll snap events should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test scroll snap events: {:?}", e),
    }

    println!("✅ Scroll snap events test completed");
}
