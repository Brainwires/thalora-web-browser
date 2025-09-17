use thalora::apis::WebApis;
use boa_engine::{Context, Source};

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
