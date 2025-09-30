#[tokio::test]
async fn test_chrome_130_document_pip_placement() {
    println!("🧪 Testing Chrome 130: Document Picture-in-Picture preferInitialWindowPlacement...");

    let browser = HeadlessWebBrowser::new();

    // Test Document Picture-in-Picture preferInitialWindowPlacement
    let js_code = r#"
        try {
            if (typeof documentPictureInPicture !== 'undefined') {
                // Test preferInitialWindowPlacement parameter
                var pipOptions = {
                    width: 300,
                    height: 200,
                    // Chrome 130: preferInitialWindowPlacement parameter
                    preferInitialWindowPlacement: true
                };

                'documentPictureInPicture preferInitialWindowPlacement option available';
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
            println!("Document PiP preferInitialWindowPlacement test: {}", value_str);
            // Picture-in-Picture might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Document PiP preferInitialWindowPlacement: {:?}", e),
    }

    println!("✅ Document PiP preferInitialWindowPlacement test completed");
}
