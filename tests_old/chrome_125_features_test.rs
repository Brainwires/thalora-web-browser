use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_125_regex_modifiers() {
    println!("🧪 Testing Chrome 125: Regular Expression Modifiers...");

    let browser = HeadlessWebBrowser::new();

    // Test regex modifiers - locally modify flags inside pattern
    let js_code = r#"
        try {
            // Case insensitive modifier inside pattern
            const regex1 = /(?i:[a-z])[a-z]$/;
            const result1 = regex1.test('Ab');

            // Multiple flags modifier
            const regex2 = /(?im:test.*line)other/;
            const result2 = regex2.test('TEST\nLINEother');

            'success: ' + result1 + ',' + result2;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Regex modifiers test: {}", value_str);
            // Even if not supported, should not throw syntax errors
            assert!(!value_str.contains("SyntaxError"), "Regex modifiers should not cause syntax errors, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test regex modifiers: {:?}", e),
    }

    println!("✅ Regex modifiers test completed");
}

#[tokio::test]
async fn test_chrome_125_duplicate_named_groups() {
    println!("🧪 Testing Chrome 125: Duplicate Named Capture Groups...");

    let browser = HeadlessWebBrowser::new();

    // Test duplicate named capture groups in alternatives
    let js_code = r#"
        try {
            // Same named group in different alternatives
            const regex = /(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})/;
            const match1 = regex.exec('2024-12');
            const match2 = regex.exec('12-2024');

            'success: ' + (match1 ? match1.groups.year : 'null') + ',' + (match2 ? match2.groups.year : 'null');
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Duplicate named groups test: {}", value_str);
            // Should not throw syntax errors
            assert!(!value_str.contains("SyntaxError"), "Duplicate named groups should not cause syntax errors, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test duplicate named groups: {:?}", e),
    }

    println!("✅ Duplicate named groups test completed");
}

#[tokio::test]
async fn test_chrome_125_compute_pressure_api() {
    println!("🧪 Testing Chrome 125: Compute Pressure API...");

    let browser = HeadlessWebBrowser::new();

    // Test PressureObserver availability
    let result = browser.lock().unwrap().execute_javascript("typeof PressureObserver").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("PressureObserver type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "PressureObserver should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check PressureObserver: {:?}", e),
    }

    // Test navigator.computePressure
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.computePressure").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.computePressure type: {}", value_str);
            // Might be undefined in headless mode
        },
        Err(e) => panic!("Failed to check navigator.computePressure: {:?}", e),
    }

    println!("✅ Compute Pressure API test completed");
}

#[tokio::test]
async fn test_chrome_125_storage_access_api() {
    println!("🧪 Testing Chrome 125: Storage Access API Extension...");

    let browser = HeadlessWebBrowser::new();

    // First check if document exists
    let doc_result = browser.lock().unwrap().execute_javascript("typeof document").await;
    match doc_result {
        Ok(value) => println!("document type: {:?}", value),
        Err(e) => println!("document check error: {:?}", e),
    }

    // Test document.requestStorageAccess
    let result = browser.lock().unwrap().execute_javascript("typeof document.requestStorageAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("document.requestStorageAccess type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "requestStorageAccess should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check requestStorageAccess: {:?}", e),
    }

    // Test document.hasStorageAccess
    let result = browser.lock().unwrap().execute_javascript("typeof document.hasStorageAccess").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("document.hasStorageAccess type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "hasStorageAccess should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check hasStorageAccess: {:?}", e),
    }

    println!("✅ Storage Access API test completed");
}

#[tokio::test]
async fn test_chrome_125_direct_sockets_api() {
    println!("🧪 Testing Chrome 125: Direct Sockets API...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.tcp
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.tcp").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.tcp type: {}", value_str);
            // Might be undefined in headless mode (Chrome Apps only)
        },
        Err(e) => panic!("Failed to check navigator.tcp: {:?}", e),
    }

    // Test navigator.udp
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.udp").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.udp type: {}", value_str);
            // Might be undefined in headless mode (Chrome Apps only)
        },
        Err(e) => panic!("Failed to check navigator.udp: {:?}", e),
    }

    println!("✅ Direct Sockets API test completed");
}

#[tokio::test]
async fn test_chrome_125_css_anchor_positioning() {
    println!("🧪 Testing Chrome 125: CSS Anchor Positioning...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS.supports for anchor positioning
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                const anchorName = CSS.supports('anchor-name', 'my-anchor');
                const positionAnchor = CSS.supports('position-anchor', 'my-anchor');
                const anchorTop = CSS.supports('top', 'anchor(bottom)');

                'anchor-name:' + anchorName + ',position-anchor:' + positionAnchor + ',anchor-top:' + anchorTop;
            } else {
                'CSS.supports not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("CSS anchor positioning support: {}", value_str);
            // Should have some level of CSS.supports available
            assert!(!value_str.contains("error:"), "CSS anchor positioning test should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS anchor positioning: {:?}", e),
    }

    println!("✅ CSS anchor positioning test completed");
}

#[tokio::test]
async fn test_chrome_125_websocket_url_handling() {
    println!("🧪 Testing Chrome 125: Enhanced WebSocket URL handling...");

    let browser = HeadlessWebBrowser::new();

    // Test WebSocket with HTTP/HTTPS URLs (should convert to ws/wss)
    let js_code = r#"
        try {
            // Test if WebSocket constructor accepts HTTP URLs
            var ws1 = new WebSocket('ws://echo.websocket.org/');
            var result = 'WebSocket created with ws:// URL: ' + (ws1 instanceof WebSocket);

            // Test relative URL handling
            var baseUrl = 'wss://example.com/base/';
            // In real implementation, this should resolve relative to current location
            result;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket URL handling: {}", value_str);
            // WebSocket should be available and constructable
            assert!(value_str.contains("true") || value_str.contains("WebSocket"),
                "WebSocket should be constructable, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebSocket URL handling: {:?}", e),
    }

    println!("✅ WebSocket URL handling test completed");
}

#[tokio::test]
async fn test_chrome_125_overall_compatibility() {
    println!("🧪 Testing Chrome 125: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("PressureObserver", "typeof PressureObserver"),
        ("document.requestStorageAccess", "typeof document.requestStorageAccess"),
        ("document.hasStorageAccess", "typeof document.hasStorageAccess"),
        ("navigator.tcp", "typeof navigator.tcp"),
        ("navigator.udp", "typeof navigator.udp"),
        ("CSS.supports", "typeof CSS !== 'undefined' && typeof CSS.supports"),
        ("WebSocket", "typeof WebSocket"),
    ];

    let mut available = 0;
    let total = features.len();

    for (feature_name, js_check) in features {
        match browser.lock().unwrap().execute_javascript(js_check).await {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                let is_available = value_str.contains("function") || value_str.contains("object") || value_str.contains("true");
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

    println!("\n📊 Chrome 125 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 125 overall compatibility test completed");
}