use thalora::HeadlessWebBrowser;
use std::collections::HashMap;
use wiremock::{matchers::{method, path, query_param}, Mock, MockServer, ResponseTemplate};
use tokio::time::{sleep, Duration};

/// Test basic navigation to a URL
#[tokio::test]
async fn test_basic_navigation() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body><h1>Welcome</h1></body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&mock_server.uri()).await.unwrap();

    assert!(content.contains("Welcome"));
    assert!(content.contains("<title>Test Page</title>"));
    assert_eq!(browser.get_current_url(), Some(mock_server.uri()));
}

/// Test navigation with wait for load
#[tokio::test]
async fn test_navigation_with_wait_for_load() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Page Content</h1>
        <div id="content">Initial content</div>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/page", mock_server.uri());
    let content = browser.navigate_to_with_options(&url, true).await.unwrap();

    assert!(content.contains("Page Content"));
    assert!(content.contains("Initial content"));
}

/// Test navigation with JavaScript execution
#[tokio::test]
async fn test_navigation_with_javascript_execution() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div id="target">Original</div>
        <script>
            document.getElementById('target').textContent = 'Modified by JS';
        </script>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/js-page"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/js-page", mock_server.uri());
    let content = browser.navigate_to_with_js_option(&url, true, true).await.unwrap();

    assert!(content.contains("target"));
}

/// Test URL resolution for relative paths
#[tokio::test]
async fn test_relative_url_resolution() {
    let mock_server = MockServer::start().await;

    let base_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <a href="./relative">Relative Link</a>
        <a href="/absolute">Absolute Link</a>
        <a href="../parent">Parent Link</a>
    </body>
    </html>
    "#;

    let relative_html = "<html><body><h1>Relative Page</h1></body></html>";
    let absolute_html = "<html><body><h1>Absolute Page</h1></body></html>";
    let parent_html = "<html><body><h1>Parent Page</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/base"))
        .respond_with(ResponseTemplate::new(200).set_body_string(base_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/relative"))
        .respond_with(ResponseTemplate::new(200).set_body_string(relative_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/absolute"))
        .respond_with(ResponseTemplate::new(200).set_body_string(absolute_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/parent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(parent_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to base page
    let url = format!("{}/base", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Test clicking relative link
    let response = browser.click_link("Relative Link").await.unwrap();
    assert!(response.success);
    assert!(response.new_content.unwrap().contains("Relative Page"));
}

/// Test clicking links on a page
#[tokio::test]
async fn test_click_link() {
    let mock_server = MockServer::start().await;

    let home_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Home</h1>
        <a href="/about">About Us</a>
        <a href="/contact">Contact</a>
    </body>
    </html>
    "#;

    let about_html = r#"
    <!DOCTYPE html>
    <html>
    <body><h1>About Page</h1></body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(home_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/about"))
        .respond_with(ResponseTemplate::new(200).set_body_string(about_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to home page
    browser.navigate_to(&mock_server.uri()).await.unwrap();

    // Click "About Us" link
    let response = browser.click_link("About").await.unwrap();

    assert!(response.success);
    assert!(response.new_content.is_some());
    assert!(response.new_content.unwrap().contains("About Page"));
    assert!(response.redirect_url.is_some());
}

/// Test typing text into form elements
#[tokio::test]
async fn test_type_text_into_element() {
    let mock_server = MockServer::start().await;

    let form_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form>
            <input type="text" id="username" name="username" />
            <input type="password" id="password" name="password" />
            <textarea id="message" name="message"></textarea>
        </form>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/form"))
        .respond_with(ResponseTemplate::new(200).set_body_string(form_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to form page
    let url = format!("{}/form", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Type into username field
    let response = browser.type_text_into_element("#username", "testuser", false).await.unwrap();
    assert!(response.success);

    // Type into password field
    let response = browser.type_text_into_element("#password", "secret123", false).await.unwrap();
    assert!(response.success);

    // Type into textarea with clear_first = true
    let response = browser.type_text_into_element("#message", "Hello World", true).await.unwrap();
    assert!(response.success);
}

/// Test typing text with clear first option
#[tokio::test]
async fn test_type_text_with_clear() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <input type="text" id="field" value="existing text" />
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

    // Type with clear_first = true
    let response = browser.type_text_into_element("#field", "new text", true).await.unwrap();
    assert!(response.success);
}

/// Test form submission with GET method
#[tokio::test]
async fn test_submit_form_get() {
    let mock_server = MockServer::start().await;

    let form_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form id="search-form" action="/search" method="get">
            <input type="text" name="q" />
            <input type="submit" value="Search" />
        </form>
    </body>
    </html>
    "#;

    let results_html = r#"
    <!DOCTYPE html>
    <html>
    <body><h1>Search Results for: test query</h1></body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(form_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("q", "test query"))
        .respond_with(ResponseTemplate::new(200).set_body_string(results_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("q".to_string(), "test query".to_string());

    let response = browser.submit_form("#search-form", form_data).await.unwrap();

    assert!(response.success);
    assert!(response.new_content.is_some());
    assert!(response.new_content.unwrap().contains("Search Results"));
}

/// Test form submission with POST method
#[tokio::test]
async fn test_submit_form_post() {
    let mock_server = MockServer::start().await;

    let form_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form id="login-form" action="/login" method="post">
            <input type="text" name="username" />
            <input type="password" name="password" />
            <input type="submit" value="Login" />
        </form>
    </body>
    </html>
    "#;

    let success_html = r#"
    <!DOCTYPE html>
    <html>
    <body><h1>Login Successful</h1></body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(form_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "testuser".to_string());
    form_data.insert("password".to_string(), "password123".to_string());

    let response = browser.submit_form("#login-form", form_data).await.unwrap();

    assert!(response.success);
    assert!(response.new_content.unwrap().contains("Login Successful"));
}

/// Test clicking checkbox elements
#[tokio::test]
async fn test_click_checkbox() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form>
            <input type="checkbox" id="agree" name="agree" />
            <label for="agree">I agree</label>
        </form>
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

    let response = browser.click_element("#agree").await.unwrap();
    assert!(response.success);
}

/// Test clicking button elements
#[tokio::test]
async fn test_click_button() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <button id="submit-btn" type="button">Click Me</button>
        <div id="result"></div>
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

    let response = browser.click_element("#submit-btn").await.unwrap();
    assert!(response.success);
}

/// Test page reload
#[tokio::test]
async fn test_reload() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body><h1>Reloadable Page</h1></body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .expect(2) // Expect 2 requests (initial + reload)
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Initial navigation
    browser.navigate_to(&mock_server.uri()).await.unwrap();

    // Reload the page
    let content = browser.reload().await.unwrap();
    assert!(content.contains("Reloadable Page"));
}

/// Test reload with no page loaded
#[tokio::test]
async fn test_reload_no_page() {
    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Attempt to reload without navigating first
    let result = browser.reload().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No current page to reload"));
}

/// Test go_back functionality (currently returns None)
#[tokio::test]
async fn test_go_back() {
    let mock_server = MockServer::start().await;

    let page1 = "<html><body><h1>Page 1</h1></body></html>";
    let page2 = "<html><body><h1>Page 2</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/page1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page2))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to two pages
    browser.navigate_to(&format!("{}/page1", mock_server.uri())).await.unwrap();
    browser.navigate_to(&format!("{}/page2", mock_server.uri())).await.unwrap();

    // Try to go back (currently returns None)
    let result = browser.go_back().await.unwrap();
    assert!(result.is_none()); // Current implementation returns None
}

/// Test go_forward functionality (currently returns None)
#[tokio::test]
async fn test_go_forward() {
    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Try to go forward (currently returns None)
    let result = browser.go_forward().await.unwrap();
    assert!(result.is_none()); // Current implementation returns None
}

/// Test wait_for_element with element present
#[tokio::test]
async fn test_wait_for_element_present() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div id="existing">I exist</div>
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

    // Wait for existing element (should return true immediately)
    let found = browser.wait_for_element("#existing", 1000).await.unwrap();
    assert!(found);
}

/// Test wait_for_element timeout
#[tokio::test]
async fn test_wait_for_element_timeout() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div id="present">I'm here</div>
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

    // Wait for non-existent element (should timeout and return false)
    let found = browser.wait_for_element("#non-existent", 500).await.unwrap();
    assert!(!found);
}

/// Test scraping current page
#[tokio::test]
async fn test_scrape_current_page() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
        <h1>Welcome</h1>
        <a href="/link1">Link 1</a>
        <a href="/link2">Link 2</a>
        <img src="/image.jpg" alt="Test Image" />
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

    let scraped_data = browser.scrape_current_page().await.unwrap();

    assert_eq!(scraped_data.title, Some("Test Page".to_string()));
    assert!(scraped_data.links.len() >= 2);
    assert!(scraped_data.images.len() >= 1);
}

/// Test scrape with no page loaded
#[tokio::test]
async fn test_scrape_no_page_loaded() {
    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let result = browser.scrape_current_page().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No current page loaded"));
}

/// Test external script URL resolution - protocol-relative
#[tokio::test]
async fn test_protocol_relative_script_url() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <script src="//cdn.example.com/script.js"></script>
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

    // Navigate (this will attempt to load external scripts but won't fail if CDN is unreachable)
    let result = browser.navigate_to(&mock_server.uri()).await;
    assert!(result.is_ok());
}

/// Test typing into non-input elements (setting textContent)
#[tokio::test]
async fn test_type_into_non_input_element() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <div id="content">Original content</div>
        <p id="paragraph">Original paragraph</p>
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

    // Type into div element (should set textContent)
    let response = browser.type_text_into_element("#content", "New content", false).await.unwrap();
    assert!(response.success);

    // Type into paragraph element
    let response = browser.type_text_into_element("#paragraph", "New paragraph", false).await.unwrap();
    assert!(response.success);
}

/// Test element click with element not found
#[tokio::test]
async fn test_click_element_not_found() {
    let mock_server = MockServer::start().await;

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <button id="exists">I exist</button>
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

    // Try to click non-existent element
    let result = browser.click_element("#does-not-exist").await;
    assert!(result.is_err());
}

/// Test form submission with absolute action URL
#[tokio::test]
async fn test_submit_form_absolute_url() {
    let mock_server = MockServer::start().await;

    let external_server = MockServer::start().await;

    let form_html = format!(r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form id="form" action="{}/submit" method="post">
            <input type="text" name="data" />
        </form>
    </body>
    </html>
    "#, external_server.uri());

    let success_html = "<html><body><h1>Submitted</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(form_html))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/submit"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_html))
        .mount(&external_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("data".to_string(), "test".to_string());

    let response = browser.submit_form("#form", form_data).await.unwrap();
    assert!(response.success);
    assert!(response.new_content.unwrap().contains("Submitted"));
}

/// Test multiple sequential navigations
#[tokio::test]
async fn test_sequential_navigations() {
    let mock_server = MockServer::start().await;

    let page1 = "<html><body><h1>Page 1</h1></body></html>";
    let page2 = "<html><body><h1>Page 2</h1></body></html>";
    let page3 = "<html><body><h1>Page 3</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/page1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page2))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page3))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate through multiple pages
    let content1 = browser.navigate_to(&format!("{}/page1", mock_server.uri())).await.unwrap();
    assert!(content1.contains("Page 1"));

    let content2 = browser.navigate_to(&format!("{}/page2", mock_server.uri())).await.unwrap();
    assert!(content2.contains("Page 2"));

    let content3 = browser.navigate_to(&format!("{}/page3", mock_server.uri())).await.unwrap();
    assert!(content3.contains("Page 3"));

    // Verify current URL is the last navigated page
    assert_eq!(browser.get_current_url(), Some(format!("{}/page3", mock_server.uri())));
}
