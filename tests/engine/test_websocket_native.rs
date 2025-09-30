use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_websocket_constructor() {
    println!("🧪 Testing WebSocket constructor...");

    let browser = HeadlessWebBrowser::new();

    let result = browser.lock().unwrap().execute_javascript("typeof WebSocket").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket type: {}", value_str);
            assert!(value_str.contains("function"), "WebSocket constructor should exist");
            println!("✅ WebSocket constructor exists");
        },
        Err(e) => {
            panic!("Failed to check WebSocket constructor: {:?}", e);
        },
    }
}

#[tokio::test]
async fn test_websocket_constants() {
    println!("🧪 Testing WebSocket constants...");

    let browser = HeadlessWebBrowser::new();

    let result = browser.lock().unwrap().execute_javascript("WebSocket.CONNECTING").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.CONNECTING: {}", value_str);
            assert!(value_str.contains("0"), "WebSocket.CONNECTING should be 0");
            println!("✅ WebSocket.CONNECTING = 0");
        },
        Err(e) => {
            panic!("Failed to check WebSocket.CONNECTING: {:?}", e);
        },
    }
}

#[tokio::test]
async fn test_websocket_instance_creation() {
    println!("🧪 Testing WebSocket instance creation...");

    let browser = HeadlessWebBrowser::new();

    let js_code = r#"
        try {
            var ws = new WebSocket("wss://echo.websocket.org");
            typeof ws;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket instance creation: {}", value_str);
            assert!(!value_str.contains("error"), "WebSocket instance creation should not error: {}", value_str);
            assert!(value_str.contains("object"), "WebSocket instance should be an object: {}", value_str);
            println!("✅ WebSocket instance created successfully");
        },
        Err(e) => {
            panic!("Failed to create WebSocket instance: {:?}", e);
        },
    }
}

#[tokio::test]
async fn test_websocket_properties() {
    println!("🧪 Testing WebSocket properties...");

    let browser = HeadlessWebBrowser::new();

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
            println!("✅ WebSocket properties accessible");
        },
        Err(e) => {
            panic!("Failed to access WebSocket properties: {:?}", e);
        },
    }
}

#[tokio::test]
async fn test_websocket_methods() {
    println!("🧪 Testing WebSocket methods...");

    let browser = HeadlessWebBrowser::new();

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
            assert!(value_str.contains("function"), "WebSocket methods should be functions: {}", value_str);
            println!("✅ WebSocket methods accessible");
        },
        Err(e) => {
            panic!("Failed to access WebSocket methods: {:?}", e);
        },
    }
}