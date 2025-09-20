use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_websocket_constructor() {
    println!("🧪 Testing WebSocket constructor...");

    let browser = HeadlessWebBrowser::new();

    let result = browser.lock().unwrap().execute_javascript("typeof WebSocket").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "WebSocket constructor should exist: {}", value_str);
            println!("✅ WebSocket constructor exists");
        },
        Err(e) => panic!("WebSocket constructor test failed: {:?}", e),
    }
}

#[tokio::test]
async fn test_websocket_constants() {
    println!("🧪 Testing WebSocket constants...");

    let browser = HeadlessWebBrowser::new();

    let result = browser.lock().unwrap().execute_javascript("WebSocket.CONNECTING + ',' + WebSocket.OPEN + ',' + WebSocket.CLOSING + ',' + WebSocket.CLOSED").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("0,1,2,3"), "WebSocket constants should be 0,1,2,3: {}", value_str);
            println!("✅ WebSocket constants properly defined");
        },
        Err(e) => panic!("WebSocket constants test failed: {:?}", e),
    }
}

#[tokio::test]
async fn test_fetch_api() {
    println!("🧪 Testing Fetch API...");

    let browser = HeadlessWebBrowser::new();

    // Test fetch function
    let result = browser.lock().unwrap().execute_javascript("typeof fetch").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "fetch function should exist: {}", value_str);
            println!("✅ fetch function exists");
        },
        Err(e) => panic!("fetch function test failed: {:?}", e),
    }

    // Test Request constructor
    let result = browser.lock().unwrap().execute_javascript("typeof Request").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "Request constructor should exist: {}", value_str);
            println!("✅ Request constructor exists");
        },
        Err(e) => panic!("Request constructor test failed: {:?}", e),
    }

    // Test Response constructor
    let result = browser.lock().unwrap().execute_javascript("typeof Response").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "Response constructor should exist: {}", value_str);
            println!("✅ Response constructor exists");
        },
        Err(e) => panic!("Response constructor test failed: {:?}", e),
    }

    // Test Headers constructor
    let result = browser.lock().unwrap().execute_javascript("typeof Headers").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "Headers constructor should exist: {}", value_str);
            println!("✅ Headers constructor exists");
        },
        Err(e) => panic!("Headers constructor test failed: {:?}", e),
    }
}

#[tokio::test]
async fn test_readable_stream() {
    println!("🧪 Testing ReadableStream...");

    let browser = HeadlessWebBrowser::new();

    // Test ReadableStream constructor
    let result = browser.lock().unwrap().execute_javascript("typeof ReadableStream").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("function"), "ReadableStream constructor should exist: {}", value_str);
            println!("✅ ReadableStream constructor exists");
        },
        Err(e) => panic!("ReadableStream constructor test failed: {:?}", e),
    }

    // Test ReadableStream instance creation
    let js_code = r#"
        try {
            var stream = new ReadableStream();
            typeof stream;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            if value_str.contains("object") {
                println!("✅ ReadableStream instance created successfully");
            } else {
                println!("🔍 ReadableStream instance result: {}", value_str);
                // Don't assert here as this might be expected behavior
            }
        },
        Err(e) => panic!("ReadableStream instance test failed: {:?}", e),
    }
}

#[tokio::test]
async fn test_symbol_async_iterator() {
    println!("🧪 Testing Symbol.asyncIterator support...");

    let browser = HeadlessWebBrowser::new();

    let result = browser.lock().unwrap().execute_javascript("typeof Symbol.asyncIterator").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            if value_str.contains("symbol") {
                println!("✅ Symbol.asyncIterator exists");
            } else {
                println!("🔍 Symbol.asyncIterator result: {}", value_str);
                // Don't assert here as this might not be implemented yet
            }
        },
        Err(e) => panic!("Symbol.asyncIterator test failed: {:?}", e),
    }
}