use thalora::{HeadlessWebBrowser, BrowserStorage, AuthContext};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::{method, header}, Mock, MockServer, ResponseTemplate};

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
