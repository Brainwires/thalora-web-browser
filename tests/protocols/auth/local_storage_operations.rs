use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
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
