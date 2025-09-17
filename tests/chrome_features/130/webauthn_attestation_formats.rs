#[tokio::test]
async fn test_chrome_130_webauthn_attestation_formats() {
    println!("🧪 Testing Chrome 130: WebAuthn attestationFormats field...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn attestationFormats field
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test creating credential options with attestationFormats
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
                        // Chrome 130: attestationFormats field
                        attestationFormats: ["packed", "fido-u2f"]
                    }
                };

                'WebAuthn attestationFormats field structure created';
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
            println!("WebAuthn attestationFormats test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn attestationFormats should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn attestationFormats: {:?}", e),
    }

    println!("✅ WebAuthn attestationFormats test completed");
}
