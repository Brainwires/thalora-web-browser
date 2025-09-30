use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_malformed_html() {
    let html_content = r#"
    <html>
    <head><title>Broken HTML</head>
    <body>
        <p>This is a paragraph without closing tag
        <div>Nested content
        <span>Some text</span>
    </body>
    "#;

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html_content))
        .mount(&mock_server)
        .await;

    let mut scraper = HeadlessWebBrowser::new();
    let result = scraper.scrape(&mock_server.uri(), false, None, false, false).await;

    // Should handle malformed HTML gracefully
    assert!(result.is_ok());
    let scraped = result.unwrap();
    assert!(scraped.title.is_some());
    assert!(scraped.title.unwrap().contains("Broken HTML"));
}
