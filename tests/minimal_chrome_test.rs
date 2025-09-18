use boa_engine::{Context, Source};

#[tokio::test]
async fn test_minimal_javascript_execution() {
    println!("🧪 Testing minimal JavaScript execution...");

    let mut context = Context::default();

    // Test basic JavaScript execution without browser initialization
    let source = Source::from_bytes("typeof WebSocketStream");
    match context.eval(source) {
        Ok(value) => {
            let value_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("WebSocketStream type: {}", value_str);
            assert!(value_str.contains("undefined"), "Expected undefined for uninitialized WebSocketStream, got: {}", value_str);
        },
        Err(e) => panic!("Failed to execute basic JavaScript: {:?}", e),
    }

    println!("✅ Minimal JavaScript execution test completed");
}

#[tokio::test]
async fn test_javascript_with_polyfills() {
    println!("🧪 Testing JavaScript execution with basic polyfills...");

    let mut context = Context::default();

    // Add basic WebSocketStream polyfill
    let polyfill_source = Source::from_bytes(r#"
        globalThis.WebSocketStream = function WebSocketStream(url, options) {
            this.url = url;
            this.options = options || {};
        };
        WebSocketStream.prototype.constructor = WebSocketStream;
    "#);

    match context.eval(polyfill_source) {
        Ok(_) => {
            println!("✅ Polyfill installed successfully");
        },
        Err(e) => panic!("Failed to install polyfill: {:?}", e),
    }

    // Test WebSocketStream availability
    let test_source = Source::from_bytes("typeof WebSocketStream");
    match context.eval(test_source) {
        Ok(value) => {
            let value_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
            println!("WebSocketStream type: {}", value_str);
            assert!(value_str.contains("function"), "WebSocketStream should be available as constructor, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check WebSocketStream: {:?}", e),
    }

    println!("✅ JavaScript with polyfills test completed");
}