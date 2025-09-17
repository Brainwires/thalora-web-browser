use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::main]
async fn main() {
    println!("🧪 Testing native WebSocket implementation in Boa...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: WebSocket constructor exists
    let result = browser.lock().unwrap().execute_javascript("typeof WebSocket").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket type: {}", value_str);
            if !value_str.contains("function") {
                println!("❌ CRITICAL: WebSocket constructor not found!");
                return;
            } else {
                println!("✅ WebSocket constructor exists");
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to check WebSocket constructor: {:?}", e);
            return;
        },
    }

    // Test 2: WebSocket constants
    let result = browser.lock().unwrap().execute_javascript("WebSocket.CONNECTING").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket.CONNECTING: {}", value_str);
            if !value_str.contains("0") {
                println!("❌ CRITICAL: WebSocket.CONNECTING not 0!");
                return;
            } else {
                println!("✅ WebSocket.CONNECTING = 0");
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to check WebSocket.CONNECTING: {:?}", e);
            return;
        },
    }

    // Test 3: Create WebSocket instance
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
            if value_str.contains("error") {
                println!("❌ CRITICAL: WebSocket instance creation failed: {}", value_str);
                return;
            } else if value_str.contains("object") {
                println!("✅ WebSocket instance created successfully");
            } else {
                println!("❌ CRITICAL: WebSocket instance not an object: {}", value_str);
                return;
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to create WebSocket instance: {:?}", e);
            return;
        },
    }

    // Test 4: WebSocket properties
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
            if value_str.contains("error") {
                println!("❌ CRITICAL: WebSocket properties access failed: {}", value_str);
                return;
            } else {
                println!("✅ WebSocket properties accessible");
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to access WebSocket properties: {:?}", e);
            return;
        },
    }

    // Test 5: WebSocket methods
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
            if value_str.contains("error") {
                println!("❌ CRITICAL: WebSocket methods access failed: {}", value_str);
                return;
            } else if value_str.contains("function") {
                println!("✅ WebSocket methods accessible");
            } else {
                println!("❌ CRITICAL: WebSocket methods not functions: {}", value_str);
                return;
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to access WebSocket methods: {:?}", e);
            return;
        },
    }

    println!("🎉 All WebSocket tests PASSED - Native implementation working!");
}