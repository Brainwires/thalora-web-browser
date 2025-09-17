use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_native_websocket_implementation() {
    println!("🧪 Testing native WebSocket implementation in Boa...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: WebSocket constructor exists
    let result = browser.lock().unwrap().execute_javascript("typeof WebSocket").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket type: {}", value_str);
            assert!(value_str.contains("function"), "WebSocket constructor should exist");
        },
        Err(e) => {
            panic!("Failed to check WebSocket constructor: {:?}", e);
        },
    }

    // Test 2: WebSocket constants
    let result = browser.lock().unwrap().execute_javascript("WebSocket.CONNECTING").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.CONNECTING: {}", value_str);
            assert!(value_str.contains("0"), "WebSocket.CONNECTING should be 0");
        },
        Err(e) => {
            panic!("Failed to check WebSocket.CONNECTING: {:?}", e);
        },
    }

    // Test 3: WebSocket.OPEN constant
    let result = browser.lock().unwrap().execute_javascript("WebSocket.OPEN").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.OPEN: {}", value_str);
            assert!(value_str.contains("1"), "WebSocket.OPEN should be 1");
        },
        Err(e) => {
            panic!("Failed to check WebSocket.OPEN: {:?}", e);
        },
    }

    // Test 4: WebSocket.CLOSING constant
    let result = browser.lock().unwrap().execute_javascript("WebSocket.CLOSING").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.CLOSING: {}", value_str);
            assert!(value_str.contains("2"), "WebSocket.CLOSING should be 2");
        },
        Err(e) => {
            panic!("Failed to check WebSocket.CLOSING: {:?}", e);
        },
    }

    // Test 5: WebSocket.CLOSED constant
    let result = browser.lock().unwrap().execute_javascript("WebSocket.CLOSED").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.CLOSED: {}", value_str);
            assert!(value_str.contains("3"), "WebSocket.CLOSED should be 3");
        },
        Err(e) => {
            panic!("Failed to check WebSocket.CLOSED: {:?}", e);
        },
    }

    // Test 6: Create WebSocket instance
    let js_code = r#"
        try {
            var ws = new WebSocket("ws://localhost:8080");
            'success';
        } catch (e) {
            'error: ' + e.message + ' | ' + e.name;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket instance creation: {}", value_str);
            if value_str.contains("error") {
                panic!("WebSocket instance creation failed: {}", value_str);
            } else if value_str.contains("success") {
                println!("✅ WebSocket instance created successfully");
            } else {
                panic!("Unexpected result from WebSocket instance creation: {}", value_str);
            }
        },
        Err(e) => {
            panic!("Failed to create WebSocket instance: {:?}", e);
        },
    }

    // Test 7: WebSocket properties
    let js_code = r#"
        try {
            var ws = new WebSocket("wss://echo.websocket.org");
            JSON.stringify({
                url: ws.url,
                readyState: ws.readyState,
                bufferedAmount: ws.bufferedAmount,
                protocol: ws.protocol,
                extensions: ws.extensions
            });
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket properties: {}", value_str);
            assert!(!value_str.contains("error"), "WebSocket properties access should not error: {}", value_str);
            assert!(value_str.contains("url"), "WebSocket should have url property");
            assert!(value_str.contains("readyState"), "WebSocket should have readyState property");
        },
        Err(e) => {
            panic!("Failed to access WebSocket properties: {:?}", e);
        },
    }

    // Test 8: WebSocket methods
    let js_code = r#"
        try {
            var ws = new WebSocket("wss://echo.websocket.org");
            var methods = {
                send: typeof ws.send,
                close: typeof ws.close
            };
            JSON.stringify(methods);
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket methods: {}", value_str);
            assert!(!value_str.contains("error"), "WebSocket methods access should not error: {}", value_str);
            assert!(value_str.contains("function"), "WebSocket methods should be functions");
        },
        Err(e) => {
            panic!("Failed to access WebSocket methods: {:?}", e);
        },
    }

    println!("🎉 All WebSocket tests PASSED - Native implementation working!");
}