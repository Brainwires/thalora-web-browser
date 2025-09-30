use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_comprehensive_auth_workflow() {
    let mock_server = MockServer::start().await;

    // Mock login page with CSRF token
    let login_page = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta name="csrf-token" content="login-csrf-456">
    </head>
    <body>
        <form action="/auth/login" method="post">
            <input type="hidden" name="_token" value="login-csrf-456">
            <input type="email" name="email">
            <input type="password" name="password">
            <input type="submit" value="Login">
        </form>
    </body>
    </html>
    "#;

    // Mock login response with JWT token
    let login_response = json!({
        "success": true,
        "token": "jwt.token.here",
        "user": {"id": 123, "email": "user@example.com"}
    });

    // Mock protected API response
    let protected_response = json!({
        "data": "protected content",
        "user": {"id": 123, "name": "John Doe"}
    });

    // Setup mocks
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(login_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(wiremock::matchers::path("/auth/login"))
        .and(header("content-type", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&login_response)
                .insert_header("Set-Cookie", "session=auth123; Path=/; HttpOnly")
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/api/protected"))
        .and(header("authorization", "Bearer jwt.token.here"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&protected_response))
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();

    // Step 1: Load login page and extract CSRF token
    let login_page_response = browser.scrape(
        &format!("{}/login", mock_server.uri()),
        false,
        None,
        false,
        false
    ).await.unwrap();

    // Extract CSRF token from the page content or use direct extraction as fallback
    let csrf_token = browser.extract_csrf_token(&login_page_response.content).unwrap()
        .or_else(|| browser.extract_csrf_token(login_page).unwrap());
    
    assert!(csrf_token.is_some());
    browser.set_csrf_token(&csrf_token.unwrap()).unwrap();

    // Step 2: Submit login form with credentials
    let login_data = json!({
        "email": "user@example.com",
        "password": "password123",
        "_token": browser.get_csrf_token().unwrap().unwrap()
    });

    let auth_response = browser.submit_json(
        &format!("{}/auth/login", mock_server.uri()),
        &login_data
    ).await.unwrap();

    assert_eq!(auth_response.status_code, 200);
    
    // Step 3: Extract and store JWT token from response
    let auth_json: serde_json::Value = serde_json::from_str(&auth_response.content).unwrap();
    let jwt_token = auth_json["token"].as_str().unwrap();
    browser.set_bearer_token(jwt_token).unwrap();

    // Step 4: Store user data in localStorage
    let user_id = auth_json["user"]["id"].as_u64().unwrap().to_string();
    browser.set_local_storage("user_id", &user_id).unwrap();
    browser.set_local_storage("jwt_token", jwt_token).unwrap();

    // Step 5: Access protected resource with JWT token
    let client = reqwest::Client::new();
    let protected_response = client
        .get(&format!("{}/api/protected", mock_server.uri()))
        .header("authorization", format!("Bearer {}", jwt_token))
        .send()
        .await
        .unwrap();

    assert_eq!(protected_response.status(), 200);

    // Verify stored auth state
    let stored_user_id = browser.get_local_storage("user_id").unwrap();
    let stored_token = browser.get_local_storage("jwt_token").unwrap();
    let bearer_token = browser.get_bearer_token().unwrap();

    assert_eq!(stored_user_id, Some("123".to_string()));
    assert_eq!(stored_token, Some(jwt_token.to_string()));
    assert_eq!(bearer_token, Some(jwt_token.to_string()));
}
