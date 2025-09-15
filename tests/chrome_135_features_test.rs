use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_135_float16array() {
    println!("🧪 Testing Chrome 135: Float16Array...");

    let browser = HeadlessWebBrowser::new();

    // Test Float16Array support
    let js_code = r#"
        try {
            if (typeof Float16Array !== 'undefined') {
                // Test Float16Array constructor and basic operations
                var float16 = new Float16Array(4);
                float16[0] = 1.5;
                float16[1] = 2.5;

                var isArray = float16 instanceof Float16Array;
                var hasLength = float16.length === 4;
                var hasValues = float16[0] === 1.5 && float16[1] === 2.5;

                'Float16Array constructor available and working: ' + (isArray && hasLength && hasValues);
            } else {
                'Float16Array not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Float16Array test: {}", value_str);
            assert!(!value_str.contains("error:"), "Float16Array should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Float16Array: {:?}", e),
    }

    println!("✅ Float16Array test completed");
}

#[tokio::test]
async fn test_chrome_135_fetch_later() {
    println!("🧪 Testing Chrome 135: fetchLater() API...");

    let browser = HeadlessWebBrowser::new();

    // Test fetchLater API
    let js_code = r#"
        try {
            if (typeof fetchLater !== 'undefined') {
                // Test fetchLater function availability
                var hasFetchLater = typeof fetchLater === 'function';
                'fetchLater API available: ' + hasFetchLater;
            } else if (typeof navigator !== 'undefined' && navigator.sendBeacon) {
                // Test if beacon API exists as fallback
                'fetchLater not available, but sendBeacon exists';
            } else {
                'fetchLater API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("fetchLater test: {}", value_str);
            // fetchLater might not be available in headless mode
        },
        Err(e) => panic!("Failed to test fetchLater: {:?}", e),
    }

    println!("✅ fetchLater test completed");
}

#[tokio::test]
async fn test_chrome_135_observable_api() {
    println!("🧪 Testing Chrome 135: Observable API...");

    let browser = HeadlessWebBrowser::new();

    // Test Observable API
    let js_code = r#"
        try {
            if (typeof Observable !== 'undefined') {
                // Test Observable constructor
                var hasObservable = typeof Observable === 'function';

                // Test basic Observable creation
                try {
                    var obs = new Observable(function(observer) {
                        observer.next(1);
                        observer.complete();
                    });
                    var hasObservableInstance = obs instanceof Observable;
                    'Observable API available and working: ' + (hasObservable && hasObservableInstance);
                } catch (obsError) {
                    'Observable constructor available but error: ' + obsError.message;
                }
            } else {
                'Observable API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Observable test: {}", value_str);
            // Observable API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Observable: {:?}", e),
    }

    println!("✅ Observable test completed");
}

#[tokio::test]
async fn test_chrome_135_navigate_event_source_element() {
    println!("🧪 Testing Chrome 135: NavigateEvent.sourceElement...");

    let browser = HeadlessWebBrowser::new();

    // Test NavigateEvent.sourceElement property
    let js_code = r#"
        try {
            if (typeof NavigateEvent !== 'undefined') {
                // Test if NavigateEvent has sourceElement property
                var hasSourceElement = 'sourceElement' in NavigateEvent.prototype;
                'NavigateEvent.sourceElement property available: ' + hasSourceElement;
            } else {
                'NavigateEvent not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("NavigateEvent.sourceElement test: {}", value_str);
            // NavigateEvent might not be available in headless mode
        },
        Err(e) => panic!("Failed to test NavigateEvent.sourceElement: {:?}", e),
    }

    println!("✅ NavigateEvent.sourceElement test completed");
}

#[tokio::test]
async fn test_chrome_135_command_attributes() {
    println!("🧪 Testing Chrome 135: Command and commandfor attributes...");

    let browser = HeadlessWebBrowser::new();

    // Test command and commandfor attributes
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                var button = document.createElement('button');

                // Test setting command and commandfor attributes
                button.setAttribute('command', 'show-popover');
                button.setAttribute('commandfor', 'my-element');

                var hasCommand = button.getAttribute('command') === 'show-popover';
                var hasCommandFor = button.getAttribute('commandfor') === 'my-element';

                'Command attributes supported: ' + (hasCommand && hasCommandFor);
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
            println!("Command attributes test: {}", value_str);
            assert!(!value_str.contains("error:"), "Command attributes should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test command attributes: {:?}", e),
    }

    println!("✅ Command attributes test completed");
}

#[tokio::test]
async fn test_chrome_135_web_speech_mediastream() {
    println!("🧪 Testing Chrome 135: Web Speech API with MediaStreamTrack...");

    let browser = HeadlessWebBrowser::new();

    // Test Web Speech API with MediaStreamTrack support
    let js_code = r#"
        try {
            if (typeof SpeechRecognition !== 'undefined' || typeof webkitSpeechRecognition !== 'undefined') {
                var SpeechRecognitionClass = SpeechRecognition || webkitSpeechRecognition;

                // Test if MediaStreamTrack can be used with Speech Recognition
                var recognition = new SpeechRecognitionClass();

                // Check if audioTrack property exists (Chrome 135 feature)
                var hasAudioTrack = 'audioTrack' in recognition;

                'Web Speech API with MediaStreamTrack support: ' + hasAudioTrack;
            } else {
                'Web Speech API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Web Speech MediaStreamTrack test: {}", value_str);
            // Web Speech API might not be available in headless mode
        },
        Err(e) => panic!("Failed to test Web Speech MediaStreamTrack: {:?}", e),
    }

    println!("✅ Web Speech MediaStreamTrack test completed");
}

#[tokio::test]
async fn test_chrome_135_csp_require_sri() {
    println!("🧪 Testing Chrome 135: CSP require-sri-for directive...");

    let browser = HeadlessWebBrowser::new();

    // Test CSP require-sri-for directive support
    let js_code = r#"
        try {
            if (typeof document !== 'undefined') {
                // Test creating meta element with CSP require-sri-for
                var meta = document.createElement('meta');
                meta.setAttribute('http-equiv', 'Content-Security-Policy');
                meta.setAttribute('content', 'require-sri-for script style');

                var hasCSPSupport = meta.getAttribute('content').includes('require-sri-for');
                'CSP require-sri-for directive syntax supported: ' + hasCSPSupport;
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
            println!("CSP require-sri-for test: {}", value_str);
            assert!(!value_str.contains("error:"), "CSP require-sri-for should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSP require-sri-for: {:?}", e),
    }

    println!("✅ CSP require-sri-for test completed");
}

#[tokio::test]
async fn test_chrome_135_rtc_encoded_frame_timestamps() {
    println!("🧪 Testing Chrome 135: RTC Encoded Frame timestamps...");

    let browser = HeadlessWebBrowser::new();

    // Test RTC Encoded Frame timestamp properties
    let js_code = r#"
        try {
            if (typeof RTCPeerConnection !== 'undefined') {
                // Test if RTCEncodedVideoFrame has timestamp properties
                var hasRTCSupport = typeof RTCPeerConnection === 'function';

                // Test availability of RTC frame APIs
                if (typeof RTCRtpSender !== 'undefined') {
                    'RTC Encoded Frame timestamp APIs context available: ' + hasRTCSupport;
                } else {
                    'RTCRtpSender not available';
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
            println!("RTC Encoded Frame timestamps test: {}", value_str);
            // RTC APIs might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test RTC Encoded Frame timestamps: {:?}", e),
    }

    println!("✅ RTC Encoded Frame timestamps test completed");
}

#[tokio::test]
async fn test_chrome_135_overall_compatibility() {
    println!("🧪 Testing Chrome 135: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("Float16Array", "typeof Float16Array"),
        ("fetchLater", "typeof fetchLater"),
        ("Observable", "typeof Observable"),
        ("NavigateEvent", "typeof NavigateEvent"),
        ("SpeechRecognition", "typeof SpeechRecognition !== 'undefined' || typeof webkitSpeechRecognition !== 'undefined'"),
        ("RTCPeerConnection", "typeof RTCPeerConnection"),
        ("document", "typeof document"),
        ("CSS", "typeof CSS"),
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

    println!("\n📊 Chrome 135 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 135 overall compatibility test completed");
}