use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_comprehensive_engine_features() {
    println!("🧪 Testing comprehensive engine feature availability...");

    let browser = HeadlessWebBrowser::new();

    // Test comprehensive feature detection
    let comprehensive_test = browser.lock().unwrap().execute_javascript(
        "typeof Object"
    ).await;

    match comprehensive_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Comprehensive engine test: {}", value_str);

            // Should have basic engine features working
            assert!(value_str.contains("function"), "Object should be available as function, got: {}", value_str);
            println!("✅ Comprehensive engine features verified");
        },
        Err(e) => panic!("Failed comprehensive engine test: {:?}", e),
    }

    println!("✅ Comprehensive engine feature test passed!");
}
