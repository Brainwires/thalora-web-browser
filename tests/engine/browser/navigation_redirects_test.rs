use thalora::HeadlessWebBrowser;
use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

/// Test 301 Moved Permanently redirect
#[tokio::test]
async fn test_301_redirect() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Final Destination</h1></body></html>";

    // Setup redirect
    Mock::given(method("GET"))
        .and(path("/old"))
        .respond_with(
            ResponseTemplate::new(301)
                .insert_header("Location", format!("{}/new", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/new"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to old URL (should follow redirect)
    let content = browser.navigate_to(&format!("{}/old", mock_server.uri())).await.unwrap();

    // Verify we got the final content (reqwest follows redirects automatically)
    assert!(content.contains("Final Destination"));
}

/// Test 302 Found (temporary redirect)
#[tokio::test]
async fn test_302_redirect() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Temporary Location</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/temp"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/target", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/temp", mock_server.uri())).await.unwrap();
    assert!(content.contains("Temporary Location"));
}

/// Test 303 See Other redirect (typically used after POST)
#[tokio::test]
async fn test_303_redirect() {
    let mock_server = MockServer::start().await;

    let result_html = "<html><body><h1>Operation Complete</h1></body></html>";

    Mock::given(method("POST"))
        .and(path("/submit"))
        .respond_with(
            ResponseTemplate::new(303)
                .insert_header("Location", format!("{}/result", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/result"))
        .respond_with(ResponseTemplate::new(200).set_body_string(result_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Simulate POST request that redirects with 303
    // Note: reqwest client will automatically follow and convert POST to GET
    let response = browser.client.post(format!("{}/submit", mock_server.uri()))
        .send()
        .await
        .unwrap();

    let content = response.text().await.unwrap();
    assert!(content.contains("Operation Complete"));
}

/// Test 307 Temporary Redirect (preserves method)
#[tokio::test]
async fn test_307_redirect() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Redirected with method preservation</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/start"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/end", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/end"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/start", mock_server.uri())).await.unwrap();
    assert!(content.contains("Redirected with method preservation"));
}

/// Test 308 Permanent Redirect (preserves method)
#[tokio::test]
async fn test_308_redirect() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Permanent redirect with method</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/old-permanent"))
        .respond_with(
            ResponseTemplate::new(308)
                .insert_header("Location", format!("{}/new-permanent", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/new-permanent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/old-permanent", mock_server.uri())).await.unwrap();
    assert!(content.contains("Permanent redirect with method"));
}

/// Test redirect chain (multiple redirects in sequence)
#[tokio::test]
async fn test_redirect_chain() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>End of chain</h1></body></html>";

    // First redirect
    Mock::given(method("GET"))
        .and(path("/step1"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/step2", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    // Second redirect
    Mock::given(method("GET"))
        .and(path("/step2"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/step3", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    // Third redirect
    Mock::given(method("GET"))
        .and(path("/step3"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/final", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    // Final destination
    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate through redirect chain
    let content = browser.navigate_to(&format!("{}/step1", mock_server.uri())).await.unwrap();
    assert!(content.contains("End of chain"));
}

/// Test circular redirect detection (reqwest has built-in protection with max redirects)
#[tokio::test]
async fn test_circular_redirect_protection() {
    let mock_server = MockServer::start().await;

    // Create circular redirect: A -> B -> A
    Mock::given(method("GET"))
        .and(path("/a"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/b", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/b"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/a", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Attempt to navigate (should fail due to redirect loop protection in reqwest)
    let result = browser.navigate_to(&format!("{}/a", mock_server.uri())).await;

    // reqwest should detect the loop and return an error
    assert!(result.is_err());
}

/// Test redirect with relative Location header
#[tokio::test]
async fn test_redirect_relative_location() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Relative redirect target</h1></body></html>";

    // Redirect with relative path
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", "/target")
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/redirect", mock_server.uri())).await.unwrap();
    assert!(content.contains("Relative redirect target"));
}

/// Test redirect to different host
#[tokio::test]
async fn test_redirect_cross_origin() {
    let server1 = MockServer::start().await;
    let server2 = MockServer::start().await;

    let final_html = "<html><body><h1>Different server</h1></body></html>";

    // Server 1 redirects to Server 2
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/target", server2.uri()).as_str())
        )
        .mount(&server1)
        .await;

    // Server 2 serves final content
    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&server2)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/redirect", server1.uri())).await.unwrap();
    assert!(content.contains("Different server"));
}

/// Test redirect with query parameters preserved
#[tokio::test]
async fn test_redirect_with_query_params() {
    let mock_server = MockServer::start().await;

    let final_html = "<html><body><h1>Target with params</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/target?preserved=yes", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/redirect?original=param", mock_server.uri())).await.unwrap();
    assert!(content.contains("Target with params"));
}

/// Test POST to GET redirect (common pattern after form submission)
#[tokio::test]
async fn test_post_to_get_redirect() {
    let mock_server = MockServer::start().await;

    let form_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <form id="form" action="/submit" method="post">
            <input type="text" name="data" />
        </form>
    </body>
    </html>
    "#;

    let success_html = "<html><body><h1>Form submitted successfully</h1></body></html>";

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(form_html))
        .mount(&mock_server)
        .await;

    // POST endpoint that redirects
    Mock::given(method("POST"))
        .and(path("/submit"))
        .respond_with(
            ResponseTemplate::new(303)
                .insert_header("Location", format!("{}/success", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    // Success page
    Mock::given(method("GET"))
        .and(path("/success"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    browser.navigate_to(&mock_server.uri()).await.unwrap();

    let mut form_data = std::collections::HashMap::new();
    form_data.insert("data".to_string(), "test".to_string());

    let response = browser.submit_form("#form", form_data).await.unwrap();

    // Should follow redirect to success page
    assert!(response.success);
    assert!(response.new_content.unwrap().contains("Form submitted successfully"));
}

/// Test 304 Not Modified (should not have body)
#[tokio::test]
async fn test_304_not_modified() {
    let mock_server = MockServer::start().await;

    // Note: 304 responses typically don't redirect but indicate cached content is still valid
    Mock::given(method("GET"))
        .and(path("/cached"))
        .respond_with(ResponseTemplate::new(304))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/cached", mock_server.uri())).await.unwrap();

    // 304 responses have empty body
    assert!(content.is_empty());
}

/// Test redirect limit (reqwest default is 10 redirects)
#[tokio::test]
async fn test_excessive_redirect_chain() {
    let mock_server = MockServer::start().await;

    // Create a chain of 15 redirects (exceeds default limit of 10)
    for i in 1..=15 {
        let next_step = if i < 15 {
            format!("{}/step{}", mock_server.uri(), i + 1)
        } else {
            format!("{}/final", mock_server.uri())
        };

        Mock::given(method("GET"))
            .and(path(format!("/step{}", i)))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("Location", next_step.as_str())
            )
            .mount(&mock_server)
            .await;
    }

    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html><body>Final</body></html>"))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // This should fail because it exceeds the redirect limit
    let result = browser.navigate_to(&format!("{}/step1", mock_server.uri())).await;

    // reqwest should return an error for too many redirects
    assert!(result.is_err());
}

/// Test redirect with fragment identifier
#[tokio::test]
async fn test_redirect_with_fragment() {
    let mock_server = MockServer::start().await;

    let final_html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Page with sections</h1>
        <div id="section1">Section 1</div>
        <div id="section2">Section 2</div>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/target#section2", mock_server.uri()).as_str())
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/target"))
        .respond_with(ResponseTemplate::new(200).set_body_string(final_html))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let content = browser.navigate_to(&format!("{}/redirect", mock_server.uri())).await.unwrap();
    assert!(content.contains("Page with sections"));
    assert!(content.contains("section2"));
}
