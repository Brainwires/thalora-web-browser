use thalora::{HeadlessWebBrowser, Form, FormField, InteractionResponse};
use std::collections::HashMap;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};
use url::Url;

#[tokio::test]
async fn test_error_handling() {
    let mut browser = HeadlessWebBrowser::new();

    // Test invalid URL
    let result = browser.scrape("not-a-url", false, None, false, false).await;
    assert!(result.is_err());

    // Test invalid CSS selector for link clicking
    let result = browser.click_link("https://example.com", "invalid[selector", false).await;
    assert!(result.is_err());

    // Test form submission with invalid URL
    let form = Form {
        action: "not-a-url".to_string(),
        method: "post".to_string(),
        fields: vec![],
        submit_buttons: vec![],
    };
    let form_data = HashMap::new();
    let result = browser.submit_form(&form, form_data, false).await;
    assert!(result.is_err());
}
