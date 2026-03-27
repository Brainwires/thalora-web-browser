#[tokio::test]
async fn test_geolocation_clear_watch() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test clearWatch method exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation.clearWatch")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test clearWatch can be called with watch ID
    let result = context.eval(Source::from_bytes(r#"
        const watchId = navigator.geolocation.watchPosition(function(position) {});
        navigator.geolocation.clearWatch(watchId);
        true; // Should not throw error
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
