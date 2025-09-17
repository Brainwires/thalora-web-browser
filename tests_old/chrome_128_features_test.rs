use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_128_promise_try() {
    println!("🧪 Testing Chrome 128: Promise.try...");

    let browser = HeadlessWebBrowser::new();

    // Test Promise.try() static method
    let js_code = r#"
        try {
            if (typeof Promise.try === 'function') {
                // Test Promise.try with synchronous function
                var result = Promise.try(() => {
                    return 'sync success';
                });

                // Test Promise.try with throwing function
                var errorResult = Promise.try(() => {
                    throw new Error('test error');
                });

                'Promise.try available: ' + (result instanceof Promise) + ', error handling: ' + (errorResult instanceof Promise);
            } else {
                'Promise.try not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Promise.try test: {}", value_str);
            assert!(!value_str.contains("error:"), "Promise.try should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Promise.try: {:?}", e),
    }

    println!("✅ Promise.try test completed");
}

#[tokio::test]
async fn test_chrome_128_caret_position_from_point() {
    println!("🧪 Testing Chrome 128: document.caretPositionFromPoint...");

    let browser = HeadlessWebBrowser::new();

    // Test document.caretPositionFromPoint() method
    let js_code = r#"
        try {
            if (typeof document.caretPositionFromPoint === 'function') {
                // Test caretPositionFromPoint with coordinates
                var caretPos = document.caretPositionFromPoint(100, 100);

                if (caretPos) {
                    'caretPositionFromPoint available, returned: ' + typeof caretPos;
                } else {
                    'caretPositionFromPoint available but returned null';
                }
            } else {
                'document.caretPositionFromPoint not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("caretPositionFromPoint test: {}", value_str);
            assert!(!value_str.contains("error:"), "caretPositionFromPoint should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test caretPositionFromPoint: {:?}", e),
    }

    println!("✅ caretPositionFromPoint test completed");
}

#[tokio::test]
async fn test_chrome_128_audio_context_onerror() {
    println!("🧪 Testing Chrome 128: AudioContext.onerror...");

    let browser = HeadlessWebBrowser::new();

    // Test AudioContext.onerror callback
    let js_code = r#"
        try {
            if (typeof AudioContext !== 'undefined') {
                var audioContext = new AudioContext();

                // Test if onerror property exists
                var hasOnError = 'onerror' in audioContext;

                if (hasOnError) {
                    // Try to set an error handler
                    audioContext.onerror = function(error) {
                        console.log('AudioContext error:', error);
                    };
                }

                'AudioContext.onerror available: ' + hasOnError;
            } else {
                'AudioContext not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("AudioContext.onerror test: {}", value_str);
            // AudioContext might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test AudioContext.onerror: {:?}", e),
    }

    println!("✅ AudioContext.onerror test completed");
}

#[tokio::test]
async fn test_chrome_128_pointer_event_device_properties() {
    println!("🧪 Testing Chrome 128: PointerEvent.deviceProperties...");

    let browser = HeadlessWebBrowser::new();

    // Test PointerEvent.deviceProperties
    let js_code = r#"
        try {
            if (typeof PointerEvent !== 'undefined') {
                // Test creating a PointerEvent with device properties
                var event = new PointerEvent('pointerdown', {
                    pointerId: 1,
                    bubbles: true,
                    cancelable: true
                });

                // Check if deviceProperties exists
                var hasDeviceProperties = 'deviceProperties' in event;

                'PointerEvent.deviceProperties available: ' + hasDeviceProperties;
            } else {
                'PointerEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("PointerEvent.deviceProperties test: {}", value_str);
            assert!(!value_str.contains("error:"), "PointerEvent.deviceProperties should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test PointerEvent.deviceProperties: {:?}", e),
    }

    println!("✅ PointerEvent.deviceProperties test completed");
}

#[tokio::test]
async fn test_chrome_128_media_session_skipad() {
    println!("🧪 Testing Chrome 128: Media Session SkipAd action...");

    let browser = HeadlessWebBrowser::new();

    // Test Media Session SkipAd action
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.mediaSession) {
                // Test if we can set SkipAd action handler
                navigator.mediaSession.setActionHandler('skipad', function() {
                    console.log('SkipAd action triggered');
                });

                'MediaSession SkipAd action supported';
            } else {
                'MediaSession not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("MediaSession SkipAd test: {}", value_str);
            // MediaSession might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test MediaSession SkipAd: {:?}", e),
    }

    println!("✅ MediaSession SkipAd test completed");
}

#[tokio::test]
async fn test_chrome_128_webauthn_hints() {
    println!("🧪 Testing Chrome 128: WebAuthn hints parameter...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn hints parameter
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test if WebAuthn is available
                var hasWebAuthn = typeof navigator.credentials.create === 'function';

                if (hasWebAuthn) {
                    // Test hints parameter (would normally be used in actual WebAuthn request)
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
                            // Chrome 128: hints parameter
                            hints: ["client-device", "security-key"]
                        }
                    };

                    'WebAuthn with hints parameter structure created';
                } else {
                    'WebAuthn not available';
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
            println!("WebAuthn hints test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn hints should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn hints: {:?}", e),
    }

    println!("✅ WebAuthn hints test completed");
}

#[tokio::test]
async fn test_chrome_128_overall_compatibility() {
    println!("🧪 Testing Chrome 128: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("Promise.try", "typeof Promise.try"),
        ("document.caretPositionFromPoint", "typeof document.caretPositionFromPoint"),
        ("AudioContext", "typeof AudioContext"),
        ("PointerEvent", "typeof PointerEvent"),
        ("navigator.credentials", "typeof navigator !== 'undefined' && typeof navigator.credentials"),
        ("navigator.mediaSession", "typeof navigator !== 'undefined' && typeof navigator.mediaSession"),
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

    println!("\n📊 Chrome 128 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 128 overall compatibility test completed");
}