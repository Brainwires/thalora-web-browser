#[tokio::test]
async fn test_chrome_139_overall_compatibility() {
    println!("🧪 Testing Chrome 139: Overall feature compatibility...");

    let browser = HeadlessWebBrowser::new();

    let features = vec![
        ("navigator.credentials", "typeof navigator !== 'undefined' && typeof navigator.credentials"),
        ("SpeechRecognition", "typeof SpeechRecognition !== 'undefined' || typeof webkitSpeechRecognition !== 'undefined'"),
        ("CSS.supports", "typeof CSS !== 'undefined' && typeof CSS.supports"),
        ("navigator.gpu", "typeof navigator !== 'undefined' && typeof navigator.gpu"),
        ("SharedWorker", "typeof SharedWorker"),
        ("window", "typeof window"),
        ("document", "typeof document"),
        ("navigator", "typeof navigator"),
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

    println!("\n📊 Chrome 139 Feature Summary:");
    println!("  ✅ Available: {}", available);
    println!("  ❌ Missing: {}", total - available);
    println!("  📈 Coverage: {:.1}%", (available as f64 / total as f64) * 100.0);

    println!("✅ Chrome 139 overall compatibility test completed");
}
