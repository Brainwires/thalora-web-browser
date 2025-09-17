use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_empty_response() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&mock_server)
        .await;

    let mut scraper = HeadlessWebBrowser::new();
    let result = scraper.scrape(&mock_server.uri(), false, None, false, false).await.unwrap();

    assert_eq!(result.title, None);
    assert!(result.content.is_empty());
    assert_eq!(result.links.len(), 0);
    assert_eq!(result.images.len(), 0);
}
