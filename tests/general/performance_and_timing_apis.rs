use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_performance_and_timing_apis() {
    println!("🧪 Testing MOCK Performance API (WARNING: Returns fake data)...");

    let browser = HeadlessWebBrowser::new();

    // Test performance and timing APIs with actual functionality
    let performance_tests = vec![
        // Basic Performance API
        ("performance.now()", "typeof performance.now() === 'number'"),
        ("performance.timeOrigin", "typeof performance.timeOrigin === 'number'"),

        // Navigation Timing
        ("performance.timing", "typeof performance.timing === 'object'"),
        ("performance.timing.navigationStart", "typeof performance.timing.navigationStart === 'number'"),
        ("performance.timing.loadEventEnd", "typeof performance.timing.loadEventEnd === 'number'"),

        // Resource Timing
        ("performance.getEntries", "typeof performance.getEntries === 'function'"),
        ("performance.getEntriesByType", "typeof performance.getEntriesByType === 'function'"),
        ("performance.getEntriesByName", "typeof performance.getEntriesByName === 'function'"),

        // User Timing
        ("performance.mark", "typeof performance.mark === 'function'"),
        ("performance.measure", "typeof performance.measure === 'function'"),
        ("performance.clearMarks", "typeof performance.clearMarks === 'function'"),
        ("performance.clearMeasures", "typeof performance.clearMeasures === 'function'"),

        // Modern Performance APIs
        ("PerformanceObserver", "typeof PerformanceObserver !== 'undefined'"),
        ("performance.observer", "typeof PerformanceObserver === 'function'"),
    ];

    let mut performance_working = 0;
    let mut performance_broken = 0;

    for (api_name, js_test) in performance_tests {
        let result = browser.lock().unwrap().execute_javascript(js_test).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") {
                    println!("✅ {}: Working", api_name);
                    performance_working += 1;
                } else {
                    println!("❌ {}: Not working ({})", api_name, value_str);
                    performance_broken += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error - {:?}", api_name, e);
                performance_broken += 1;
            }
        }
    }

    println!("\n📊 Performance API Results:");
    println!("  ✅ Working: {}", performance_working);
    println!("  ❌ Broken: {}", performance_broken);
    println!("  📈 Performance API Coverage: {:.1}%", (performance_working as f64 / (performance_working + performance_broken) as f64) * 100.0);

    assert!(performance_working > 0, "At least some performance APIs should work");
}
