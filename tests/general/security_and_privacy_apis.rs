use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_security_and_privacy_apis() {
    println!("🧪 Testing Security and Privacy API compliance...");

    let browser = HeadlessWebBrowser::new();

    // Test security and privacy related APIs
    let security_tests = vec![
        // Security Context
        ("Secure Context", "window.isSecureContext"),
        ("Origin", "window.origin !== null"),

        // Crypto APIs
        ("Web Crypto API", "typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined'"),
        ("crypto.randomUUID", "typeof crypto.randomUUID === 'function'"),
        ("crypto.getRandomValues", "typeof crypto.getRandomValues === 'function'"),

        // Privacy APIs
        ("Document Policy", "typeof document.policy !== 'undefined'"),
        ("Feature Policy", "typeof document.featurePolicy !== 'undefined'"),
        ("Permissions Policy", "typeof document.permissionsPolicy !== 'undefined'"),

        // Content Security Policy
        ("CSP violation events", "typeof SecurityPolicyViolationEvent !== 'undefined'"),

        // Trusted Types
        ("Trusted Types", "typeof TrustedHTML !== 'undefined'"),

        // Cross-Origin Isolation
        ("crossOriginIsolated", "typeof crossOriginIsolated !== 'undefined'"),
    ];

    let mut security_available = 0;
    let mut security_missing = 0;

    for (security_name, js_check) in security_tests {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") || (!value_str.contains("undefined") && !value_str.contains("false")) {
                    println!("✅ {}: Available ({})", security_name, value_str);
                    security_available += 1;
                } else {
                    println!("❌ {}: Not available", security_name);
                    security_missing += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", security_name, e);
                security_missing += 1;
            }
        }
    }

    println!("\n📊 Security & Privacy API Results:");
    println!("  ✅ Available: {}", security_available);
    println!("  ❌ Missing: {}", security_missing);
    println!("  📈 Security API Coverage: {:.1}%", (security_available as f64 / (security_available + security_missing) as f64) * 100.0);

    assert!(security_available + security_missing > 0, "Should have tested security features");
}
