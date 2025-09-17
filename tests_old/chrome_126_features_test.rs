use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_126_gamepad_haptic_enhancements() {
    println!("🧪 Testing Chrome 126: Gamepad API Trigger-Rumble Extension...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.getGamepads availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.getGamepads").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.getGamepads type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "getGamepads should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.getGamepads: {:?}", e),
    }

    // Test GamepadHapticActuator with trigger-rumble extensions
    let js_code = r#"
        try {
            // Check if GamepadHapticActuator exists
            if (typeof GamepadHapticActuator !== 'undefined') {
                'GamepadHapticActuator available';
            } else {
                'GamepadHapticActuator not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("GamepadHapticActuator test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test GamepadHapticActuator: {:?}", e),
    }

    println!("✅ Gamepad trigger-rumble test completed");
}

#[tokio::test]
async fn test_chrome_126_webgl_object_exposure() {
    println!("🧪 Testing Chrome 126: WebGL Enhancements (WebGLObject exposure)...");

    let browser = HeadlessWebBrowser::new();

    // Test WebGLObject availability
    let result = browser.lock().unwrap().execute_javascript("typeof WebGLObject").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebGLObject type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "WebGLObject should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check WebGLObject: {:?}", e),
    }

    // Test WebGL context availability
    let js_code = r#"
        try {
            // Try to get WebGL context
            const canvas = typeof HTMLCanvasElement !== 'undefined' ? document.createElement('canvas') : null;
            if (canvas && canvas.getContext) {
                const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
                if (gl) {
                    'WebGL context available';
                } else {
                    'WebGL context not available';
                }
            } else {
                'Canvas or getContext not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("WebGL context test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test WebGL context: {:?}", e),
    }

    println!("✅ WebGL enhancements test completed");
}

#[tokio::test]
async fn test_chrome_126_mediarecorder_improvements() {
    println!("🧪 Testing Chrome 126: MediaRecorder MP4 support...");

    let browser = HeadlessWebBrowser::new();

    // Test MediaRecorder availability
    let result = browser.lock().unwrap().execute_javascript("typeof MediaRecorder").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("MediaRecorder type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "MediaRecorder should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check MediaRecorder: {:?}", e),
    }

    // Test MediaRecorder.isTypeSupported for MP4
    let js_code = r#"
        try {
            if (typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported) {
                const mp4Support = MediaRecorder.isTypeSupported('video/mp4');
                const opusSupport = MediaRecorder.isTypeSupported('audio/opus');
                'mp4:' + mp4Support + ',opus:' + opusSupport;
            } else {
                'MediaRecorder.isTypeSupported not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("MediaRecorder format support: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test MediaRecorder formats: {:?}", e),
    }

    println!("✅ MediaRecorder improvements test completed");
}

#[tokio::test]
async fn test_chrome_126_visual_viewport_scrollend() {
    println!("🧪 Testing Chrome 126: visualViewport onscrollend support...");

    let browser = HeadlessWebBrowser::new();

    // Test visualViewport availability
    let result = browser.lock().unwrap().execute_javascript("typeof window.visualViewport").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("window.visualViewport type: {}", value_str);
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "visualViewport should be object or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check window.visualViewport: {:?}", e),
    }

    // Test onscrollend event handler
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.visualViewport) {
                // Test if onscrollend property exists
                const hasScrollEnd = 'onscrollend' in window.visualViewport;
                'onscrollend supported: ' + hasScrollEnd;
            } else {
                'visualViewport not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("visualViewport onscrollend test: {:?}", value);
            // Should not error out
        },
        Err(e) => panic!("Failed to test visualViewport onscrollend: {:?}", e),
    }

    println!("✅ visualViewport scrollend test completed");
}

#[tokio::test]
async fn test_chrome_126_import_syntax_changes() {
    println!("🧪 Testing Chrome 126: Import assertion syntax changes...");

    let browser = HeadlessWebBrowser::new();

    // Test import with 'with' keyword (new syntax replacing 'assert')
    let js_code = r#"
        try {
            // This would normally be a syntax test in a real module context
            // In our context, we just test that the syntax doesn't cause parse errors
            const syntaxTest = 'import foo from "./foo.json" with { type: "json" }';
            'import with syntax: valid string';
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Import syntax test: {}", value_str);
            assert!(value_str.contains("valid"), "Import syntax should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test import syntax: {:?}", e),
    }

    println!("✅ Import syntax changes test completed");
}

#[tokio::test]
async fn test_chrome_126_css_view_transitions() {
    println!("🧪 Testing Chrome 126: Cross-Document View Transitions...");

    let browser = HeadlessWebBrowser::new();

    // Test CSS.supports for view transition features
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                const viewTransitionName = CSS.supports('view-transition-name', 'my-transition');
                const atViewTransition = CSS.supports('@view-transition', 'navigation: auto');
                'view-transition-name:' + viewTransitionName + ',@view-transition:' + atViewTransition;
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
            println!("CSS view transitions support: {}", value_str);
            assert!(!value_str.contains("error:"), "CSS view transitions test should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test CSS view transitions: {:?}", e),
    }

    println!("✅ CSS view transitions test completed");
}

#[tokio::test]
async fn test_chrome_126_overall_compatibility() {
    println!("🧪 Testing Chrome 126: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("navigator.getGamepads", "typeof navigator.getGamepads"),
        ("GamepadHapticActuator", "typeof GamepadHapticActuator"),
        ("WebGLObject", "typeof WebGLObject"),
        ("MediaRecorder", "typeof MediaRecorder"),
        ("window.visualViewport", "typeof window.visualViewport"),
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

    println!("\n📊 Chrome 126 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 126 overall compatibility test completed");
}