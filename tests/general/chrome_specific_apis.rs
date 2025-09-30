use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_chrome_specific_apis() {
    println!("🧪 Testing Chrome-specific API availability...");

    let browser = HeadlessWebBrowser::new();

    // Test Chrome-specific APIs that other browsers might not have
    let chrome_specific_tests = vec![
        // Chrome DevTools APIs
        ("chrome.devtools", "typeof chrome !== 'undefined' && typeof chrome.devtools !== 'undefined'"),
        ("chrome.extension", "typeof chrome !== 'undefined' && typeof chrome.extension !== 'undefined'"),
        ("chrome.runtime", "typeof chrome !== 'undefined' && typeof chrome.runtime !== 'undefined'"),

        // Chrome-specific Web APIs
        ("webkitRequestFileSystem", "typeof webkitRequestFileSystem"),
        ("webkitStorageInfo", "typeof navigator.webkitStorageInfo"),
        ("chrome.app", "typeof chrome !== 'undefined' && typeof chrome.app !== 'undefined'"),

        // Chrome Performance APIs
        ("performance.measureUserAgentSpecificMemory", "typeof performance.measureUserAgentSpecificMemory"),
        ("performance.mark", "typeof performance.mark"),
        ("performance.measure", "typeof performance.measure"),

        // Chrome Security APIs
        ("window.isSecureContext", "typeof window.isSecureContext !== 'undefined'"),
        ("window.origin", "typeof window.origin !== 'undefined'"),
        ("document.visibilityState", "typeof document.visibilityState !== 'undefined'"),
    ];

    let mut chrome_api_available = 0;
    let mut chrome_api_missing = 0;

    for (api_name, js_check) in chrome_specific_tests {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("undefined") || value_str.contains("false") {
                    println!("❌ {}: Not available", api_name);
                    chrome_api_missing += 1;
                } else {
                    println!("✅ {}: Available ({})", api_name, value_str);
                    chrome_api_available += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", api_name, e);
                chrome_api_missing += 1;
            }
        }
    }

    println!("\n📊 Chrome-Specific API Results:");
    println!("  ✅ Available: {}", chrome_api_available);
    println!("  ❌ Missing: {}", chrome_api_missing);
    println!("  📈 Chrome API Coverage: {:.1}%", (chrome_api_available as f64 / (chrome_api_available + chrome_api_missing) as f64) * 100.0);

    // Chrome APIs are expected to be mostly missing in a non-Chrome browser
    assert!(chrome_api_missing >= chrome_api_available, "Most Chrome-specific APIs should be missing in Thalora");
}
