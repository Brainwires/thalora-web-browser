use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_geolocation_api_comprehensive() {
    let mut context = Context::default();
    thalora::apis::polyfills::console::setup_console(&mut context).expect("Failed to setup console");
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test navigator.geolocation exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");

    // Test geolocation methods exist
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.geolocation.getCurrentPosition === "function" &&
        typeof navigator.geolocation.watchPosition === "function" &&
        typeof navigator.geolocation.clearWatch === "function"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getCurrentPosition functionality
    let result = context.eval(Source::from_bytes(r#"
        let positionReceived = false;
        let coords = null;
        let timestamp = null;

        navigator.geolocation.getCurrentPosition(function(position) {
            positionReceived = true;
            coords = position.coords;
            timestamp = position.timestamp;
        });

        positionReceived &&
        typeof coords.latitude === 'number' && coords.latitude >= -90 && coords.latitude <= 90 &&
        typeof coords.longitude === 'number' && coords.longitude >= -180 && coords.longitude <= 180 &&
        typeof coords.accuracy === 'number' && coords.accuracy > 0 &&
        typeof timestamp === "number"
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test watchPosition returns ID
    let result = context.eval(Source::from_bytes(r#"
        const watchId = navigator.geolocation.watchPosition(function() {});
        typeof watchId === "number" && watchId > 0
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    println!("✅ Geolocation API tests passed");
}
