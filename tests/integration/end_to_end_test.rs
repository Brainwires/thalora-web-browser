use thalora::HeadlessWebBrowser;
use std::collections::HashMap;
use wiremock::{matchers::{method, path, body_string_contains}, Mock, MockServer, ResponseTemplate};

/// Test complete user journey: homepage -> login -> dashboard
#[tokio::test]
async fn test_complete_user_journey() {
    let mock_server = MockServer::start().await;

    let homepage = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Welcome</h1>
        <a href="/login">Login</a>
    </body>
    </html>
    "#;

    let login_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Login</h1>
        <form id="login-form" action="/authenticate" method="post">
            <input type="text" name="username" id="username" />
            <input type="password" name="password" id="password" />
            <button type="submit">Submit</button>
        </form>
    </body>
    </html>
    "#;

    let dashboard = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Dashboard</h1>
        <p>Welcome, testuser!</p>
    </body>
    </html>
    "#;

    // Mock homepage
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(homepage))
        .mount(&mock_server)
        .await;

    // Mock login page
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(login_page))
        .mount(&mock_server)
        .await;

    // Mock authentication
    Mock::given(method("POST"))
        .and(path("/authenticate"))
        .and(body_string_contains("username=testuser"))
        .respond_with(ResponseTemplate::new(200).set_body_string(dashboard))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Step 1: Visit homepage
    let content = browser.navigate_to(&mock_server.uri()).await.unwrap();
    assert!(content.contains("Welcome"));

    // Step 2: Click login link
    let response = browser.click_link("Login").await.unwrap();
    assert!(response.success);
    assert!(response.new_content.as_ref().unwrap().contains("Login"));

    // Step 3: Fill in login form
    browser.type_text_into_element("#username", "testuser", false).await.unwrap();
    browser.type_text_into_element("#password", "password123", false).await.unwrap();

    // Step 4: Submit form
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "testuser".to_string());
    form_data.insert("password".to_string(), "password123".to_string());

    let response = browser.submit_form("#login-form", form_data).await.unwrap();

    // Step 5: Verify dashboard
    assert!(response.success);
    assert!(response.new_content.unwrap().contains("Dashboard"));
}

/// Test multi-page search workflow
#[tokio::test]
async fn test_search_workflow() {
    let mock_server = MockServer::start().await;

    let search_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Search</h1>
        <form action="/search" method="get">
            <input type="text" name="q" id="search-input" />
            <button type="submit">Search</button>
        </form>
    </body>
    </html>
    "#;

    let results_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Search Results</h1>
        <div class="result">
            <a href="/result/1">First Result</a>
        </div>
        <div class="result">
            <a href="/result/2">Second Result</a>
        </div>
    </body>
    </html>
    "#;

    let detail_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Result Details</h1>
        <p>This is the detailed content.</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(search_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_string(results_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/result/1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(detail_page))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Navigate to search page
    browser.navigate_to(&mock_server.uri()).await.unwrap();

    // Enter search query
    browser.type_text_into_element("#search-input", "test query", false).await.unwrap();

    // Submit search
    let mut form_data = HashMap::new();
    form_data.insert("q".to_string(), "test query".to_string());
    browser.submit_form("form", form_data).await.unwrap();

    // Click first result
    let response = browser.click_link("First Result").await.unwrap();
    assert!(response.new_content.unwrap().contains("Result Details"));
}

/// Test shopping cart workflow
#[tokio::test]
async fn test_shopping_cart_workflow() {
    let mock_server = MockServer::start().await;

    let products_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Products</h1>
        <div class="product">
            <h2>Product 1</h2>
            <a href="/add-to-cart/1">Add to Cart</a>
        </div>
        <a href="/cart">View Cart</a>
    </body>
    </html>
    "#;

    let cart_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Shopping Cart</h1>
        <div class="cart-item">Product 1</div>
        <a href="/checkout">Checkout</a>
    </body>
    </html>
    "#;

    let checkout_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Checkout</h1>
        <form action="/place-order" method="post">
            <input type="text" name="address" id="address" />
            <button type="submit">Place Order</button>
        </form>
    </body>
    </html>
    "#;

    let confirmation_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Order Confirmed</h1>
        <p>Thank you for your order!</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(products_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/add-to-cart/1"))
        .respond_with(ResponseTemplate::new(302)
            .insert_header("Location", format!("{}/cart", mock_server.uri()).as_str()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/cart"))
        .respond_with(ResponseTemplate::new(200).set_body_string(cart_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/checkout"))
        .respond_with(ResponseTemplate::new(200).set_body_string(checkout_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/place-order"))
        .respond_with(ResponseTemplate::new(200).set_body_string(confirmation_page))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // View products
    browser.navigate_to(&mock_server.uri()).await.unwrap();

    // Add product to cart
    browser.click_link("Add to Cart").await.unwrap();

    // View cart (should redirect)
    let content = browser.get_current_content();
    assert!(content.contains("Shopping Cart"));

    // Proceed to checkout
    browser.click_link("Checkout").await.unwrap();

    // Fill shipping info and place order
    browser.type_text_into_element("#address", "123 Main St", false).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("address".to_string(), "123 Main St".to_string());
    let response = browser.submit_form("form", form_data).await.unwrap();

    assert!(response.new_content.unwrap().contains("Order Confirmed"));
}

/// Test account registration workflow
#[tokio::test]
async fn test_registration_workflow() {
    let mock_server = MockServer::start().await;

    let signup_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Sign Up</h1>
        <form action="/register" method="post">
            <input type="text" name="username" id="username" />
            <input type="email" name="email" id="email" />
            <input type="password" name="password" id="password" />
            <button type="submit">Register</button>
        </form>
    </body>
    </html>
    "#;

    let success_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Registration Successful</h1>
        <p>Please check your email to verify your account.</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/signup"))
        .respond_with(ResponseTemplate::new(200).set_body_string(signup_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/register"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_page))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/signup", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Fill registration form
    browser.type_text_into_element("#username", "newuser", false).await.unwrap();
    browser.type_text_into_element("#email", "newuser@example.com", false).await.unwrap();
    browser.type_text_into_element("#password", "securepass123", false).await.unwrap();

    // Submit registration
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "newuser".to_string());
    form_data.insert("email".to_string(), "newuser@example.com".to_string());
    form_data.insert("password".to_string(), "securepass123".to_string());

    let response = browser.submit_form("form", form_data).await.unwrap();
    assert!(response.new_content.unwrap().contains("Registration Successful"));
}

/// Test paginated content browsing
#[tokio::test]
async fn test_pagination_workflow() {
    let mock_server = MockServer::start().await;

    let page1 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Articles - Page 1</h1>
        <div>Article 1</div>
        <div>Article 2</div>
        <a href="/page/2">Next Page</a>
    </body>
    </html>
    "#;

    let page2 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Articles - Page 2</h1>
        <div>Article 3</div>
        <div>Article 4</div>
        <a href="/page/1">Previous Page</a>
        <a href="/page/3">Next Page</a>
    </body>
    </html>
    "#;

    let page3 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Articles - Page 3</h1>
        <div>Article 5</div>
        <div>Article 6</div>
        <a href="/page/2">Previous Page</a>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/page/1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page/2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page2))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page/3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page3))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Start on page 1
    let url = format!("{}/page/1", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Go to page 2
    browser.click_link("Next Page").await.unwrap();
    assert!(browser.get_current_content().contains("Page 2"));

    // Go to page 3
    browser.click_link("Next Page").await.unwrap();
    assert!(browser.get_current_content().contains("Page 3"));

    // Go back to page 2
    browser.click_link("Previous Page").await.unwrap();
    assert!(browser.get_current_content().contains("Page 2"));
}

/// Test file download simulation
#[tokio::test]
async fn test_download_workflow() {
    let mock_server = MockServer::start().await;

    let download_page = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Downloads</h1>
        <a href="/files/document.pdf">Download PDF</a>
    </body>
    </html>
    "#;

    let pdf_content = b"%PDF-1.4 fake pdf content";

    Mock::given(method("GET"))
        .and(path("/downloads"))
        .respond_with(ResponseTemplate::new(200).set_body_string(download_page))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/files/document.pdf"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_bytes(pdf_content)
            .insert_header("Content-Type", "application/pdf")
            .insert_header("Content-Disposition", "attachment; filename=document.pdf"))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    let url = format!("{}/downloads", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();

    // Click download link
    let response = browser.click_link("Download PDF").await.unwrap();
    assert!(response.success);
}

/// Test multi-step form workflow
#[tokio::test]
async fn test_multi_step_form_workflow() {
    let mock_server = MockServer::start().await;

    let step1 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Step 1: Personal Info</h1>
        <form action="/form/step2" method="post">
            <input type="text" name="name" id="name" />
            <button type="submit">Next</button>
        </form>
    </body>
    </html>
    "#;

    let step2 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Step 2: Contact Info</h1>
        <form action="/form/step3" method="post">
            <input type="email" name="email" id="email" />
            <button type="submit">Next</button>
        </form>
    </body>
    </html>
    "#;

    let step3 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Step 3: Preferences</h1>
        <form action="/form/submit" method="post">
            <input type="checkbox" name="newsletter" id="newsletter" />
            <button type="submit">Complete</button>
        </form>
    </body>
    </html>
    "#;

    let complete = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Form Complete</h1>
        <p>Thank you for completing the form!</p>
    </body>
    </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/form/step1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(step1))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/form/step2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(step2))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/form/step3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(step3))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/form/submit"))
        .respond_with(ResponseTemplate::new(200).set_body_string(complete))
        .mount(&mock_server)
        .await;

    let browser_arc = HeadlessWebBrowser::new();
    let mut browser = browser_arc.lock().unwrap();

    // Step 1
    let url = format!("{}/form/step1", mock_server.uri());
    browser.navigate_to(&url).await.unwrap();
    browser.type_text_into_element("#name", "John Doe", false).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("name".to_string(), "John Doe".to_string());
    browser.submit_form("form", form_data).await.unwrap();

    // Step 2
    assert!(browser.get_current_content().contains("Step 2"));
    browser.type_text_into_element("#email", "john@example.com", false).await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("email".to_string(), "john@example.com".to_string());
    browser.submit_form("form", form_data).await.unwrap();

    // Step 3
    assert!(browser.get_current_content().contains("Step 3"));
    browser.click_element("#newsletter").await.unwrap();

    let mut form_data = HashMap::new();
    form_data.insert("newsletter".to_string(), "on".to_string());
    let response = browser.submit_form("form", form_data).await.unwrap();

    // Verify completion
    assert!(response.new_content.unwrap().contains("Form Complete"));
}

/// Test session persistence across pages
#[tokio::test]
async fn test_session_persistence() {
    let mock_server = MockServer::start().await;

    let page1 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Page 1</h1>
        <a href="/page2">Go to Page 2</a>
        <script>
            sessionStorage.setItem('visited_page1', 'true');
        </script>
    </body>
    </html>
    "#;

    let page2 = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <h1>Page 2</h1>
        <script>
            const visited = sessionStorage.getItem('visited_page1');
            if (visited) {
                document.body.innerHTML += '<p>Session data found!</p>';
            }
        </script>
    </body>
    </html>
    "#;

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

    // Visit page 1
    let url = format!("{}/page1", mock_server.uri());
    browser.navigate_to_with_js_option(&url, true, true).await.unwrap();

    // Navigate to page 2
    browser.click_link("Go to Page 2").await.unwrap();
}
