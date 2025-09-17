use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_134_digital_credential_api() {
    println!("🧪 Testing Chrome 134: Digital Credential API (Origin Trial)...");

    let browser = HeadlessWebBrowser::new();

    // Test Digital Credential API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.credentials) {
                // Test if DigitalCredential is available
                if (typeof DigitalCredential !== 'undefined') {
                    'DigitalCredential API available';
                } else if (navigator.credentials.get) {
                    // Test if digital credential options are supported
                    try {
                        // This would normally require origin trial token
                        'navigator.credentials.get available (digital credential may require origin trial)';
                    } catch (credError) {
                        'Digital Credential API: ' + credError.message;
                    }
                } else {
                    'Digital Credential API not available';
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
            println!("Digital Credential API test: {}", value_str);
            // Digital Credential API is in Origin Trial, might not be available
        },
        Err(e) => panic!("Failed to test Digital Credential API: {:?}", e),
    }

    println!("✅ Digital Credential API test completed");
}
