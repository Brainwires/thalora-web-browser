use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_bearer_token_authentication() {
    let mock_server = MockServer::start().await;

    // Mock API endpoint that requires Bearer token
    Mock::given(method("POST"))
        .and(header("authorization", "Bearer test-token-123"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "success": true,
                    "data": "authenticated request"
                }))
        )
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // Set bearer token
    browser.set_bearer_token("test-token-123").unwrap();
    let stored_token = browser.get_bearer_token().unwrap();
    assert_eq!(stored_token, Some("test-token-123".to_string()));

    // Submit JSON request with bearer token
    let response = browser.submit_json(
        &format!("{}/api/protected", mock_server.uri()),
        &json!({"action": "get_user_data"})
    ).await.unwrap();

    assert_eq!(response.status_code, 200);
    assert!(response.content.contains("authenticated request"));
}
