use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_web_api_availability() {
    println!("🧪 Testing Web API availability (WARNING: Many are MOCK implementations)...");

    let browser = HeadlessWebBrowser::new();

    // Test Web API availability
    let web_apis = vec![
        // Core APIs
        ("Window", "typeof window"),
        ("Document", "typeof document"),
        ("Navigator", "typeof navigator"),
        ("Location", "typeof location"),
        ("History", "typeof history"),

        // Storage APIs
        ("localStorage", "typeof localStorage"),
        ("sessionStorage", "typeof sessionStorage"),

        // Network APIs
        ("fetch", "typeof fetch"),
        ("XMLHttpRequest", "typeof XMLHttpRequest"),
        ("WebSocket", "typeof WebSocket"),

        // Modern APIs
        ("Crypto", "typeof crypto"),
        ("Performance", "typeof performance"),
        ("Geolocation", "typeof navigator.geolocation"),

        // Device APIs (Chrome 131+ features)
        ("WebHID", "typeof navigator.hid"),
        ("USB", "typeof navigator.usb"),
        ("Serial", "typeof navigator.serial"),
        ("Bluetooth", "typeof navigator.bluetooth"),

        // Graphics APIs
        ("Canvas", "typeof document.createElement('canvas').getContext"),

        // Media APIs
        ("getUserMedia", "typeof navigator.mediaDevices"),

        // Worker APIs
        ("Worker", "typeof Worker"),
        ("ServiceWorker", "typeof navigator.serviceWorker"),

        // Utility APIs
        ("Permissions", "typeof navigator.permissions"),
        ("Clipboard", "typeof navigator.clipboard"),
    ];

    let mut available_apis = 0;
    let mut missing_apis = 0;

    for (api_name, js_check) in web_apis {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("undefined") {
                    println!("❌ {}: Not available", api_name);
                    missing_apis += 1;
                } else {
                    println!("✅ {}: Available ({})", api_name, value_str);
                    available_apis += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", api_name, e);
                missing_apis += 1;
            }
        }
    }

    println!("\n📊 Web API Availability Results:");
    println!("  ✅ Available: {}", available_apis);
    println!("  ❌ Missing: {}", missing_apis);
    println!("  📈 Coverage: {:.1}%", (available_apis as f64 / (available_apis + missing_apis) as f64) * 100.0);

    assert!(available_apis > 10, "Should have at least 10 Web APIs available");
}
