use thalora::HeadlessWebBrowser;

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
