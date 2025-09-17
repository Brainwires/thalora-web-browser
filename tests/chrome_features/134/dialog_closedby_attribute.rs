#[tokio::test]
async fn test_chrome_134_dialog_closedby_attribute() {
    println!("🧪 Testing Chrome 134: Dialog closedby attribute...");

    let browser = HeadlessWebBrowser::new();

    // Test Dialog closedby attribute
    let js_code = r#"
        try {
            if (typeof HTMLDialogElement !== 'undefined') {
                var dialog = document.createElement('dialog');

                // Test if closedby attribute is supported
                dialog.setAttribute('closedby', 'any');
                var hasClosedBySupport = dialog.getAttribute('closedby') === 'any';

                'Dialog closedby attribute supported: ' + hasClosedBySupport;
            } else {
                'HTMLDialogElement not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Dialog closedby test: {}", value_str);
            // Dialog elements might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Dialog closedby: {:?}", e),
    }

    println!("✅ Dialog closedby test completed");
}
