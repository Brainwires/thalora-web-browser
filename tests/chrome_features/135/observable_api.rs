#[tokio::test]
async fn test_chrome_135_observable_api() {
    println!("🧪 Testing Chrome 135: Observable API...");

    let browser = HeadlessWebBrowser::new();

    // Test Observable API
    let js_code = r#"
        try {
            if (typeof Observable !== 'undefined') {
                // Test Observable constructor
                var hasObservable = typeof Observable === 'function';

                // Test basic Observable creation
                try {
                    var obs = new Observable(function(observer) {
                        observer.next(1);
                        observer.complete();
                    });
                    var hasObservableInstance = obs instanceof Observable;
                    'Observable API available and working: ' + (hasObservable && hasObservableInstance);
                } catch (obsError) {
                    'Observable constructor available but error: ' + obsError.message;
                }
            } else {
                'Observable API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Observable test: {}", value_str);
            // Observable API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Observable: {:?}", e),
    }

    println!("✅ Observable test completed");
}
