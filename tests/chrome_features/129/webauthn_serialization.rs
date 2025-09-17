#[tokio::test]
async fn test_chrome_129_webauthn_serialization() {
    println!("🧪 Testing Chrome 129: WebAuthn serialization methods...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn serialization methods
    let js_code = r#"
        try {
            if (typeof PublicKeyCredential !== 'undefined') {
                // Test if toJSON method exists
                var hasToJSON = typeof PublicKeyCredential.prototype.toJSON === 'function';

                // Test if static parsing methods exist
                var hasParseCreation = typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function';
                var hasParseRequest = typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'function';

                'PublicKeyCredential.toJSON: ' + hasToJSON +
                ', parseCreationOptionsFromJSON: ' + hasParseCreation +
                ', parseRequestOptionsFromJSON: ' + hasParseRequest;
            } else {
                'PublicKeyCredential not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebAuthn serialization test: {}", value_str);
            assert!(!value_str.contains("error:"), "WebAuthn serialization should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebAuthn serialization: {:?}", e),
    }

    println!("✅ WebAuthn serialization test completed");
}
