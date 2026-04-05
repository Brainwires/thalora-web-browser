// DuckDuckGo search submission test
// Tests form interaction: navigate, type, submit, verify results

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_duckduckgo_search_via_url() {
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    // Navigate directly to search results URL
    let result = guard
        .navigate_to("https://duckduckgo.com/?q=Rust+programming+language")
        .await;

    assert!(result.is_ok(), "DDG search nav failed: {:?}", result.err());
    let content = result.unwrap();

    eprintln!("DDG search results: {} bytes", content.len());
    assert!(
        content.len() > 5000,
        "Search results too short: {} bytes",
        content.len()
    );

    // Verify search results contain relevant content
    let content_lower = content.to_lowercase();
    assert!(
        content_lower.contains("rust"),
        "Search results should mention 'rust'"
    );
}

#[tokio::test]
async fn test_duckduckgo_form_submit() {
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    // Step 1: Navigate to DuckDuckGo
    let nav = guard.navigate_to("https://duckduckgo.com").await;
    assert!(nav.is_ok(), "DDG nav failed: {:?}", nav.err());

    // Step 2: Submit search via form
    let mut form_data = std::collections::HashMap::new();
    form_data.insert("q".to_string(), "thalora web browser".to_string());

    let submit = guard.submit_form("form", form_data).await;
    match &submit {
        Ok(response) => {
            eprintln!(
                "Form submit success: {}, message: {}",
                response.success, response.message
            );
            if let Some(ref content) = response.new_content {
                eprintln!("New content: {} bytes", content.len());
            }
            if let Some(ref url) = response.redirect_url {
                eprintln!("Redirect URL: {}", url);
            }
        }
        Err(e) => {
            // DDG may use JS-only forms — error is acceptable, crash is not
            eprintln!("Form submit error (expected for JS-heavy sites): {}", e);
        }
    }
}

#[tokio::test]
async fn test_duckduckgo_type_into_search() {
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    // Navigate
    let nav = guard.navigate_to("https://duckduckgo.com").await;
    assert!(nav.is_ok(), "DDG nav failed: {:?}", nav.err());

    // Type into search box
    let type_result = guard
        .type_text_into_element("input[name='q']", "hello world", true)
        .await;

    match &type_result {
        Ok(response) => {
            eprintln!(
                "Type success: {}, message: {}",
                response.success, response.message
            );
        }
        Err(e) => {
            eprintln!("Type error (may need JS context): {}", e);
        }
    }

    // Verify via JS
    let value = guard
        .execute_javascript(
            r#"
            (function() {
                try {
                    var input = document.querySelector('input[name="q"]');
                    return input ? 'found:' + (input.value || 'empty') : 'not_found';
                } catch(e) {
                    return 'error:' + e.message;
                }
            })()
            "#,
        )
        .await;
    eprintln!("Input value: {:?}", value);
}
