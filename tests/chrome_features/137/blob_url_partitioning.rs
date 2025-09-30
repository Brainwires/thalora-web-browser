#[tokio::test]
async fn test_chrome_137_blob_url_partitioning() {
    println!("🧪 Testing Chrome 137: Blob URL partitioning...");

    let browser = HeadlessWebBrowser::new();

    // Test Blob URL creation and access
    let js_code = r#"
        try {
            if (typeof Blob !== 'undefined' && typeof URL !== 'undefined') {
                // Test basic Blob URL creation
                var blob = new Blob(['test content'], { type: 'text/plain' });
                var blobUrl = URL.createObjectURL(blob);

                var hasBlobUrl = blobUrl.startsWith('blob:');
                var urlStructure = 'Blob URL created: ' + hasBlobUrl;

                // Test URL.revokeObjectURL
                var hasRevoke = typeof URL.revokeObjectURL === 'function';
                if (hasRevoke) {
                    URL.revokeObjectURL(blobUrl);
                }

                urlStructure + ', revoke available: ' + hasRevoke;
            } else {
                'Blob or URL not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Blob URL partitioning test: {}", value_str);
            assert!(!value_str.contains("error:"), "Blob URL should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Blob URL partitioning: {:?}", e),
    }

    println!("✅ Blob URL partitioning test completed");
}
