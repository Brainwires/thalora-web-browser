#[tokio::test]
async fn test_worker_constructor_availability() {
    println!("🧪 Testing Worker constructor availability...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            // Test Worker constructor exists
            var workerAvailable = typeof Worker === 'function';

            if (workerAvailable) {
                'Worker constructor available: ' + workerAvailable;
            } else {
                'Worker constructor not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker availability test: {}", value_str);
            assert!(value_str.contains("Worker constructor available: true"));
        },
        Err(e) => panic!("Failed to test Worker availability: {:?}", e),
    }

    println!("✅ Worker constructor availability test completed");
}

#[tokio::test]
async fn test_worker_constructor_requires_new() {
    println!("🧪 Testing Worker constructor requires 'new'...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            // Test that calling Worker without 'new' throws an error
            Worker('https://example.com/worker.js');
            'error: Worker() should have thrown';
        } catch (e) {
            if (e.message && e.message.includes("requires 'new'")) {
                'success: Worker requires new keyword';
            } else {
                'error: Wrong error message: ' + e.message;
            }
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker 'new' requirement test: {}", value_str);
            assert!(value_str.contains("success: Worker requires new keyword"));
        },
        Err(e) => panic!("Failed to test Worker 'new' requirement: {:?}", e),
    }

    println!("✅ Worker 'new' requirement test completed");
}

#[tokio::test]
async fn test_worker_constructor_with_valid_url() {
    println!("🧪 Testing Worker constructor with valid URL...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            // Test Worker creation with valid URL
            var worker = new Worker('https://example.com/worker.js');

            if (worker && typeof worker === 'object') {
                'success: Worker created successfully';
            } else {
                'error: Worker creation failed';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker creation test: {}", value_str);
            assert!(value_str.contains("success: Worker created successfully"));
        },
        Err(e) => panic!("Failed to test Worker creation: {:?}", e),
    }

    println!("✅ Worker creation test completed");
}

#[tokio::test]
async fn test_worker_constructor_with_invalid_url() {
    println!("🧪 Testing Worker constructor with invalid URL...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            // Test Worker creation with invalid URL
            var worker = new Worker('not-a-valid-url');
            'error: Worker should have thrown with invalid URL';
        } catch (e) {
            if (e.message && e.message.includes("Invalid Worker script URL")) {
                'success: Worker rejected invalid URL';
            } else {
                'error: Wrong error message: ' + e.message;
            }
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker invalid URL test: {}", value_str);
            assert!(value_str.contains("success: Worker rejected invalid URL"));
        },
        Err(e) => panic!("Failed to test Worker invalid URL: {:?}", e),
    }

    println!("✅ Worker invalid URL test completed");
}

#[tokio::test]
async fn test_worker_has_methods() {
    println!("🧪 Testing Worker has required methods...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            var worker = new Worker('https://example.com/worker.js');

            var hasPostMessage = typeof worker.postMessage === 'function';
            var hasTerminate = typeof worker.terminate === 'function';

            if (hasPostMessage && hasTerminate) {
                'success: Worker has postMessage and terminate methods';
            } else {
                'error: Worker missing methods - postMessage: ' + hasPostMessage + ', terminate: ' + hasTerminate;
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker methods test: {}", value_str);
            assert!(value_str.contains("success: Worker has postMessage and terminate methods"));
        },
        Err(e) => panic!("Failed to test Worker methods: {:?}", e),
    }

    println!("✅ Worker methods test completed");
}

#[tokio::test]
async fn test_worker_script_url_property() {
    println!("🧪 Testing Worker scriptURL property...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            var worker = new Worker('https://example.com/worker.js');

            if (worker.scriptURL === 'https://example.com/worker.js') {
                'success: Worker scriptURL property correct';
            } else {
                'error: Worker scriptURL incorrect: ' + worker.scriptURL;
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker scriptURL test: {}", value_str);
            assert!(value_str.contains("success: Worker scriptURL property correct"));
        },
        Err(e) => panic!("Failed to test Worker scriptURL: {:?}", e),
    }

    println!("✅ Worker scriptURL test completed");
}

#[tokio::test]
async fn test_worker_post_message() {
    println!("🧪 Testing Worker postMessage method...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            var worker = new Worker('https://example.com/worker.js');

            // Test postMessage doesn't throw
            worker.postMessage('hello world');
            worker.postMessage({type: 'test', data: 'value'});

            'success: Worker postMessage methods work';
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker postMessage test: {}", value_str);
            assert!(value_str.contains("success: Worker postMessage methods work"));
        },
        Err(e) => panic!("Failed to test Worker postMessage: {:?}", e),
    }

    println!("✅ Worker postMessage test completed");
}

#[tokio::test]
async fn test_worker_terminate() {
    println!("🧪 Testing Worker terminate method...");

    let browser = thalora::HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            var worker = new Worker('https://example.com/worker.js');

            // Test terminate doesn't throw
            worker.terminate();

            'success: Worker terminate method works';
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Worker terminate test: {}", value_str);
            assert!(value_str.contains("success: Worker terminate method works"));
        },
        Err(e) => panic!("Failed to test Worker terminate: {:?}", e),
    }

    println!("✅ Worker terminate test completed");
}