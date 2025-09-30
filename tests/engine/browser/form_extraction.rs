use thalora::{HeadlessWebBrowser, Form, FormField, InteractionResponse};
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
    assert_eq!(search_form.action, "https://example.com/"); // Empty action resolves to base URL
    assert_eq!(search_form.method, "get");
    assert_eq!(search_form.fields.len(), 1);
}
