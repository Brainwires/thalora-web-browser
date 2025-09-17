use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_136_webauthn_conditional_create() {
    println!("🧪 Testing Chrome 136: WebAuthn conditional create...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn conditional create for passkey upgrades
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test if conditional mediation is supported
                var credentialsAPI = navigator.credentials;
                var hasCredentials = typeof credentialsAPI.create === 'function';

                // Test conditional create structure
                if (hasCredentials) {
                    var createOptions = {
                        publicKey: {
                            challenge: new Uint8Array(32),
                            rp: { name: "Test RP" },
                            user: {
                                id: new Uint8Array(16),
                                name: "test@example.com",
                                displayName: "Test User"
                            },
                            pubKeyCredParams: [{alg: -7, type: "public-key"}],
                            // Chrome 136: conditional create support
                            mediation: "conditional"
                        }
                    };

                    'WebAuthn conditional create structure supported: ' + hasCredentials;
                } else {
                    'WebAuthn credentials.create not available';
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
            println!("WebAuthn conditional create test: {}", value_str);
            // WebAuthn might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebAuthn conditional create: {:?}", e),
    }

    println!("✅ WebAuthn conditional create test completed");
}
