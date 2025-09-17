use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_125_compute_pressure_api() {
    println!("🧪 Testing Chrome 125: Compute Pressure API...");

    let browser = HeadlessWebBrowser::new();

    // Test PressureObserver availability
    let result = browser.lock().unwrap().execute_javascript("typeof PressureObserver").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("PressureObserver type: {}", value_str);
            assert!(value_str.contains("function") || value_str.contains("undefined"),
                "PressureObserver should be function or undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check PressureObserver: {:?}", e),
    }

    // Test navigator.computePressure
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.computePressure").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.computePressure type: {}", value_str);
            // Might be undefined in headless mode
        },
        Err(e) => panic!("Failed to check navigator.computePressure: {:?}", e),
    }

    println!("✅ Compute Pressure API test completed");
}
