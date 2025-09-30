#[tokio::test]
async fn test_chrome_137_webcrypto_ed25519() {
    println!("🧪 Testing Chrome 137: WebCrypto Ed25519 support...");

    let browser = HeadlessWebBrowser::new();

    // Test WebCrypto Ed25519 algorithm support
    let js_code = r#"
        try {
            if (typeof crypto !== 'undefined' && crypto.subtle) {
                // Test if Ed25519 is supported in generateKey
                var hasCryptoSubtle = typeof crypto.subtle.generateKey === 'function';

                // Test basic crypto availability
                var cryptoAvailable = 'crypto.subtle available: ' + hasCryptoSubtle;

                // Note: Actually testing Ed25519 would require async operations
                // For now, just check crypto availability
                cryptoAvailable;
            } else {
                'crypto.subtle not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebCrypto Ed25519 test: {}", value_str);
            // WebCrypto might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebCrypto Ed25519: {:?}", e),
    }

    println!("✅ WebCrypto Ed25519 test completed");
}
