use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_navigator_geolocation_exists() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test navigator.geolocation exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");
}

#[tokio::test]
async fn test_geolocation_get_current_position() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test getCurrentPosition method exists
    let result = context.eval(Source::from_bytes("typeof navigator.geolocation.getCurrentPosition")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test getCurrentPosition with callback
    let result = context.eval(Source::from_bytes(r#"
        let positionReceived = false;
        let latitude = null;
        let longitude = null;

        navigator.geolocation.getCurrentPosition(function(position) {
            positionReceived = true;
            latitude = position.coords.latitude;
            longitude = position.coords.longitude;
        });

        // Check if position data is set correctly
        positionReceived && latitude === 37.7749 && longitude === -122.4194;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_geolocation_position_object() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test position object structure
    let result = context.eval(Source::from_bytes(r#"
        let coordsValid = false;
        let timestampValid = false;

        navigator.geolocation.getCurrentPosition(function(position) {
            // Check coords object has required properties
            coordsValid = position.coords &&
                         typeof position.coords.latitude === "number" &&
                         typeof position.coords.longitude === "number" &&
                         typeof position.coords.accuracy === "number" &&
                         position.coords.altitude === null &&
                         position.coords.altitudeAccuracy === null &&
                         position.coords.heading === null &&
                         position.coords.speed === null;

            // Check timestamp exists
            timestampValid = typeof position.timestamp === "number";
        });

        coordsValid && timestampValid;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_geolocation_watch_position() {
    let mut context = Context::default();
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

#[tokio::test]
async fn test_geolocation_clear_watch() {
    let mut context = Context::default();
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

#[tokio::test]
async fn test_geolocation_error_handling() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test getCurrentPosition requires callback
    let result = context.eval(Source::from_bytes(r#"
        try {
            navigator.geolocation.getCurrentPosition();
            "no_error";
        } catch (e) {
            e.message;
        }
    "#)).unwrap();
    let error_msg = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(error_msg.contains("requires a success callback"));

    // Test watchPosition requires callback
    let result = context.eval(Source::from_bytes(r#"
        try {
            navigator.geolocation.watchPosition();
            "no_error";
        } catch (e) {
            e.message;
        }
    "#)).unwrap();
    let error_msg = result.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(error_msg.contains("requires a success callback"));
}

#[tokio::test]
async fn test_geolocation_coordinates_accuracy() {
    let mut context = Context::default();
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