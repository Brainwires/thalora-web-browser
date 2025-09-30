use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_css_selector_extraction() {
    let html_content = r#"
    <html>
    <body>
        <div class="content">
            <h2>Main Content</h2>
            <p>Some important text.</p>
        </div>
        <div class="sidebar">
            <h3>Sidebar</h3>
            <p>Sidebar content.</p>
        </div>
    </body>
    </html>
    "#;

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html_content))
        .mount(&mock_server)
        .await;

    let mut scraper = HeadlessWebBrowser::new();
    let result = scraper.scrape(
        &mock_server.uri(),
        false,
        Some(".content"), // only extract content div
        false,
        false,
    ).await.unwrap();

    assert!(result.content.contains("Main Content"));
    assert!(!result.content.contains("Sidebar"));
}
