use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_custom_headers() {
    let mock_server = MockServer::start().await;

    // Mock API that requires custom headers
    Mock::given(method("POST"))
        .and(header("x-api-key", "secret-api-key"))
        .and(header("x-client-version", "1.0.0"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "success"}))
        )
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // Set custom headers
    browser.set_custom_header("x-api-key", "secret-api-key").unwrap();
    browser.set_custom_header("x-client-version", "1.0.0").unwrap();

    let headers = browser.get_custom_headers().unwrap();
    assert_eq!(headers.get("x-api-key"), Some(&"secret-api-key".to_string()));
    assert_eq!(headers.get("x-client-version"), Some(&"1.0.0".to_string()));

    // Submit request with custom headers
    let response = browser.submit_json(
        &format!("{}/api/custom", mock_server.uri()),
        &json!({"test": "data"})
    ).await.unwrap();

    assert_eq!(response.status_code, 200);
}
