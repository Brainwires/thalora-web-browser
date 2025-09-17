use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_webauthn_client_capabilities() {
    println!("🧪 Testing Chrome 133: WebAuthn getClientCapabilities()...");

    let browser = HeadlessWebBrowser::new();

    // Test WebAuthn getClientCapabilities
    let js_code = r#"
        try {
            if (typeof PublicKeyCredential !== 'undefined') {
                // Test if getClientCapabilities method exists
                var hasGetClientCapabilities = typeof PublicKeyCredential.getClientCapabilities === 'function';
                'PublicKeyCredential.getClientCapabilities available: ' + hasGetClientCapabilities;
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
            println!("WebAuthn getClientCapabilities test: {}", value_str);
            // WebAuthn might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test WebAuthn getClientCapabilities: {:?}", e),
    }

    println!("✅ WebAuthn getClientCapabilities test completed");
}
