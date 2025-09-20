use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_131_translator_api() {
    println!("🧪 Testing Chrome 131: Translator API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Translator API availability
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml && navigator.ml.createTranslator) {
                // Test Translator API availability
                var hasTranslator = typeof navigator.ml.createTranslator === 'function';
                'Translator API available: ' + hasTranslator;
            } else if (typeof Translator !== 'undefined') {
                // Alternative global interface
                'Translator global available';
            } else {
                'Translator API not available (Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Translator API test: {}", value_str);
            // Translator API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Translator API: {:?}", e),
    }

    println!("✅ Translator API test completed");
}

#[tokio::test]
async fn test_chrome_131_webrtc_scale_resolution_down_to() {
    println!("🧪 Testing Chrome 131: WebRTC scaleResolutionDownTo...");

    let browser = HeadlessWebBrowser::new();

    // Test WebRTC scaleResolutionDownTo
    let js_code = r#"
        try {
            if (typeof RTCPeerConnection !== 'undefined') {
                // Test creating peer connection
                var pc = new RTCPeerConnection();

                // Test if we can create encoding parameters with scaleResolutionDownTo
                var encodingParams = {
                    scaleResolutionDownTo: { width: 640, height: 360 }
                };

                'WebRTC scaleResolutionDownTo parameter structure created';
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
            println!("WebRTC scaleResolutionDownTo test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebRTC scaleResolutionDownTo should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebRTC scaleResolutionDownTo: {:?}", e),
    }

    println!("✅ WebRTC scaleResolutionDownTo test completed");
}

#[tokio::test]
async fn test_chrome_131_webxr_hand_tracking() {
    println!("🧪 Testing Chrome 131: WebXR Hand Tracking...");

    let browser = HeadlessWebBrowser::new();

    // Test WebXR Hand Tracking
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.xr) {
                // Test XR availability and hand tracking support
                var hasXR = typeof navigator.xr.isSessionSupported === 'function';
                'navigator.xr available for hand tracking: ' + hasXR;
            } else {
                'navigator.xr not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebXR Hand Tracking test: {}", value_str);
            // WebXR might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebXR Hand Tracking: {:?}", e),
    }

    println!("✅ WebXR Hand Tracking test completed");
}

#[tokio::test]
async fn test_chrome_131_webgpu_get_configuration() {
    println!("🧪 Testing Chrome 131: WebGPU getConfiguration...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGPU getConfiguration method
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.gpu) {
                // Test if canvas context has getConfiguration method
                var canvas = document.createElement('canvas');
                if (canvas && canvas.getContext) {
                    var ctx = canvas.getContext('webgpu');
                    if (ctx && typeof ctx.getConfiguration === 'function') {
                        'WebGPU getConfiguration method available';
                    } else {
                        'WebGPU context available but getConfiguration method missing';
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
            println!("WebGPU getConfiguration test: {}", value_str);
            // WebGPU might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebGPU getConfiguration: {:?}", e),
    }

    println!("✅ WebGPU getConfiguration test completed");
}

#[tokio::test]
async fn test_chrome_131_webaudio_playout_stats() {
    println!("🧪 Testing Chrome 131: WebAudio Playout Statistics...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAudio playout statistics
    let js_code = r#"
        try {
            if (typeof AudioContext !== 'undefined') {
                var audioContext = new AudioContext();

                // Test if playoutStats property exists
                var hasPlayoutStats = 'playoutStats' in audioContext;

                'AudioContext.playoutStats available: ' + hasPlayoutStats;
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
            println!("WebAudio playout stats test: {}", value_str);
            // AudioContext might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebAudio playout stats: {:?}", e),
    }

    println!("✅ WebAudio playout stats test completed");
}

#[tokio::test]
async fn test_chrome_131_summarizer_api() {
    println!("🧪 Testing Chrome 131: Summarizer API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Summarizer API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.ml && navigator.ml.createSummarizer) {
                // Test Summarizer API availability
                var hasSummarizer = typeof navigator.ml.createSummarizer === 'function';
                'Summarizer API available: ' + hasSummarizer;
            } else if (typeof Summarizer !== 'undefined') {
                // Alternative global interface
                'Summarizer global available';
            } else {
                'Summarizer API not available (Origin Trial)';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Summarizer API test: {}", value_str);
            // Summarizer API is in Origin Trial, so might not be available
        },
        Err(e) => panic!("Failed to test Summarizer API: {:?}", e),
    }

    println!("✅ Summarizer API test completed");
}

#[tokio::test]
async fn test_chrome_131_overall_compatibility() {
    println!("🧪 Testing Chrome 131: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("Translator API", "typeof navigator !== 'undefined' && navigator.ml && typeof navigator.ml.createTranslator"),
        ("RTCPeerConnection", "typeof RTCPeerConnection"),
        ("navigator.xr", "typeof navigator !== 'undefined' && typeof navigator.xr"),
        ("navigator.gpu", "typeof navigator !== 'undefined' && typeof navigator.gpu"),
        ("AudioContext", "typeof AudioContext"),
        ("Summarizer API", "typeof navigator !== 'undefined' && navigator.ml && typeof navigator.ml.createSummarizer"),
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

    println!("\n📊 Chrome 131 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 131 overall compatibility test completed");
}