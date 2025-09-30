use thalora::{HeadlessWebBrowser, ScrapedData};
use serde_json::{json, Map};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

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
