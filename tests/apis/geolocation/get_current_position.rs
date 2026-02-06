#[tokio::test]
async fn test_geolocation_get_current_position() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test getCurrentPosition method exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation.getCurrentPosition")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");
    // Test getCurrentPosition with callback
    // (exact values depend on whether IP geolocation succeeds or falls back to San Francisco)
    let result = context.eval(Source::from_bytes(r#"
        let positionReceived = false;
        let latitude = null;
        let longitude = null;
        navigator.geolocation.getCurrentPosition(function(position) {
            positionReceived = true;
            latitude = position.coords.latitude;
            longitude = position.coords.longitude;
        });
        // Check if position data is set correctly (valid coordinate range)
        positionReceived &&
        typeof latitude === 'number' && latitude >= -90 && latitude <= 90 &&
        typeof longitude === 'number' && longitude >= -180 && longitude <= 180;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
