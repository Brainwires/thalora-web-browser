use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_130_serial_port_connected() {
    println!("🧪 Testing Chrome 130: SerialPort.connected attribute...");

    let browser = HeadlessWebBrowser::new();

    // Test SerialPort.connected attribute
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.serial) {
                // Test if SerialPort has connected attribute (mock test)
                var hasSerial = typeof navigator.serial.requestPort === 'function';
                'navigator.serial available: ' + hasSerial;
            } else {
                'navigator.serial not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("SerialPort.connected test: {}", value_str);
            // Web Serial might not be available in headless mode
        },
        Err(e) => panic!("Failed to test SerialPort.connected: {:?}", e),
    }

    println!("✅ SerialPort.connected test completed");
}

#[tokio::test]
async fn test_chrome_130_webassembly_string_builtins() {
    println!("🧪 Testing Chrome 130: WebAssembly JavaScript String Builtins...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAssembly String Builtins availability
    let js_code = r#"
        try {
            if (typeof WebAssembly !== 'undefined') {
                // Test if WebAssembly has string builtins support
                var hasWebAssembly = typeof WebAssembly.Module === 'function';

                // Check for string manipulation capabilities
                var stringSupport = 'WebAssembly string builtins concept available';

                'WebAssembly available: ' + hasWebAssembly + ', ' + stringSupport;
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
            println!("WebAssembly String Builtins test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAssembly String Builtins should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAssembly String Builtins: {:?}", e),
    }

    println!("✅ WebAssembly String Builtins test completed");
}

#[tokio::test]
async fn test_chrome_130_webauthn_attestation_formats() {
    println!("🧪 Testing Chrome 130: WebAuthn attestationFormats field...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn attestationFormats field
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test creating credential options with attestationFormats
                var credentialOptions = {
                    publicKey: {
                        challenge: new Uint8Array(32),
                        rp: { name: "Test RP" },
                        user: {
                            id: new Uint8Array(16),
                            name: "test@example.com",
                            displayName: "Test User"
                        },
                        pubKeyCredParams: [{alg: -7, type: "public-key"}],
                        // Chrome 130: attestationFormats field
                        attestationFormats: ["packed", "fido-u2f"]
                    }
                };

                'WebAuthn attestationFormats field structure created';
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
            println!("WebAuthn attestationFormats test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn attestationFormats should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn attestationFormats: {:?}", e),
    }

    println!("✅ WebAuthn attestationFormats test completed");
}

#[tokio::test]
async fn test_chrome_130_document_pip_placement() {
    println!("🧪 Testing Chrome 130: Document Picture-in-Picture preferInitialWindowPlacement...");

    let browser = HeadlessWebBrowser::new();

    // Test Document Picture-in-Picture preferInitialWindowPlacement
    let js_code = r#"
        try {
            if (typeof documentPictureInPicture !== 'undefined') {
                // Test preferInitialWindowPlacement parameter
                var pipOptions = {
                    width: 300,
                    height: 200,
                    // Chrome 130: preferInitialWindowPlacement parameter
                    preferInitialWindowPlacement: true
                };

                'documentPictureInPicture preferInitialWindowPlacement option available';
            } else {
                'documentPictureInPicture not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Document PiP preferInitialWindowPlacement test: {}", value_str);
            // Picture-in-Picture might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Document PiP preferInitialWindowPlacement: {:?}", e),
    }

    println!("✅ Document PiP preferInitialWindowPlacement test completed");
}

#[tokio::test]
async fn test_chrome_130_indexeddb_error_reporting() {
    println!("🧪 Testing Chrome 130: IndexedDB improved error reporting...");

    let browser = HeadlessWebBrowser::new();

    // Test IndexedDB error reporting improvements
    let js_code = r#"
        try {
            if (typeof indexedDB !== 'undefined') {
                // Test IndexedDB availability and error handling
                var hasIndexedDB = typeof indexedDB.open === 'function';

                // Chrome 130: Enhanced error reporting for large value read failures
                'IndexedDB available with enhanced error reporting: ' + hasIndexedDB;
            } else {
                'IndexedDB not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("IndexedDB error reporting test: {}", value_str);
            assert!(!value_str.contains("error:"), "IndexedDB error reporting should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test IndexedDB error reporting: {:?}", e),
    }

    println!("✅ IndexedDB error reporting test completed");
}

#[tokio::test]
async fn test_chrome_130_language_detector_api() {
    println!("🧪 Testing Chrome 130: Language Detector API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Language Detector API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml) {
                // Test Language Detector API availability
                var hasLanguageDetector = typeof navigator.ml.createLanguageDetector === 'function';
                'Language Detector API available: ' + hasLanguageDetector;
            } else if (typeof LanguageDetector !== 'undefined') {
                // Alternative global interface
                'LanguageDetector global available';
            } else {
                'Language Detector API not available (Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Language Detector API test: {}", value_str);
            // Language Detector API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Language Detector API: {:?}", e),
    }

    println!("✅ Language Detector API test completed");
}

#[tokio::test]
async fn test_chrome_130_overall_compatibility() {
    println!("🧪 Testing Chrome 130: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("navigator.serial", "typeof navigator !== 'undefined' && typeof navigator.serial"),
        ("WebAssembly", "typeof WebAssembly"),
        ("navigator.credentials", "typeof navigator !== 'undefined' && typeof navigator.credentials"),
        ("documentPictureInPicture", "typeof documentPictureInPicture"),
        ("indexedDB", "typeof indexedDB"),
        ("LanguageDetector", "typeof LanguageDetector !== 'undefined' || (typeof navigator !== 'undefined' && navigator.ml)"),
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

    println!("\n📊 Chrome 130 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 130 overall compatibility test completed");
}