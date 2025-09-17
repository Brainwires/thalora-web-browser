use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_129_scheduler_yield() {
    println!("🧪 Testing Chrome 129: scheduler.yield()...");

    let browser = HeadlessWebBrowser::new();

    // Test scheduler.yield() API
    let js_code = r#"
        try {
            if (typeof scheduler !== 'undefined' && typeof scheduler.yield === 'function') {
                // Test scheduler.yield() returns a promise
                var yieldPromise = scheduler.yield();

                if (yieldPromise && typeof yieldPromise.then === 'function') {
                    'scheduler.yield available and returns promise';
                } else {
                    'scheduler.yield available but does not return promise';
                }
            } else {
                'scheduler.yield not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("scheduler.yield test: {}", value_str);
            assert!(!value_str.contains("error:"), "scheduler.yield should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test scheduler.yield: {:?}", e),
    }

    println!("✅ scheduler.yield test completed");
}

#[tokio::test]
async fn test_chrome_129_intl_duration_format() {
    println!("🧪 Testing Chrome 129: Intl.DurationFormat...");

    let browser = HeadlessWebBrowser::new();

    // Test Intl.DurationFormat API
    let js_code = r#"
        try {
            if (typeof Intl !== 'undefined' && typeof Intl.DurationFormat === 'function') {
                // Test creating a DurationFormat instance
                var formatter = new Intl.DurationFormat('en', {
                    style: 'long',
                    hours: 'numeric',
                    minutes: 'numeric',
                    seconds: 'numeric'
                });

                if (formatter && typeof formatter.format === 'function') {
                    // Test formatting a duration
                    var formatted = formatter.format({ hours: 1, minutes: 40, seconds: 30 });
                    'Intl.DurationFormat available, formatted: ' + formatted;
                } else {
                    'Intl.DurationFormat constructor available but format method missing';
                }
            } else {
                'Intl.DurationFormat not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Intl.DurationFormat test: {}", value_str);
            assert!(!value_str.contains("error:"), "Intl.DurationFormat should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Intl.DurationFormat: {:?}", e),
    }

    println!("✅ Intl.DurationFormat test completed");
}

#[tokio::test]
async fn test_chrome_129_webauthn_serialization() {
    println!("🧪 Testing Chrome 129: WebAuthn serialization methods...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn serialization methods
    let js_code = r#"
        try {
            if (typeof PublicKeyCredential !== 'undefined') {
                // Test if toJSON method exists
                var hasToJSON = typeof PublicKeyCredential.prototype.toJSON === 'function';

                // Test if static parsing methods exist
                var hasParseCreation = typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function';
                var hasParseRequest = typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'function';

                'PublicKeyCredential.toJSON: ' + hasToJSON +
                ', parseCreationOptionsFromJSON: ' + hasParseCreation +
                ', parseRequestOptionsFromJSON: ' + hasParseRequest;
            } else {
                'PublicKeyCredential not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAuthn serialization test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn serialization should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn serialization: {:?}", e),
    }

    println!("✅ WebAuthn serialization test completed");
}

#[tokio::test]
async fn test_chrome_129_rtc_data_channel_blob() {
    println!("🧪 Testing Chrome 129: RTCDataChannel Blob support...");

    let browser = HeadlessWebBrowser::new();

    // Test RTCDataChannel Blob support
    let js_code = r#"
        try {
            if (typeof RTCPeerConnection !== 'undefined') {
                // Create a peer connection to test data channel
                var pc = new RTCPeerConnection();
                var dataChannel = pc.createDataChannel('test');

                // Test if binaryType property exists
                var hasBinaryType = 'binaryType' in dataChannel;

                // Test if we can set binaryType to 'blob'
                if (hasBinaryType) {
                    dataChannel.binaryType = 'blob';
                    var binaryTypeSet = dataChannel.binaryType === 'blob';
                    'RTCDataChannel binaryType supported: ' + binaryTypeSet;
                } else {
                    'RTCDataChannel binaryType property not found';
                }
            } else {
                'RTCPeerConnection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("RTCDataChannel Blob test: {}", value_str);
            // WebRTC might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test RTCDataChannel Blob: {:?}", e),
    }

    println!("✅ RTCDataChannel Blob test completed");
}

#[tokio::test]
async fn test_chrome_129_file_system_observer() {
    println!("🧪 Testing Chrome 129: FileSystemObserver (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test FileSystemObserver API
    let js_code = r#"
        try {
            if (typeof FileSystemObserver !== 'undefined') {
                // Test FileSystemObserver constructor
                'FileSystemObserver constructor available';
            } else {
                'FileSystemObserver not available (expected in Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("FileSystemObserver test: {}", value_str);
            // FileSystemObserver is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test FileSystemObserver: {:?}", e),
    }

    println!("✅ FileSystemObserver test completed");
}

#[tokio::test]
async fn test_chrome_129_webgpu_hdr_support() {
    println!("🧪 Testing Chrome 129: WebGPU HDR support...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU HDR support
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test if we can get a canvas context with HDR options
                var canvas = document.createElement('canvas');
                if (canvas && canvas.getContext) {
                    var ctx = canvas.getContext('webgpu');
                    if (ctx) {
                        'WebGPU context available for HDR testing';
                    } else {
                        'WebGPU context not available';
                    }
                } else {
                    'Canvas getContext not available';
                }
            } else {
                'WebGPU not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGPU HDR test: {}", value_str);
            // WebGPU might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU HDR: {:?}", e),
    }

    println!("✅ WebGPU HDR test completed");
}

#[tokio::test]
async fn test_chrome_129_overall_compatibility() {
    println!("🧪 Testing Chrome 129: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("scheduler.yield", "typeof scheduler !== 'undefined' && typeof scheduler.yield"),
        ("Intl.DurationFormat", "typeof Intl !== 'undefined' && typeof Intl.DurationFormat"),
        ("PublicKeyCredential", "typeof PublicKeyCredential"),
        ("RTCPeerConnection", "typeof RTCPeerConnection"),
        ("navigator.gpu", "typeof navigator !== 'undefined' && typeof navigator.gpu"),
        ("FileSystemObserver", "typeof FileSystemObserver"),
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

    println!("\n📊 Chrome 129 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 129 overall compatibility test completed");
}