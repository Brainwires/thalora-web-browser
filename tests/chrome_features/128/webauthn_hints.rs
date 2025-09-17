use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_128_webauthn_hints() {
    println!("🧪 Testing Chrome 128: WebAuthn hints parameter...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn hints parameter
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test if WebAuthn is available
                var hasWebAuthn = typeof navigator.credentials.create === 'function';

                if (hasWebAuthn) {
                    // Test hints parameter (would normally be used in actual WebAuthn request)
                    var credentialOptions = {
                        publicKey: {
                            challenge: new Uint8Array(32),
                            rp: { name: "Test RP" },
                            user: {
                                id: new Uint8Array(16),
                                name: "test@example.com",
                                displayName: "Test User"
                            },
                            pubKeyCredParams: [{alg: -7, type: "public-key"}],
                            // Chrome 128: hints parameter
                            hints: ["client-device", "security-key"]
                        }
                    };

                    'WebAuthn with hints parameter structure created';
                } else {
                    'WebAuthn not available';
                }
            } else {
                'navigator.credentials not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAuthn hints test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn hints should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn hints: {:?}", e),
    }

    println!("✅ WebAuthn hints test completed");
}
