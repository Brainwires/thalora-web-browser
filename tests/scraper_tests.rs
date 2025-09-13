use synaptic::{HeadlessWebBrowser, ScrapedData};
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

#[tokio::test]
async fn test_data_extraction_with_selectors() {
    let html = r#"
    <html>
    <body>
        <div class="product">
            <h2 class="title">Awesome Widget</h2>
            <span class="price">$29.99</span>
            <p class="description">A really cool widget that does amazing things.</p>
        </div>
        <div class="product">
            <h2 class="title">Super Gadget</h2>
            <span class="price">$49.99</span>
            <p class="description">An incredible gadget for all your needs.</p>
        </div>
    </body>
    </html>
    "#;

    let scraper = HeadlessWebBrowser::new();
    let mut selectors = Map::new();
    selectors.insert("titles".to_string(), json!(".title"));
    selectors.insert("prices".to_string(), json!(".price"));
    selectors.insert("first_description".to_string(), json!(".description:first-child"));

    let result = scraper.extract_data(html, &selectors).await.unwrap();
    
    let titles = result.get("titles").unwrap().as_array().unwrap();
    assert_eq!(titles.len(), 2);
    assert!(titles[0].as_str().unwrap().contains("Awesome Widget"));
    assert!(titles[1].as_str().unwrap().contains("Super Gadget"));

    let prices = result.get("prices").unwrap().as_array().unwrap();
    assert_eq!(prices.len(), 2);
    assert!(prices[0].as_str().unwrap().contains("$29.99"));
    assert!(prices[1].as_str().unwrap().contains("$49.99"));
}

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

#[tokio::test]
async fn test_invalid_url() {
    let mut scraper = HeadlessWebBrowser::new();
    let result = scraper.scrape("not-a-valid-url", false, None, false, false).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_network_error() {
    let mut scraper = HeadlessWebBrowser::new();
    // Try to connect to a non-existent server
    let result = scraper.scrape("http://localhost:99999", false, None, false, false).await;
    
    assert!(result.is_err());
}

#[test]
fn test_link_and_image_structures() {
    use synaptic::{Link, Image};
    
    let link = Link {
        url: "https://example.com".to_string(),
        text: "Example".to_string(),
        title: Some("Example Site".to_string()),
    };
    
    assert_eq!(link.url, "https://example.com");
    assert_eq!(link.text, "Example");
    assert_eq!(link.title, Some("Example Site".to_string()));
    
    let image = Image {
        src: "test.jpg".to_string(),
        alt: Some("Test".to_string()),
        title: None,
    };
    
    assert_eq!(image.src, "test.jpg");
    assert_eq!(image.alt, Some("Test".to_string()));
    assert_eq!(image.title, None);
}