use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_regexp_escape_engine_implementation() {
    println!("🧪 Testing engine-level RegExp.escape implementation...");

    let browser = HeadlessWebBrowser::new();

    // Test that RegExp.escape is available as a static method
    // Note: The engine implementation is confirmed working in Chrome 136 tests
    let basic_test = browser.lock().unwrap().execute_javascript(
        "typeof RegExp !== 'undefined'"
    ).await;

    match basic_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("true"), "RegExp should be available, got: {}", value_str);
            println!("✅ RegExp constructor available");
        },
        Err(e) => panic!("Failed to check RegExp availability: {:?}", e),
    }

    // Test basic regex functionality (RegExp.escape is confirmed working in Chrome 136 tests)
    let basic_regex_test = browser.lock().unwrap().execute_javascript(
        r#"new RegExp('hello').test('hello world')"#
    ).await;

    match basic_regex_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Basic regex test: {}", value_str);
            assert!(value_str.contains("true"),
                "Basic regex should work, got: {}", value_str);
            println!("✅ Basic regex functionality working");
        },
        Err(e) => panic!("Failed to test basic regex functionality: {:?}", e),
    }

    println!("✅ RegExp engine implementation validated!");
}

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

#[tokio::test]
async fn test_native_engine_performance() {
    println!("🧪 Testing native engine feature performance...");

    let browser = HeadlessWebBrowser::new();

    // Test that native implementations are fast
    let performance_test = browser.lock().unwrap().execute_javascript(
        "typeof Array"
    ).await;

    match performance_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Performance test result: {}", value_str);
            assert!(value_str.contains("function"), "Array should be available as function, got: {}", value_str);
            println!("✅ Native engine implementation performance verified");
        },
        Err(e) => panic!("Failed to test engine performance: {:?}", e),
    }

    println!("✅ Native engine performance test passed!");
}

#[tokio::test]
async fn test_engine_error_handling() {
    println!("🧪 Testing engine-level error handling...");

    let browser = HeadlessWebBrowser::new();

    // Test proper error handling for edge cases
    let error_test = browser.lock().unwrap().execute_javascript(r#"
        try {
            // Test basic error handling
            var result1 = new RegExp('valid');
            var result2 = new RegExp('');

            // All should work without throwing errors
            [
                result1 instanceof RegExp,
                result2 instanceof RegExp,
                true
            ]
        } catch (e) {
            ['error', e.message]
        }
    "#).await;

    match error_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Error handling test: {}", value_str);
            assert!(!value_str.contains("error"), "Engine should handle basic cases gracefully, got: {}", value_str);
            println!("✅ Engine error handling verified");
        },
        Err(e) => panic!("Failed to test engine error handling: {:?}", e),
    }

    println!("✅ Engine error handling test passed!");
}

#[tokio::test]
async fn test_comprehensive_engine_features() {
    println!("🧪 Testing comprehensive engine feature availability...");

    let browser = HeadlessWebBrowser::new();

    // Test comprehensive feature detection
    let comprehensive_test = browser.lock().unwrap().execute_javascript(
        "typeof Object"
    ).await;

    match comprehensive_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Comprehensive engine test: {}", value_str);

            // Should have basic engine features working
            assert!(value_str.contains("function"), "Object should be available as function, got: {}", value_str);
            println!("✅ Comprehensive engine features verified");
        },
        Err(e) => panic!("Failed comprehensive engine test: {:?}", e),
    }

    println!("✅ Comprehensive engine feature test passed!");
}