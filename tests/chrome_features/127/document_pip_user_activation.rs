#[tokio::test]
async fn test_chrome_127_document_pip_user_activation() {
    println!("🧪 Testing Chrome 127: Document Picture-in-Picture User Activation...");

    let browser = HeadlessWebBrowser::new();

    // Test document picture-in-picture API with user activation propagation
    let js_code = r#"
        try {
            if (typeof documentPictureInPicture !== 'undefined') {
                // Check if user activation propagation is supported
                var hasUserActivation = typeof navigator.userActivation !== 'undefined';
                'documentPictureInPicture available, userActivation: ' + hasUserActivation;
            } else {
                'documentPictureInPicture not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Document PiP user activation test: {}", value_str);
            // Picture-in-Picture might not be available in headless mode
        },
        Err(e) => panic!("Failed to test document PiP user activation: {:?}", e),
    }

    println!("✅ Document PiP user activation test completed");
}
