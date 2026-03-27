#[tokio::test]
async fn test_geolocation_coordinates_accuracy() {
    let mut context = Context::default();
    thalora_browser_apis::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test specific coordinate values (San Francisco)
    let result = context.eval(Source::from_bytes(r#"
        let coords = null;
        navigator.geolocation.getCurrentPosition(function(position) {
            coords = position.coords;
        });
        coords.latitude === 37.7749 &&
        coords.longitude === -122.4194 &&
        coords.accuracy === 100.0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
