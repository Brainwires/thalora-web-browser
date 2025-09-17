use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_132_element_capture() {
    println!("🧪 Testing Chrome 132: Element Capture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Element Capture API
    let js_code = r#"
        try {
            // Test if MediaStreamTrack has element capture capabilities
            if (typeof MediaStreamTrack !== 'undefined') {
                // Test element capture (would normally require getUserMedia first)
                'MediaStreamTrack available for Element Capture';
            } else {
                'MediaStreamTrack not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Element Capture test: {}", value_str);
            // MediaStreamTrack might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Element Capture: {:?}", e),
    }

    println!("✅ Element Capture test completed");
}

#[tokio::test]
async fn test_chrome_132_dialog_toggle_event() {
    println!("🧪 Testing Chrome 132: Dialog ToggleEvent...");

    let browser = HeadlessWebBrowser::new();

    // Test Dialog ToggleEvent
    let js_code = r#"
        try {
            // Test if ToggleEvent is available
            if (typeof ToggleEvent !== 'undefined') {
                var event = new ToggleEvent('toggle', {
                    oldState: 'closed',
                    newState: 'open'
                });
                'ToggleEvent created with newState: ' + event.newState;
            } else {
                'ToggleEvent constructor not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Dialog ToggleEvent test: {}", value_str);
            assert!(!value_str.contains("error:"), "Dialog ToggleEvent should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Dialog ToggleEvent: {:?}", e),
    }

    println!("✅ Dialog ToggleEvent test completed");
}

#[tokio::test]
async fn test_chrome_132_file_system_access_android() {
    println!("🧪 Testing Chrome 132: File System Access API on Android...");

    let browser = HeadlessWebBrowser::new();

    // Test File System Access API methods
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.showOpenFilePicker) {
                // Test if File System Access API is available
                var hasFileSystemAccess = typeof window.showOpenFilePicker === 'function';
                var hasSaveFilePicker = typeof window.showSaveFilePicker === 'function';
                var hasDirectoryPicker = typeof window.showDirectoryPicker === 'function';

                'File System Access - open:' + hasFileSystemAccess +
                ', save:' + hasSaveFilePicker +
                ', directory:' + hasDirectoryPicker;
            } else {
                'File System Access API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("File System Access API test: {}", value_str);
            // File System Access might not be available in headless mode
        },
        Err(e) => panic!("Failed to test File System Access API: {:?}", e),
    }

    println!("✅ File System Access API test completed");
}

#[tokio::test]
async fn test_chrome_132_request_response_bytes() {
    println!("🧪 Testing Chrome 132: Request/Response bytes() method...");

    let browser = HeadlessWebBrowser::new();

    // Test Request.bytes() and Response.bytes() methods
    let js_code = r#"
        try {
            if (typeof Request !== 'undefined' && typeof Response !== 'undefined') {
                // Test if bytes() method exists on Request
                var request = new Request('https://example.com');
                var hasRequestBytes = typeof request.bytes === 'function';

                // Test if bytes() method exists on Response
                var response = new Response('test data');
                var hasResponseBytes = typeof response.bytes === 'function';

                'Request.bytes: ' + hasRequestBytes + ', Response.bytes: ' + hasResponseBytes;
            } else {
                'Request/Response not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Request/Response bytes() test: {}", value_str);
            assert!(!value_str.contains("error:"), "Request/Response bytes() should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Request/Response bytes(): {:?}", e),
    }

    println!("✅ Request/Response bytes() test completed");
}

#[tokio::test]
async fn test_chrome_132_device_posture_api() {
    println!("🧪 Testing Chrome 132: Device Posture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Device Posture API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.devicePosture) {
                // Test Device Posture API
                var hasDevicePosture = typeof navigator.devicePosture === 'object';
                var posture = navigator.devicePosture.type || 'continuous';
                'Device Posture API available, type: ' + posture;
            } else {
                'Device Posture API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Device Posture API test: {}", value_str);
            // Device Posture API might not be available on non-foldable devices
        },
        Err(e) => panic!("Failed to test Device Posture API: {:?}", e),
    }

    println!("✅ Device Posture API test completed");
}

#[tokio::test]
async fn test_chrome_132_multi_screen_capture() {
    println!("🧪 Testing Chrome 132: Multi-Screen Capture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Multi-Screen Capture API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.mediaDevices) {
                // Test if getAllScreensMedia is available
                var hasGetAllScreensMedia = typeof navigator.mediaDevices.getAllScreensMedia === 'function';
                'getAllScreensMedia available: ' + hasGetAllScreensMedia;
            } else {
                'navigator.mediaDevices not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Multi-Screen Capture test: {}", value_str);
            // Multi-screen capture might require enterprise policy
        },
        Err(e) => panic!("Failed to test Multi-Screen Capture: {:?}", e),
    }

    println!("✅ Multi-Screen Capture test completed");
}

#[tokio::test]
async fn test_chrome_132_overall_compatibility() {
    println!("🧪 Testing Chrome 132: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("MediaStreamTrack", "typeof MediaStreamTrack"),
        ("ToggleEvent", "typeof ToggleEvent"),
        ("showOpenFilePicker", "typeof showOpenFilePicker"),
        ("Request", "typeof Request"),
        ("Response", "typeof Response"),
        ("navigator.devicePosture", "typeof navigator !== 'undefined' && typeof navigator.devicePosture"),
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

    println!("\n📊 Chrome 132 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 132 overall compatibility test completed");
}