use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

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
