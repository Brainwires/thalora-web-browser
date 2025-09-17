use thalora::{HeadlessWebBrowser, Form, FormField, InteractionResponse};
use std::collections::HashMap;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};
use url::Url;

#[tokio::test]
async fn test_form_submission_get() {
    let mock_server = MockServer::start().await;
    
    // Mock the search endpoint
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/search"))
        .and(wiremock::matchers::query_param("q", "rust web scraping"))
        .and(wiremock::matchers::query_param("type", "web"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html><body><h1>Search Results</h1></body></html>")
        )
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // Create a search form
    let form = Form {
        action: format!("{}/search", mock_server.uri()),
        method: "get".to_string(),
        fields: vec![
            FormField {
                name: "q".to_string(),
                field_type: "search".to_string(),
                value: None,
                required: false,
                placeholder: Some("Search...".to_string()),
            },
            FormField {
                name: "type".to_string(),
                field_type: "hidden".to_string(),
                value: Some("web".to_string()),
                required: false,
                placeholder: None,
            },
        ],
        submit_buttons: vec!["Search".to_string()],
    };

    // Prepare search data
    let mut form_data = HashMap::new();
    form_data.insert("q".to_string(), "rust web scraping".to_string());
    form_data.insert("type".to_string(), "web".to_string());

    // Submit the search form
    let response = browser.submit_form(&form, form_data, false).await.unwrap();

    // Verify response
    assert_eq!(response.status_code, 200);
    assert!(response.content.contains("Search Results"));
}
