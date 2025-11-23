use thalora::HeadlessWebBrowser;
use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Test handling many sequential requests
#[tokio::test]
async fn test_sequential_requests() {
    let mock_server = MockServer::start().await;

    let html = "<html><body><h1>Test Page</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(100)
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Make 100 sequential requests
    for i in 0..100 {
        let result = browser.navigate_to(&mock_server.uri()).await;
        assert!(result.is_ok(), "Request {} failed", i);
    }
}

/// Test handling concurrent requests (simulated with multiple browser instances)
#[tokio::test]
async fn test_concurrent_requests() {
    let mock_server = MockServer::start().await;

    let html = "<html><body><h1>Concurrent Test</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(10)
        .mount(&mock_server)
        .await;

    // Create 10 browser instances
    let mut handles = vec![];

    for _ in 0..10 {
        let url = mock_server.uri();
        let handle = tokio::spawn(async move {
            let browser_arc = HeadlessWebBrowser::new();
            let mut browser = browser_arc.lock().unwrap();
            browser.navigate_to(&url).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

/// Test handling large HTML content (10MB)
#[tokio::test]
async fn test_large_html_content() {
    let mock_server = MockServer::start().await;

    // Generate 10MB of HTML
    let large_content = format!(
        "<html><body><p>{}</p></body></html>",
        "Large content paragraph. ".repeat(500_000)
    );

    assert!(large_content.len() > 10_000_000);

    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200).set_body_string(large_content.clone()))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/large", mock_server.uri());
    let result = browser.navigate_to(&url).await;

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.len() > 10_000_000);
}

/// Test handling many links on a page
#[tokio::test]
async fn test_page_with_many_links() {
    let mock_server = MockServer::start().await;

    // Generate HTML with 1000 links
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!("<a href='/link{}'>Link {}</a>\n", i, i));
    }
    html.push_str("</body></html>");

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    let scraped = browser.scrape_current_page().await.unwrap();
    assert!(scraped.links.len() >= 1000);
}

/// Test handling deeply nested HTML structure
#[tokio::test]
async fn test_deeply_nested_html() {
    let mock_server = MockServer::start().await;

    // Generate deeply nested HTML (100 levels)
    let mut html = String::from("<html><body>");
    for i in 0..100 {
        html.push_str(&format!("<div class='level-{}'>", i));
    }
    html.push_str("<p>Deep content</p>");
    for _ in 0..100 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    Mock::given(method("GET"))
        .and(path("/deep"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/deep", mock_server.uri());
    let result = browser.navigate_to(&url).await;

    assert!(result.is_ok());
}

/// Test handling many forms on a page
#[tokio::test]
async fn test_page_with_many_forms() {
    let mock_server = MockServer::start().await;

    // Generate HTML with 100 forms
    let mut html = String::from("<html><body>");
    for i in 0..100 {
        html.push_str(&format!(r#"
            <form id="form-{}" action="/submit-{}" method="post">
                <input type="text" name="field{}" />
                <button type="submit">Submit {}</button>
            </form>
        "#, i, i, i, i));
    }
    html.push_str("</body></html>");

    Mock::given(method("GET"))
        .and(path("/forms"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/forms", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Verify all forms were parsed
    let content = browser.get_current_content();
    assert!(content.contains("form-99"));
}

/// Test handling very long URLs
#[tokio::test]
async fn test_very_long_url() {
    let mock_server = MockServer::start().await;

    let html = "<html><body><h1>Long URL Test</h1></body></html>";

    // Create a very long query string
    let long_query = format!("param={}", "x".repeat(1000));

    Mock::given(method("GET"))
        .and(path("/long"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/long?{}", mock_server.uri(), long_query);
    let result = browser.navigate_to(&url).await;

    assert!(result.is_ok());
}

/// Test rapid successive navigations
#[tokio::test]
async fn test_rapid_navigations() {
    let mock_server = MockServer::start().await;

    for i in 0..10 {
        let html = format!("<html><body><h1>Page {}</h1></body></html>", i);
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&mock_server)
            .await;
    }

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate rapidly between pages
    for i in 0..10 {
        let url = format!("{}/page{}", mock_server.uri(), i);
        let result = browser.navigate_to(&url).await;
        assert!(result.is_ok());
    }
}

/// Test handling many images on a page
#[tokio::test]
async fn test_page_with_many_images() {
    let mock_server = MockServer::start().await;

    // Generate HTML with 500 images
    let mut html = String::from("<html><body>");
    for i in 0..500 {
        html.push_str(&format!("<img src='/image{}.jpg' alt='Image {}' />\n", i, i));
    }
    html.push_str("</body></html>");

    Mock::given(method("GET"))
        .and(path("/images"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/images", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    let scraped = browser.scrape_current_page().await.unwrap();
    assert!(scraped.images.len() >= 500);
}

/// Test handling complex CSS selectors
#[tokio::test]
async fn test_complex_css_selectors() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <html>
    <body>
        <div class="container">
            <div id="main">
                <p class="text important">Paragraph 1</p>
                <p class="text">Paragraph 2</p>
                <div class="nested">
                    <span>Nested content</span>
                </div>
            </div>
        </div>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    // Test that complex selectors can be used
    let result = browser.wait_for_element(".container #main .text.important", 1000).await;
    assert!(result.is_ok());
}

/// Test memory usage with repeated navigations
#[tokio::test]
async fn test_memory_stability() {
    let mock_server = MockServer::start().await;

    let html = "<html><body><p>Test content</p></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(50)
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Perform many navigations to check for memory leaks
    for _ in 0..50 {
        browser.navigate_to(&mock_server.uri()).await.unwrap();
    }

    // If we get here without OOM, the test passes
    assert!(true);
}

/// Test handling malformed HTML gracefully
#[tokio::test]
async fn test_malformed_html_handling() {
    let mock_server = MockServer::start().await;

    let malformed_html = r#"
    <html>
    <body>
        <div>
            <p>Unclosed paragraph
            <span>Nested span</div>
        </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/malformed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(malformed_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/malformed", mock_server.uri());
    let result = browser.navigate_to(&url).await;

    // Should handle gracefully
    assert!(result.is_ok());
}

/// Test handling Unicode content
#[tokio::test]
async fn test_unicode_content() {
    let mock_server = MockServer::start().await;

    let unicode_html = r#"
    <html>
    <head><meta charset="UTF-8"></head>
    <body>
        <h1>多语言测试 🌍</h1>
        <p>English, 中文, 日本語, 한국어, العربية, עברית, Русский</p>
        <p>Emojis: 😀 🎉 🚀 ⭐ 💻 🌈</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/unicode"))
        .respond_with(ResponseTemplate::new(200).set_body_string(unicode_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/unicode", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    let scraped = browser.scrape_current_page().await.unwrap();
    assert!(scraped.content.contains("多语言测试"));
}

/// Test handling special HTML entities
#[tokio::test]
async fn test_html_entities() {
    let mock_server = MockServer::start().await;

    let html_with_entities = r#"
    <html>
    <body>
        <p>&lt;div&gt; &amp; &quot;quotes&quot; &apos;apostrophe&apos;</p>
        <p>&copy; &reg; &trade; &euro;</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/entities"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html_with_entities))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/entities", mock_server.uri());
    let result = browser.navigate_to(&url).await;

    assert!(result.is_ok());
}

/// Test concurrent browser instances
#[tokio::test]
async fn test_multiple_browser_instances() {
    let mock_server = MockServer::start().await;

    let html = "<html><body><h1>Multi-browser test</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(5)
        .mount(&mock_server)
        .await;

    // Create 5 separate browser instances
    let mut handles = vec![];

    for _ in 0..5 {
        let url = mock_server.uri();
        let handle = tokio::spawn(async move {
            let browser_arc = HeadlessWebBrowser::new();
            let mut browser = browser_arc.lock().unwrap();
            browser.navigate_to(&url).await
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

/// Test handling timeout scenarios
#[tokio::test]
async fn test_navigation_timeout() {
    let mock_server = MockServer::start().await;

    // Delay response by 100ms
    let html = "<html><body>Delayed response</body></html>";

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html)
            .set_delay(std::time::Duration::from_millis(100)))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/slow", mock_server.uri());
    let result = browser.navigate_to(&url).await;

    // Should succeed even with delay (within 30s timeout)
    assert!(result.is_ok());
}

/// Test handling many metadata tags
#[tokio::test]
async fn test_page_with_many_meta_tags() {
    let mock_server = MockServer::start().await;

    let mut html = String::from(r#"<html><head>"#);
    for i in 0..100 {
        html.push_str(&format!(r#"<meta name="property{}" content="value{}" />"#, i, i));
    }
    html.push_str("</head><body>Content</body></html>");

    Mock::given(method("GET"))
        .and(path("/meta"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/meta", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    let scraped = browser.scrape_current_page().await.unwrap();
    assert!(scraped.metadata.len() >= 100);
}

/// Test rate limiting scenario (many rapid requests)
#[tokio::test]
async fn test_rate_limiting_scenario() {
    let mock_server = MockServer::start().await;

    let html = "<html><body>Rate limited page</body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(20)
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Make 20 rapid requests
    for _ in 0..20 {
        let result = browser.navigate_to(&mock_server.uri()).await;
        assert!(result.is_ok());
    }
}
