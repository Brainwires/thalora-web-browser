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
