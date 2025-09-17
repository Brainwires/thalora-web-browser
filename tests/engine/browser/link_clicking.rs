use thalora::{HeadlessWebBrowser, Form, FormField, InteractionResponse};
use std::collections::HashMap;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};
use url::Url;

async fn test_link_clicking() {
    let mock_server = MockServer::start().await;
    
    let initial_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Home Page</h1>
        <a href="/about" class="nav-link">About</a>
        <a href="/contact">Contact</a>
        <a href="https://external.com">External</a>
        <nav>
            <a href="/about" class="nav-link">About Us</a>
        </nav>
    </body>
    </html>
    "#;

    let about_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>About Us</h1>
        <p>Welcome to our about page!</p>
    </body>
    </html>
    "#;

    // Mock the initial page
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(initial_page))
        .mount(&mock_server)
        .await;

    // Mock the about page
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/about"))
        .respond_with(ResponseTemplate::new(200).set_body_string(about_page))
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // First navigate to the base page to get the links
    let _home_response = browser.scrape(
        &mock_server.uri(),
        false,
        None,
        false,
        false
    ).await.unwrap();
    
    // Then click on the "About" link using CSS selector  
    let response = browser.click_link(
        &mock_server.uri(),
        "a[href='/about']",
        false
    ).await.unwrap();

    // Verify we navigated to the about page
    assert_eq!(response.status_code, 200);
    assert!(response.content.contains("About Us"));
    assert!(response.content.contains("Welcome to our about page"));
}
