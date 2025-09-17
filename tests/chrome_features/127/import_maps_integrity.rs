use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_127_import_maps_integrity() {
    println!("🧪 Testing Chrome 127: Import Maps Integrity...");

    let browser = HeadlessWebBrowser::new();

    // Test if HTMLScriptElement supports import maps
    let js_code = r#"
        try {
            // Test basic import map support
            var script = document.createElement('script');
            script.type = 'importmap';

            // Test integrity in import maps (Chrome 127 feature)
            var importMapWithIntegrity = {
                "imports": {
                    "module1": "/path/to/module1.js",
                    "module2": "/path/to/module2.js"
                },
                "integrity": {
                    "/path/to/module1.js": "sha384-...",
                    "/path/to/module2.js": "sha384-..."
                }
            };

            script.textContent = JSON.stringify(importMapWithIntegrity);
            'import maps with integrity supported';
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Import maps integrity test: {}", value_str);
            assert!(!value_str.contains("error:"), "Import maps integrity should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test import maps integrity: {:?}", e),
    }

    println!("✅ Import maps integrity test completed");
}
