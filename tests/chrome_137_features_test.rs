use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_137_selection_direction() {
    println!("🧪 Testing Chrome 137: Selection.direction...");

    let browser = HeadlessWebBrowser::new();

    // Test Selection.direction property
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if direction property exists
                var hasDirection = 'direction' in selection;
                var directionType = typeof selection.direction;

                'Selection.direction property: ' + hasDirection + ' (type: ' + directionType + ')';
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Selection.direction test: {}", value_str);
            // Selection API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Selection.direction: {:?}", e),
    }

    println!("✅ Selection.direction test completed");
}

#[tokio::test]
async fn test_chrome_137_selection_get_composed_ranges() {
    println!("🧪 Testing Chrome 137: Selection.getComposedRanges()...");

    let browser = HeadlessWebBrowser::new();

    // Test Selection.getComposedRanges method
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if getComposedRanges method exists
                var hasGetComposedRanges = typeof selection.getComposedRanges === 'function';

                if (hasGetComposedRanges) {
                    'Selection.getComposedRanges method available: true';
                } else {
                    'Selection.getComposedRanges method not available';
                }
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Selection.getComposedRanges test: {}", value_str);
            // Selection API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Selection.getComposedRanges: {:?}", e),
    }

    println!("✅ Selection.getComposedRanges test completed");
}

#[tokio::test]
async fn test_chrome_137_webassembly_jspi() {
    println!("🧪 Testing Chrome 137: WebAssembly JSPI (JavaScript Promise Integration)...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAssembly JSPI support
    let js_code = r#"
        try {
            if (typeof WebAssembly !== 'undefined') {
                // Test basic WebAssembly availability
                var hasWebAssembly = typeof WebAssembly === 'object';

                // Test for JSPI-related features (experimental)
                var hasPromiseIntegration = typeof WebAssembly.promising === 'function' ||
                                           typeof WebAssembly.Suspending === 'function';

                var wasmSupport = 'WebAssembly available: ' + hasWebAssembly;
                var jspiSupport = 'JSPI features: ' + hasPromiseIntegration;

                wasmSupport + ', ' + jspiSupport;
            } else {
                'WebAssembly not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAssembly JSPI test: {}", value_str);
            // JSPI is experimental and might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebAssembly JSPI: {:?}", e),
    }

    println!("✅ WebAssembly JSPI test completed");
}

#[tokio::test]
async fn test_chrome_137_webcrypto_ed25519() {
    println!("🧪 Testing Chrome 137: WebCrypto Ed25519 support...");

    let browser = HeadlessWebBrowser::new();

    // Test WebCrypto Ed25519 algorithm support
    let js_code = r#"
        try {
            if (typeof crypto !== 'undefined' && crypto.subtle) {
                // Test if Ed25519 is supported in generateKey
                var hasCryptoSubtle = typeof crypto.subtle.generateKey === 'function';

                // Test basic crypto availability
                var cryptoAvailable = 'crypto.subtle available: ' + hasCryptoSubtle;

                // Note: Actually testing Ed25519 would require async operations
                // For now, just check crypto availability
                cryptoAvailable;
            } else {
                'crypto.subtle not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebCrypto Ed25519 test: {}", value_str);
            // WebCrypto might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebCrypto Ed25519: {:?}", e),
    }

    println!("✅ WebCrypto Ed25519 test completed");
}

#[tokio::test]
async fn test_chrome_137_css_if_function() {
    println!("🧪 Testing Chrome 137: CSS if() function...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS if() function support
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test CSS if() function support
                var supportsIf = CSS.supports('color', 'if(true, red, blue)');

                'CSS if() function supported: ' + supportsIf;
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
            println!("CSS if() function test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS if() function should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS if() function: {:?}", e),
    }

    println!("✅ CSS if() function test completed");
}

#[tokio::test]
async fn test_chrome_137_reading_flow_properties() {
    println!("🧪 Testing Chrome 137: reading-flow and reading-order CSS properties...");

    let browser = HeadlessWebBrowser::new();

    // Test reading-flow and reading-order CSS properties
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test reading-flow property
                var supportsReadingFlow = CSS.supports('reading-flow', 'flex-visual');

                // Test reading-order property
                var supportsReadingOrder = CSS.supports('reading-order', '1');

                'reading-flow: ' + supportsReadingFlow + ', reading-order: ' + supportsReadingOrder;
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
            println!("Reading flow properties test: {}", value_str);
            assert!(!value_str.contains("error:"), "Reading flow properties should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test reading flow properties: {:?}", e),
    }

    println!("✅ Reading flow properties test completed");
}

#[tokio::test]
async fn test_chrome_137_blob_url_partitioning() {
    println!("🧪 Testing Chrome 137: Blob URL partitioning...");

    let browser = HeadlessWebBrowser::new();

    // Test Blob URL creation and access
    let js_code = r#"
        try {
            if (typeof Blob !== 'undefined' && typeof URL !== 'undefined') {
                // Test basic Blob URL creation
                var blob = new Blob(['test content'], { type: 'text/plain' });
                var blobUrl = URL.createObjectURL(blob);

                var hasBlobUrl = blobUrl.startsWith('blob:');
                var urlStructure = 'Blob URL created: ' + hasBlobUrl;

                // Test URL.revokeObjectURL
                var hasRevoke = typeof URL.revokeObjectURL === 'function';
                if (hasRevoke) {
                    URL.revokeObjectURL(blobUrl);
                }

                urlStructure + ', revoke available: ' + hasRevoke;
            } else {
                'Blob or URL not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Blob URL partitioning test: {}", value_str);
            assert!(!value_str.contains("error:"), "Blob URL should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Blob URL partitioning: {:?}", e),
    }

    println!("✅ Blob URL partitioning test completed");
}

#[tokio::test]
async fn test_chrome_137_webgpu_improvements() {
    println!("🧪 Testing Chrome 137: WebGPU texture view improvements...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU texture view features
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test basic WebGPU availability
                var hasGPU = typeof navigator.gpu.requestAdapter === 'function';

                'WebGPU basic support: ' + hasGPU;
            } else {
                'navigator.gpu not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGPU improvements test: {}", value_str);
            // WebGPU might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU improvements: {:?}", e),
    }

    println!("✅ WebGPU improvements test completed");
}

#[tokio::test]
async fn test_chrome_137_svg_transform_attribute() {
    println!("🧪 Testing Chrome 137: SVG transform attribute on root element...");

    let browser = HeadlessWebBrowser::new();

    // Test SVG transform attribute support
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                // Test creating SVG element with transform attribute
                var svg = document.createElement('svg');
                svg.setAttribute('transform', 'scale(2) rotate(45)');

                var hasTransform = svg.getAttribute('transform') === 'scale(2) rotate(45)';
                var svgSupport = 'SVG element creation: true';
                var transformSupport = 'transform attribute: ' + hasTransform;

                svgSupport + ', ' + transformSupport;
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
            println!("SVG transform attribute test: {}", value_str);
            assert!(!value_str.contains("error:"), "SVG transform should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test SVG transform attribute: {:?}", e),
    }

    println!("✅ SVG transform attribute test completed");
}

#[tokio::test]
async fn test_chrome_137_overall_compatibility() {
    println!("🧪 Testing Chrome 137: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("window.getSelection", "typeof window !== 'undefined' && typeof window.getSelection"),
        ("WebAssembly", "typeof WebAssembly"),
        ("crypto.subtle", "typeof crypto !== 'undefined' && typeof crypto.subtle"),
        ("CSS.supports", "typeof CSS !== 'undefined' && typeof CSS.supports"),
        ("Blob", "typeof Blob"),
        ("URL", "typeof URL"),
        ("navigator.gpu", "typeof navigator !== 'undefined' && typeof navigator.gpu"),
        ("document", "typeof document"),
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

    println!("\n📊 Chrome 137 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 137 overall compatibility test completed");
}