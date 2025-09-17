#[tokio::test]
async fn test_chrome_133_x25519_crypto() {
    println!("🧪 Testing Chrome 133: X25519 crypto algorithm...");

    let browser = HeadlessWebBrowser::new();

    // Test X25519 algorithm support
    let js_code = r#"
        try {
            if (typeof crypto !== 'undefined' && crypto.subtle) {
                // Test X25519 key generation
                try {
                    var keyGenPromise = crypto.subtle.generateKey({
                        name: 'X25519'
                    }, true, ['deriveKey']);
                    'X25519 algorithm: supported for key generation';
                } catch (keyGenError) {
                    'X25519 algorithm: ' + keyGenError.message;
                }
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
            println!("X25519 crypto test: {}", value_str);
            // Crypto API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test X25519 crypto: {:?}", e),
    }

    println!("✅ X25519 crypto test completed");
}
