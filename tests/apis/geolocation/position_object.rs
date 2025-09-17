use thalora::apis::WebApis;
use boa_engine::{Context, Source};

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
