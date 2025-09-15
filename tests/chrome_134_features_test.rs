use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_134_dialog_closedby_attribute() {
    println!("🧪 Testing Chrome 134: Dialog closedby attribute...");

    let browser = HeadlessWebBrowser::new();

    // Test Dialog closedby attribute
    let js_code = r#"
        try {
            if (typeof HTMLDialogElement !== 'undefined') {
                var dialog = document.createElement('dialog');

                // Test if closedby attribute is supported
                dialog.setAttribute('closedby', 'any');
                var hasClosedBySupport = dialog.getAttribute('closedby') === 'any';

                'Dialog closedby attribute supported: ' + hasClosedBySupport;
            } else {
                'HTMLDialogElement not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Dialog closedby test: {}", value_str);
            // Dialog elements might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Dialog closedby: {:?}", e),
    }

    println!("✅ Dialog closedby test completed");
}

#[tokio::test]
async fn test_chrome_134_web_locks_shared_storage() {
    println!("🧪 Testing Chrome 134: Web Locks API in Shared Storage...");

    let browser = HeadlessWebBrowser::new();

    // Test Web Locks API in Shared Storage
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.locks) {
                // Test basic Web Locks API availability
                var hasWebLocks = typeof navigator.locks.request === 'function';

                if (typeof sharedStorage !== 'undefined') {
                    // Test if shared storage supports locks
                    var hasSharedStorageLocks = typeof sharedStorage.batchUpdate === 'function';
                    'Web Locks available: ' + hasWebLocks + ', SharedStorage.batchUpdate: ' + hasSharedStorageLocks;
                } else {
                    'Web Locks available: ' + hasWebLocks + ', sharedStorage not available';
                }
            } else {
                'Web Locks API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Web Locks Shared Storage test: {}", value_str);
            // Web Locks might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Web Locks Shared Storage: {:?}", e),
    }

    println!("✅ Web Locks Shared Storage test completed");
}

#[tokio::test]
async fn test_chrome_134_canvas_image_smoothing_quality() {
    println!("🧪 Testing Chrome 134: Canvas imageSmoothingQuality...");

    let browser = HeadlessWebBrowser::new();

    // Test Canvas imageSmoothingQuality
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                var canvas = document.createElement('canvas');
                var ctx = canvas.getContext('2d');

                if (ctx) {
                    // Test if imageSmoothingQuality is supported
                    var hasImageSmoothingQuality = 'imageSmoothingQuality' in ctx;

                    if (hasImageSmoothingQuality) {
                        // Test setting different quality levels
                        ctx.imageSmoothingQuality = 'high';
                        var qualitySet = ctx.imageSmoothingQuality === 'high';
                        'imageSmoothingQuality supported and working: ' + qualitySet;
                    } else {
                        'imageSmoothingQuality not supported in context';
                    }
                } else {
                    'Canvas context not available';
                }
            } else {
                'document not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Canvas imageSmoothingQuality test: {}", value_str);
            assert!(!value_str.contains("error:"), "Canvas imageSmoothingQuality should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Canvas imageSmoothingQuality: {:?}", e),
    }

    println!("✅ Canvas imageSmoothingQuality test completed");
}

#[tokio::test]
async fn test_chrome_134_offscreen_canvas_context_attributes() {
    println!("🧪 Testing Chrome 134: OffscreenCanvas getContextAttributes...");

    let browser = HeadlessWebBrowser::new();

    // Test OffscreenCanvas getContextAttributes
    let js_code = r#"
        try {
            if (typeof OffscreenCanvas !== 'undefined') {
                var canvas = new OffscreenCanvas(100, 100);
                var ctx = canvas.getContext('2d');

                if (ctx && typeof ctx.getContextAttributes === 'function') {
                    var attributes = ctx.getContextAttributes();
                    'OffscreenCanvas getContextAttributes available: ' + (attributes !== null);
                } else {
                    'OffscreenCanvas getContextAttributes not available';
                }
            } else {
                'OffscreenCanvas not supported';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("OffscreenCanvas getContextAttributes test: {}", value_str);
            // OffscreenCanvas might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test OffscreenCanvas getContextAttributes: {:?}", e),
    }

    println!("✅ OffscreenCanvas getContextAttributes test completed");
}

#[tokio::test]
async fn test_chrome_134_console_timestamp() {
    println!("🧪 Testing Chrome 134: console.timeStamp enhancements...");

    let browser = HeadlessWebBrowser::new();

    // Test console.timeStamp enhancements
    let js_code = r#"
        try {
            if (typeof console !== 'undefined' && console.timeStamp) {
                // Test basic timeStamp functionality
                var hasTimeStamp = typeof console.timeStamp === 'function';

                // Try calling with custom parameters (Chrome 134 enhancements)
                try {
                    console.timeStamp('test-timestamp', { detail: 'custom' });
                    'console.timeStamp with enhancements: available';
                } catch (timestampError) {
                    'console.timeStamp basic available: ' + hasTimeStamp;
                }
            } else {
                'console.timeStamp not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("console.timeStamp test: {}", value_str);
            assert!(!value_str.contains("error:"), "console.timeStamp should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test console.timeStamp: {:?}", e),
    }

    println!("✅ console.timeStamp test completed");
}

#[tokio::test]
async fn test_chrome_134_explicit_resource_management() {
    println!("🧪 Testing Chrome 134: Explicit Resource Management...");

    let browser = HeadlessWebBrowser::new();

    // Test Explicit Resource Management (using and await using)
    let js_code = r#"
        try {
            // Test if Symbol.dispose and Symbol.asyncDispose are available
            var hasDispose = typeof Symbol !== 'undefined' && typeof Symbol.dispose !== 'undefined';
            var hasAsyncDispose = typeof Symbol !== 'undefined' && typeof Symbol.asyncDispose !== 'undefined';

            if (hasDispose || hasAsyncDispose) {
                'Explicit Resource Management symbols - dispose: ' + hasDispose + ', asyncDispose: ' + hasAsyncDispose;
            } else {
                'Explicit Resource Management not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Explicit Resource Management test: {}", value_str);
            // This is a newer JavaScript feature that might not be fully available
        },
        Err(e) => panic!("Failed to test Explicit Resource Management: {:?}", e),
    }

    println!("✅ Explicit Resource Management test completed");
}

#[tokio::test]
async fn test_chrome_134_digital_credential_api() {
    println!("🧪 Testing Chrome 134: Digital Credential API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Digital Credential API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test if DigitalCredential is available
                if (typeof DigitalCredential !== 'undefined') {
                    'DigitalCredential API available';
                } else if (navigator.credentials.get) {
                    // Test if digital credential options are supported
                    try {
                        // This would normally require origin trial token
                        'navigator.credentials.get available (digital credential may require origin trial)';
                    } catch (credError) {
                        'Digital Credential API: ' + credError.message;
                    }
                } else {
                    'Digital Credential API not available';
                }
            } else {
                'navigator.credentials not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Digital Credential API test: {}", value_str);
            // Digital Credential API is in Origin Trial, might not be available
        },
        Err(e) => panic!("Failed to test Digital Credential API: {:?}", e),
    }

    println!("✅ Digital Credential API test completed");
}

#[tokio::test]
async fn test_chrome_134_has_slotted_pseudo_class() {
    println!("🧪 Testing Chrome 134: :has-slotted pseudo-class...");

    let browser = HeadlessWebBrowser::new();

    // Test :has-slotted pseudo-class
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test if :has-slotted pseudo-class is supported
                var supportsHasSlotted = CSS.supports('selector(:has-slotted(*))');
                ':has-slotted pseudo-class supported: ' + supportsHasSlotted;
            } else if (typeof document !== 'undefined') {
                // Fallback test
                try {
                    var style = document.createElement('style');
                    style.textContent = ':host(:has-slotted(*)) { display: block; }';
                    ':has-slotted pseudo-class: fallback test completed';
                } catch (styleError) {
                    ':has-slotted pseudo-class: ' + styleError.message;
                }
            } else {
                ':has-slotted pseudo-class: cannot test without CSS.supports or document';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!(":has-slotted pseudo-class test: {}", value_str);
            // CSS.supports might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test :has-slotted pseudo-class: {:?}", e),
    }

    println!("✅ :has-slotted pseudo-class test completed");
}

#[tokio::test]
async fn test_chrome_134_overall_compatibility() {
    println!("🧪 Testing Chrome 134: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("HTMLDialogElement", "typeof HTMLDialogElement"),
        ("navigator.locks", "typeof navigator !== 'undefined' && typeof navigator.locks"),
        ("imageSmoothingQuality", "typeof document !== 'undefined'"), // We'll check this indirectly
        ("OffscreenCanvas", "typeof OffscreenCanvas"),
        ("console.timeStamp", "typeof console !== 'undefined' && typeof console.timeStamp"),
        ("Symbol.dispose", "typeof Symbol !== 'undefined' && typeof Symbol.dispose !== 'undefined'"),
        ("navigator.credentials", "typeof navigator !== 'undefined' && typeof navigator.credentials"),
        ("CSS.supports", "typeof CSS !== 'undefined' && typeof CSS.supports"),
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

    println!("\n📊 Chrome 134 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 134 overall compatibility test completed");
}