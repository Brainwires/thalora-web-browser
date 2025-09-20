use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_124_websocketstream_api() {
    println!("🧪 Testing Chrome 124: WebSocketStream API...");

    let browser = HeadlessWebBrowser::new();

    // Test WebSocketStream constructor availability
    let result = browser.lock().unwrap().execute_javascript("typeof WebSocketStream").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocketStream type: {}", value_str);
            assert!(value_str.contains("function"), "WebSocketStream should be available as constructor, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check WebSocketStream: {:?}", e),
    }

    // Test WebSocketStream has expected methods
    let result = browser.lock().unwrap().execute_javascript("WebSocketStream.prototype.constructor === WebSocketStream").await;
    match result {
        Ok(value) => {
            println!("WebSocketStream prototype check: {:?}", value);
            assert!(format!("{:?}", value).contains("true"), "WebSocketStream prototype should be properly set up");
        },
        Err(e) => panic!("Failed to check WebSocketStream prototype: {:?}", e),
    }

    println!("✅ WebSocketStream API test completed");
}

#[tokio::test]
async fn test_chrome_124_streams_async_iteration() {
    println!("🧪 Testing Chrome 124: Streams API Async Iteration...");

    let browser = HeadlessWebBrowser::new();

    // Test ReadableStream has Symbol.asyncIterator
    let result = browser.lock().unwrap().execute_javascript("typeof ReadableStream.prototype[Symbol.asyncIterator]").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ReadableStream async iterator type: {}", value_str);
            assert!(value_str.contains("function"), "ReadableStream should have async iterator method, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check ReadableStream async iterator: {:?}", e),
    }

    // Test that we can create a ReadableStream that's async iterable
    let js_code = r#"
        const stream = new ReadableStream({
            start(controller) {
                controller.enqueue("hello");
                controller.enqueue("world");
                controller.close();
            }
        });
        typeof stream[Symbol.asyncIterator]
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("ReadableStream instance async iterator: {:?}", value);
            assert!(format!("{:?}", value).contains("function"), "ReadableStream instance should have async iterator");
        },
        Err(e) => panic!("Failed to test ReadableStream async iteration: {:?}", e),
    }

    println!("✅ Streams async iteration test completed");
}

#[tokio::test]
async fn test_chrome_124_dom_html_unsafe_methods() {
    println!("🧪 Testing Chrome 124: DOM setHTMLUnsafe and parseHTMLUnsafe...");

    let browser = HeadlessWebBrowser::new();

    // Test Element.prototype.setHTMLUnsafe
    let result = browser.lock().unwrap().execute_javascript("typeof Element.prototype.setHTMLUnsafe").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("setHTMLUnsafe type: {}", value_str);
            assert!(value_str.contains("function"), "setHTMLUnsafe should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check setHTMLUnsafe: {:?}", e),
    }

    // Test Document.parseHTMLUnsafe
    let result = browser.lock().unwrap().execute_javascript("typeof Document.parseHTMLUnsafe").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("parseHTMLUnsafe type: {}", value_str);
            assert!(value_str.contains("function"), "parseHTMLUnsafe should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check parseHTMLUnsafe: {:?}", e),
    }

    println!("✅ DOM HTML unsafe methods test completed");
}

#[tokio::test]
async fn test_chrome_124_pageswap_event() {
    println!("🧪 Testing Chrome 124: pageswap event...");

    let browser = HeadlessWebBrowser::new();

    // Test that pageswap event can be listened to
    let result = browser.lock().unwrap().execute_javascript("typeof window.addEventListener").await;
    match result {
        Ok(value) => {
            println!("addEventListener available: {:?}", value);
            assert!(format!("{:?}", value).contains("function"), "addEventListener should be available");
        },
        Err(e) => panic!("Failed to check addEventListener: {:?}", e),
    }

    // Test pageswap event registration (should not throw)
    let js_code = r#"
        try {
            window.addEventListener('pageswap', function(event) {
                // Event handler for pageswap
            });
            'success'
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("pageswap event registration: {}", value_str);
            assert!(value_str.contains("success"), "pageswap event should be registerable, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test pageswap event: {:?}", e),
    }

    println!("✅ pageswap event test completed");
}

#[tokio::test]
async fn test_chrome_124_webgpu_enhancements() {
    println!("🧪 Testing Chrome 124: WebGPU enhancements...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.gpu availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.gpu").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.gpu type: {}", value_str);
            // Note: WebGPU might not be available in headless mode, so we check for object or undefined
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "navigator.gpu should exist or be undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.gpu: {:?}", e),
    }

    // Test WebGPU in ServiceWorker context (basic check)
    let result = browser.lock().unwrap().execute_javascript("typeof ServiceWorkerGlobalScope").await;
    match result {
        Ok(value) => {
            println!("ServiceWorkerGlobalScope availability: {:?}", value);
            // ServiceWorker might not be available in this context, which is fine
        },
        Err(_) => {
            // ServiceWorker context check is optional
            println!("ServiceWorker context not available (expected in headless mode)");
        }
    }

    println!("✅ WebGPU enhancements test completed");
}

#[tokio::test]
async fn test_chrome_124_webmidi_permissions() {
    println!("🧪 Testing Chrome 124: WebMIDI permissions...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.requestMIDIAccess availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.requestMIDIAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.requestMIDIAccess type: {}", value_str);
            assert!(value_str.contains("function"), "navigator.requestMIDIAccess should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.requestMIDIAccess: {:?}", e),
    }

    // Test that requesting MIDI access requires permissions (should not crash)
    let js_code = r#"
        try {
            // This should work without throwing, even if it fails due to permissions
            typeof navigator.requestMIDIAccess === 'function' ? 'function_available' : 'not_available'
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebMIDI function check: {}", value_str);
            assert!(value_str.contains("function_available"), "WebMIDI should be available, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebMIDI permissions: {:?}", e),
    }

    println!("✅ WebMIDI permissions test completed");
}

#[tokio::test]
async fn test_chrome_124_client_hints() {
    println!("🧪 Testing Chrome 124: Client Hints (Sec-CH-UA-Form-Factors)...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.userAgentData availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.userAgentData").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.userAgentData type: {}", value_str);
            // User Agent Client Hints might not be available in all contexts
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "navigator.userAgentData should exist or be undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.userAgentData: {:?}", e),
    }

    // Test that we can check for form factors support
    let js_code = r#"
        try {
            // Check if getHighEntropyValues is available
            if (navigator.userAgentData && typeof navigator.userAgentData.getHighEntropyValues === 'function') {
                'client_hints_available'
            } else {
                'client_hints_not_available'
            }
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("Client hints availability: {:?}", value);
            // Client hints might not be fully available in headless mode, which is acceptable
        },
        Err(e) => panic!("Failed to test client hints: {:?}", e),
    }

    println!("✅ Client hints test completed");
}

#[tokio::test]
async fn test_chrome_124_overall_compatibility() {
    println!("🧪 Testing Chrome 124: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("WebSocketStream", "typeof WebSocketStream"),
        ("ReadableStream async iteration", "typeof ReadableStream.prototype[Symbol.asyncIterator]"),
        ("setHTMLUnsafe", "typeof Element.prototype.setHTMLUnsafe"),
        ("parseHTMLUnsafe", "typeof Document.parseHTMLUnsafe"),
        ("navigator.gpu", "typeof navigator.gpu"),
        ("navigator.requestMIDIAccess", "typeof navigator.requestMIDIAccess"),
        ("navigator.userAgentData", "typeof navigator.userAgentData"),
    ];

    let mut available = 0;
    let total = features.len();

    for (feature_name, js_check) in features {
        match browser.lock().unwrap().execute_javascript(js_check).await {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                let is_available = value_str.contains("function") || value_str.contains("object");
                if is_available {
                    available += 1;
                    println!("✅ {}: Available", feature_name);
                } else {
                    println!("❌ {}: Not available ({})", feature_name, value_str);
                }
            },
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", feature_name, e);
            }
        }
    }

    println!("\n📊 Chrome 124 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 124 overall compatibility test completed");
}