use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_basic_html_scraping() {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
        <meta name="description" content="A test page for scraping">
    </head>
    <body>
        <h1>Welcome to Test Page</h1>
        <p>This is a paragraph with some text.</p>
        <a href="https://example.com" title="Example Link">Example</a>
        <img src="test.jpg" alt="Test Image" title="A test image">
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
        false, // don't wait for JS
        None,
        true,  // extract links
        true,  // extract images
    ).await.unwrap();

    assert_eq!(result.title, Some("Test Page".to_string()));
    assert!(result.content.contains("Welcome to Test Page"));
    assert_eq!(result.links.len(), 1);
    assert!(result.links[0].url.starts_with("https://example.com"));
    assert_eq!(result.links[0].text, "Example");
    assert_eq!(result.images.len(), 1);
    assert!(result.images[0].src.contains("test.jpg"));
    assert_eq!(result.metadata.get("description"), Some(&"A test page for scraping".to_string()));
}
