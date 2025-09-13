use synaptic::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_local_storage_operations() {
    let browser = HeadlessWebBrowser::new();

    // Test setting and getting localStorage
    browser.set_local_storage("auth_token", "abc123").unwrap();
    browser.set_local_storage("user_id", "12345").unwrap();

    let token = browser.get_local_storage("auth_token").unwrap();
    let user_id = browser.get_local_storage("user_id").unwrap();
    let missing = browser.get_local_storage("nonexistent").unwrap();

    assert_eq!(token, Some("abc123".to_string()));
    assert_eq!(user_id, Some("12345".to_string()));
    assert_eq!(missing, None);

    // Test storage state persistence
    let storage_state = browser.get_storage_state().unwrap();
    assert!(storage_state.local_storage.contains_key("auth_token"));
    assert!(storage_state.local_storage.contains_key("user_id"));
}

#[tokio::test]
async fn test_session_storage_operations() {
    let browser = HeadlessWebBrowser::new();

    // Test setting and getting sessionStorage
    browser.set_session_storage("session_token", "xyz789").unwrap();
    browser.set_session_storage("temp_data", "temporary").unwrap();

    let token = browser.get_session_storage("session_token").unwrap();
    let temp = browser.get_session_storage("temp_data").unwrap();

    assert_eq!(token, Some("xyz789".to_string()));
    assert_eq!(temp, Some("temporary".to_string()));

    // Test clearing session storage
    browser.clear_session_storage().unwrap();
    let cleared_token = browser.get_session_storage("session_token").unwrap();
    assert_eq!(cleared_token, None);
}

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

#[tokio::test]
async fn test_csrf_token_extraction() {
    let html_with_csrf = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta name="csrf-token" content="csrf-abc123">
        <title>CSRF Test</title>
    </head>
    <body>
        <form>
            <input type="hidden" name="_token" value="form-token-456">
        </form>
    </body>
    </html>
    "#;

    let browser = HeadlessWebBrowser::new();
    
    // Test CSRF token extraction
    let csrf_token = browser.extract_csrf_token(html_with_csrf).unwrap();
    assert_eq!(csrf_token, Some("csrf-abc123".to_string()));

    // Test setting and getting CSRF token
    browser.set_csrf_token("manual-csrf-token").unwrap();
    let stored_csrf = browser.get_csrf_token().unwrap();
    assert_eq!(stored_csrf, Some("manual-csrf-token".to_string()));
}

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

#[tokio::test]
async fn test_storage_state_persistence() {
    let browser1 = HeadlessWebBrowser::new();
    
    // Set up storage state in first browser
    browser1.set_local_storage("user_token", "token123").unwrap();
    browser1.set_session_storage("session_id", "session456").unwrap();
    browser1.set_bearer_token("bearer789").unwrap();
    browser1.set_csrf_token("csrf012").unwrap();

    // Export storage state
    let storage_state = browser1.get_storage_state().unwrap();
    
    // Create new browser and restore state
    let browser2 = HeadlessWebBrowser::new();
    browser2.restore_storage_state(storage_state).unwrap();

    // Verify state was restored
    let restored_local = browser2.get_local_storage("user_token").unwrap();
    let restored_session = browser2.get_session_storage("session_id").unwrap();
    
    assert_eq!(restored_local, Some("token123".to_string()));
    assert_eq!(restored_session, Some("session456".to_string()));
}

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

#[test]
fn test_browser_storage_serialization() {
    let mut local_storage = HashMap::new();
    local_storage.insert("token".to_string(), "abc123".to_string());
    
    let mut session_storage = HashMap::new();
    session_storage.insert("session".to_string(), "xyz789".to_string());

    let storage = BrowserStorage {
        local_storage,
        session_storage,
    };

    // Test serialization
    let serialized = serde_json::to_string(&storage).unwrap();
    let deserialized: BrowserStorage = serde_json::from_str(&serialized).unwrap();

    assert_eq!(storage.local_storage, deserialized.local_storage);
    assert_eq!(storage.session_storage, deserialized.session_storage);
}

#[test]
fn test_auth_context_serialization() {
    let mut custom_headers = HashMap::new();
    custom_headers.insert("x-api-key".to_string(), "secret".to_string());

    let storage = BrowserStorage {
        local_storage: HashMap::new(),
        session_storage: HashMap::new(),
    };

    let auth_context = AuthContext {
        bearer_token: Some("jwt.token.here".to_string()),
        csrf_token: Some("csrf123".to_string()),
        custom_headers,
        storage,
    };

    // Test serialization
    let serialized = serde_json::to_string(&auth_context).unwrap();
    let deserialized: AuthContext = serde_json::from_str(&serialized).unwrap();

    assert_eq!(auth_context.bearer_token, deserialized.bearer_token);
    assert_eq!(auth_context.csrf_token, deserialized.csrf_token);
    assert_eq!(auth_context.custom_headers, deserialized.custom_headers);
}