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
