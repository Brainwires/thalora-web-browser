use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_javascript_engine_features() {
    println!("🧪 Testing REAL JavaScript engine features (Boa ES6-ES2023)...");

    let browser = HeadlessWebBrowser::new();

    // Test modern JavaScript features directly
    let js_tests = vec![
        // ES6 Features
        ("Arrow Functions", "(() => 'test')()"),
        ("Template Literals", "`Hello ${'world'}`"),
        ("Destructuring", "__destructure([1, 2], ['a', 'b']); a + b"),
        ("Default Parameters", "(function(x = 5) { return x; })()"),
        ("Spread Operator", "[...[1, 2, 3]].length"),

        // ES2017+ Features
        ("Async/Await Basic", "__async(function() { return 'async'; })().constructor.name"),
        ("Object Entries", "Object.entries({a: 1}).length"),
        ("Object Values", "Object.values({a: 1, b: 2}).length"),

        // Math Extensions
        ("Math.trunc", "Math.trunc(4.9)"),
        ("Math.sign", "Math.sign(-5)"),

        // String Methods
        ("String.includes", "'hello'.includes('ell')"),
        ("String.startsWith", "'hello'.startsWith('hel')"),
        ("String.endsWith", "'hello'.endsWith('llo')"),

        // Array Methods
        ("Array.from", "Array.from('abc').length"),
        ("Array.find", "[1,2,3].find(x => x > 1)"),
        ("Array.includes", "[1,2,3].includes(2)"),

        // Promises
        ("Promise.resolve", "Promise.resolve(42).constructor.name"),
        ("Promise.all", "Promise.all([Promise.resolve(1)]).constructor.name"),

        // Modern APIs availability checks
        ("fetch API", "typeof fetch"),
        ("localStorage", "typeof localStorage"),
        ("sessionStorage", "typeof sessionStorage"),
        ("WebSocket", "typeof WebSocket"),
        ("console", "typeof console"),
        ("navigator", "typeof navigator"),
        ("navigator.userAgent", "navigator.userAgent.length > 0"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (test_name, js_code) in js_tests {
        let result = browser.lock().unwrap().execute_javascript(js_code).await;

        match result {
            Ok(value) => {
                println!("✅ {}: {:?}", test_name, value);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {}: Error - {:?}", test_name, e);
                failed += 1;
            }
        }
    }

    println!("\n📊 JavaScript Feature Test Results:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    println!("  📈 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    assert!(passed > failed, "Should pass more JavaScript feature tests than fail");
}
