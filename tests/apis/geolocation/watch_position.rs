#[tokio::test]
async fn test_geolocation_watch_position() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test watchPosition method exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation.watchPosition")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test watchPosition returns a watch ID
    let result = context.eval(Source::from_bytes(r#"
        const watchId = navigator.geolocation.watchPosition(function(position) {});
        typeof watchId === "number" && watchId > 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
