#[tokio::test]
async fn test_geolocation_coordinates_accuracy() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test coordinate values are valid numbers within range
    // (exact values depend on whether IP geolocation succeeds or falls back to San Francisco)
    let result = context.eval(Source::from_bytes(r#"
        let coords = null;
        navigator.geolocation.getCurrentPosition(function(position) {
            coords = position.coords;
        });
        typeof coords.latitude === 'number' &&
        typeof coords.longitude === 'number' &&
        typeof coords.accuracy === 'number' &&
        coords.latitude >= -90 && coords.latitude <= 90 &&
        coords.longitude >= -180 && coords.longitude <= 180 &&
        coords.accuracy > 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
