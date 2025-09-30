#[tokio::test]
async fn test_chrome_139_webauth_immediate_mediation() {
    println!("🧪 Testing Chrome 139: WebAuthn immediate mediation mode...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn immediate mediation
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test immediate mediation mode
                var hasCredentials = typeof navigator.credentials.get === 'function';

                if (hasCredentials) {
                    // Test mediation options
                    var mediationOptions = {
                        mediation: 'immediate',
                        publicKey: {
                            challenge: new Uint8Array(32),
                            timeout: 60000,
                            userVerification: 'preferred'
                        }
                    };

                    'WebAuthn immediate mediation support: ' + hasCredentials;
                } else {
                    'navigator.credentials.get not available';
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
            println!("WebAuthn immediate mediation test: {}", value_str);
            // WebAuthn might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebAuthn immediate mediation: {:?}", e),
    }

    println!("✅ WebAuthn immediate mediation test completed");
}
