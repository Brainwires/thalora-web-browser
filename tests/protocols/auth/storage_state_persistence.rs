use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

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
