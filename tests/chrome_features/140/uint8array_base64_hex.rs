use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_uint8array_base64_hex() {
    println!("🧪 Testing Chrome 140: Uint8Array Base64/Hex conversion...");

    let browser = HeadlessWebBrowser::new();

    // Test Uint8Array Base64/Hex conversion methods
    let js_code = r#"
        try {
            if (typeof Uint8Array !== 'undefined') {
                var array = new Uint8Array([72, 101, 108, 108, 111]); // "Hello"

                // Chrome 140: Base64/Hex conversion methods
                var hasToBase64 = typeof array.toBase64 === 'function';
                var hasFromBase64 = typeof Uint8Array.fromBase64 === 'function';
                var hasToHex = typeof array.toHex === 'function';
                var hasFromHex = typeof Uint8Array.fromHex === 'function';

                'Uint8Array Base64/Hex methods - toBase64: ' + hasToBase64 +
                ', fromBase64: ' + hasFromBase64 +
                ', toHex: ' + hasToHex +
                ', fromHex: ' + hasFromHex;
            } else {
                'Uint8Array not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Uint8Array Base64/Hex test: {}", value_str);
            assert!(!value_str.contains("error:"), "Uint8Array Base64/Hex should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Uint8Array Base64/Hex: {:?}", e),
    }

    println!("✅ Uint8Array Base64/Hex test completed");
}
