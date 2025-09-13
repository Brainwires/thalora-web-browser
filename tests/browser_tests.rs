use synaptic::{HeadlessWebBrowser, Form, FormField, InteractionResponse};
use serde_json::json;
use std::collections::HashMap;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};
use url::Url;

#[tokio::test]
async fn test_form_extraction() {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Form Page</title></head>
    <body>
        <form action="/login" method="post">
            <input type="text" name="username" placeholder="Username" required>
            <input type="password" name="password" placeholder="Password" required>
            <input type="email" name="email" value="test@example.com">
            <textarea name="message" placeholder="Your message">Default message</textarea>
            <select name="country">
                <option value="us">United States</option>
                <option value="uk" selected>United Kingdom</option>
                <option value="ca">Canada</option>
            </select>
            <input type="submit" value="Login">
            <button type="submit">Submit</button>
        </form>
        <form action="" method="get">
            <input type="search" name="q" placeholder="Search...">
            <input type="submit" value="Search">
        </form>
    </body>
    </html>
    "#;

    let browser = HeadlessWebBrowser::new();
    let base_url = Url::parse("https://example.com").unwrap();
    let forms = browser.extract_forms(html_content, &base_url).unwrap();

    // Should extract 2 forms
    assert_eq!(forms.len(), 2);

    // Test first form (login form)
    let login_form = &forms[0];
    assert_eq!(login_form.action, "https://example.com/login");
    assert_eq!(login_form.method, "post");
    assert_eq!(login_form.fields.len(), 5); // username, password, email, message, country
    assert_eq!(login_form.submit_buttons.len(), 2); // "Login" and "Submit"

    // Test form fields
    let username_field = &login_form.fields[0];
    assert_eq!(username_field.name, "username");
    assert_eq!(username_field.field_type, "text");
    assert_eq!(username_field.placeholder, Some("Username".to_string()));
    assert!(username_field.required);

    let email_field = &login_form.fields[2];
    assert_eq!(email_field.name, "email");
    assert_eq!(email_field.field_type, "email");
    assert_eq!(email_field.value, Some("test@example.com".to_string()));

    let select_field = &login_form.fields[4];
    assert_eq!(select_field.name, "country");
    assert_eq!(select_field.field_type, "select");
    assert_eq!(select_field.value, Some("uk".to_string())); // Selected option

    // Test second form (search form)
    let search_form = &forms[1];
    assert_eq!(search_form.action, "https://example.com"); // Empty action resolves to base URL
    assert_eq!(search_form.method, "get");
    assert_eq!(search_form.fields.len(), 1);
}

#[tokio::test]
async fn test_form_submission_post() {
    let mock_server = MockServer::start().await;
    
    // Mock the form submission endpoint
    Mock::given(method("POST"))
        .and(wiremock::matchers::path("/login"))
        .respond_with(
            ResponseTemplate::new(302)
                .set_body_string("Redirecting...")
                .insert_header("Location", "/dashboard")
                .insert_header("Set-Cookie", "session=abc123; Path=/; HttpOnly")
        )
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // Create a test form
    let form = Form {
        action: format!("{}/login", mock_server.uri()),
        method: "post".to_string(),
        fields: vec![
            FormField {
                name: "username".to_string(),
                field_type: "text".to_string(),
                value: None,
                required: true,
                placeholder: Some("Username".to_string()),
            },
            FormField {
                name: "password".to_string(),
                field_type: "password".to_string(),
                value: None,
                required: true,
                placeholder: Some("Password".to_string()),
            },
        ],
        submit_buttons: vec!["Login".to_string()],
    };

    // Prepare form data
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "testuser".to_string());
    form_data.insert("password".to_string(), "testpass".to_string());

    // Submit the form
    let response = browser.submit_form(&form, form_data, false).await.unwrap();

    // Verify response
    assert_eq!(response.status_code, 302);
    assert!(response.content.contains("Redirecting"));
    assert!(response.cookies.contains_key("session"));
    assert_eq!(response.cookies.get("session"), Some(&"abc123".to_string()));
}

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

#[tokio::test]
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
    
    // Click on the "About" link using CSS selector
    let response = browser.click_link(
        &mock_server.uri(),
        "a.nav-link",
        false
    ).await.unwrap();

    // Verify we navigated to the about page
    assert_eq!(response.status_code, 200);
    assert!(response.content.contains("About Us"));
    assert!(response.content.contains("Welcome to our about page"));
}

#[tokio::test]
async fn test_cookie_persistence() {
    let mock_server = MockServer::start().await;
    
    // First request sets cookies
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/set-cookies"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Cookies set")
                .insert_header("Set-Cookie", "user_id=12345; Path=/")
                .insert_header("Set-Cookie", "session=abcdef; Path=/; HttpOnly")
        )
        .mount(&mock_server)
        .await;

    // Second request should include cookies
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/protected"))
        .and(wiremock::matchers::header("cookie", "user_id=12345; session=abcdef"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Protected content")
        )
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // First request to set cookies
    let _response1 = browser.scrape(
        &format!("{}/set-cookies", mock_server.uri()),
        false,
        None,
        false,
        false
    ).await.unwrap();

    // Check that cookies are stored
    let cookies = browser.get_cookies(&mock_server.uri()).unwrap();
    assert!(cookies.contains_key("user_id"));
    assert!(cookies.contains_key("session"));
    assert_eq!(cookies.get("user_id"), Some(&"12345".to_string()));

    // Second request should automatically include cookies
    let response2 = browser.scrape(
        &format!("{}/protected", mock_server.uri()),
        false,
        None,
        false,
        false
    ).await.unwrap();

    assert_eq!(response2.title, Some("Protected content".to_string()));
}

#[tokio::test]
async fn test_multi_step_workflow() {
    let mock_server = MockServer::start().await;
    
    let login_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Login</h1>
        <form action="/authenticate" method="post">
            <input type="text" name="username" required>
            <input type="password" name="password" required>
            <input type="submit" value="Login">
        </form>
    </body>
    </html>
    "#;

    let dashboard_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Dashboard</h1>
        <p>Welcome back!</p>
        <a href="/profile">View Profile</a>
    </body>
    </html>
    "#;

    let profile_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>User Profile</h1>
        <p>Name: John Doe</p>
    </body>
    </html>
    "#;

    // Mock login page
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(login_page))
        .mount(&mock_server)
        .await;

    // Mock authentication (login form submission)
    Mock::given(method("POST"))
        .and(wiremock::matchers::path("/authenticate"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(dashboard_page)
                .insert_header("Set-Cookie", "auth_token=xyz789; Path=/")
        )
        .mount(&mock_server)
        .await;

    // Mock profile page (requires authentication)
    Mock::given(method("GET"))
        .and(wiremock::matchers::path("/profile"))
        .and(wiremock::matchers::header("cookie", "auth_token=xyz789"))
        .respond_with(ResponseTemplate::new(200).set_body_string(profile_page))
        .mount(&mock_server)
        .await;

    let mut browser = HeadlessWebBrowser::new();
    
    // Step 1: Navigate to login page
    let login_response = browser.scrape(
        &format!("{}/login", mock_server.uri()),
        false,
        None,
        false,
        false
    ).await.unwrap();

    assert!(login_response.title.as_ref().unwrap().contains("Login"));

    // Step 2: Extract and fill login form
    let base_url = Url::parse(&mock_server.uri()).unwrap();
    let forms = browser.extract_forms(&login_response.content, &base_url).unwrap();
    assert_eq!(forms.len(), 1);

    let login_form = &forms[0];
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "testuser".to_string());
    form_data.insert("password".to_string(), "secret123".to_string());

    // Step 3: Submit login form
    let dashboard_response = browser.submit_form(login_form, form_data, false).await.unwrap();
    assert_eq!(dashboard_response.status_code, 200);
    assert!(dashboard_response.content.contains("Dashboard"));
    assert!(dashboard_response.cookies.contains_key("auth_token"));

    // Step 4: Navigate to profile page (cookies automatically sent)
    let profile_response = browser.click_link(
        &dashboard_response.url,
        "a[href='/profile']",
        false
    ).await.unwrap();

    assert_eq!(profile_response.status_code, 200);
    assert!(profile_response.content.contains("User Profile"));
    assert!(profile_response.content.contains("John Doe"));
}

#[tokio::test]
async fn test_error_handling() {
    let mut browser = HeadlessWebBrowser::new();

    // Test invalid URL
    let result = browser.scrape("not-a-url", false, None, false, false).await;
    assert!(result.is_err());

    // Test invalid CSS selector for link clicking
    let result = browser.click_link("https://example.com", "invalid[selector", false).await;
    assert!(result.is_err());

    // Test form submission with invalid URL
    let form = Form {
        action: "not-a-url".to_string(),
        method: "post".to_string(),
        fields: vec![],
        submit_buttons: vec![],
    };
    let form_data = HashMap::new();
    let result = browser.submit_form(&form, form_data, false).await;
    assert!(result.is_err());
}

#[test]
fn test_form_field_structure() {
    let field = FormField {
        name: "email".to_string(),
        field_type: "email".to_string(),
        value: Some("test@example.com".to_string()),
        required: true,
        placeholder: Some("Enter email".to_string()),
    };

    assert_eq!(field.name, "email");
    assert_eq!(field.field_type, "email");
    assert_eq!(field.value, Some("test@example.com".to_string()));
    assert!(field.required);
    assert_eq!(field.placeholder, Some("Enter email".to_string()));
}

#[test]
fn test_interaction_response_structure() {
    let mut cookies = HashMap::new();
    cookies.insert("session".to_string(), "abc123".to_string());

    let response = InteractionResponse {
        url: "https://example.com/result".to_string(),
        status_code: 200,
        content: "<html><body>Success</body></html>".to_string(),
        cookies,
        scraped_data: None,
    };

    assert_eq!(response.url, "https://example.com/result");
    assert_eq!(response.status_code, 200);
    assert!(response.content.contains("Success"));
    assert_eq!(response.cookies.get("session"), Some(&"abc123".to_string()));
    assert!(response.scraped_data.is_none());
}