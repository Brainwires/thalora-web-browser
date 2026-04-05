// Real-world site loading tests
// Validates that the browser can navigate to and load content from popular sites
// without crashing from any of our recent changes (SecurityContext, CSP, SRI,
// event bubbling, module loader, etc.)

use thalora::HeadlessWebBrowser;

/// Helper: navigate to a URL and return (content_length, has_errors)
async fn load_site(url: &str) -> (usize, Option<String>) {
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    match guard.navigate_to(url).await {
        Ok(content) => {
            let len = content.len();
            if len == 0 {
                (0, Some("Empty content returned".to_string()))
            } else {
                (len, None)
            }
        }
        Err(e) => (0, Some(format!("{}", e))),
    }
}

#[tokio::test]
async fn test_load_example_com() {
    let (len, err) = load_site("https://example.com").await;
    assert!(err.is_none(), "example.com failed: {:?}", err);
    assert!(len > 100, "example.com content too short: {} bytes", len);
    eprintln!("example.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_httpbin_html() {
    let (len, err) = load_site("https://httpbin.org/html").await;
    assert!(err.is_none(), "httpbin.org/html failed: {:?}", err);
    assert!(len > 100, "httpbin content too short: {} bytes", len);
    eprintln!("httpbin.org/html: {} bytes", len);
}

#[tokio::test]
async fn test_load_wikipedia() {
    let (len, err) = load_site("https://en.wikipedia.org/wiki/Main_Page").await;
    assert!(err.is_none(), "wikipedia failed: {:?}", err);
    assert!(len > 1000, "wikipedia content too short: {} bytes", len);
    eprintln!("wikipedia: {} bytes", len);
}

#[tokio::test]
async fn test_load_github() {
    let (len, err) = load_site("https://github.com").await;
    assert!(err.is_none(), "github.com failed: {:?}", err);
    assert!(len > 1000, "github content too short: {} bytes", len);
    eprintln!("github.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_google() {
    let (len, err) = load_site("https://www.google.com").await;
    assert!(err.is_none(), "google.com failed: {:?}", err);
    assert!(len > 500, "google content too short: {} bytes", len);
    eprintln!("google.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_reddit() {
    let (len, err) = load_site("https://old.reddit.com").await;
    assert!(err.is_none(), "reddit failed: {:?}", err);
    assert!(len > 1000, "reddit content too short: {} bytes", len);
    eprintln!("old.reddit.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_hacker_news() {
    let (len, err) = load_site("https://news.ycombinator.com").await;
    assert!(err.is_none(), "hacker news failed: {:?}", err);
    assert!(len > 500, "HN content too short: {} bytes", len);
    eprintln!("news.ycombinator.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_mozilla_mdn() {
    let (len, err) = load_site("https://developer.mozilla.org/en-US/").await;
    assert!(err.is_none(), "MDN failed: {:?}", err);
    assert!(len > 1000, "MDN content too short: {} bytes", len);
    eprintln!("developer.mozilla.org: {} bytes", len);
}

#[tokio::test]
async fn test_load_duckduckgo() {
    let (len, err) = load_site("https://duckduckgo.com").await;
    assert!(err.is_none(), "duckduckgo failed: {:?}", err);
    assert!(len > 500, "DDG content too short: {} bytes", len);
    eprintln!("duckduckgo.com: {} bytes", len);
}

#[tokio::test]
async fn test_load_cnn() {
    let (len, err) = load_site("https://lite.cnn.com").await;
    assert!(err.is_none(), "CNN lite failed: {:?}", err);
    assert!(len > 500, "CNN content too short: {} bytes", len);
    eprintln!("lite.cnn.com: {} bytes", len);
}

// Test a site known to use CSP headers
#[tokio::test]
async fn test_load_site_with_csp_github() {
    // GitHub uses strict CSP — verify we don't crash on CSP parsing
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    let result = guard.navigate_to("https://github.com/about").await;
    assert!(result.is_ok(), "GitHub (CSP-heavy) failed: {:?}", result.err());
    let content = result.unwrap();
    assert!(content.len() > 1000, "GitHub about page too short");
    eprintln!("github.com/about (CSP): {} bytes", content.len());
}

// Test a site that uses module scripts (modern SPA)
#[tokio::test]
async fn test_load_site_with_modules_svelte() {
    let (len, err) = load_site("https://svelte.dev").await;
    assert!(err.is_none(), "svelte.dev failed: {:?}", err);
    assert!(len > 500, "svelte.dev content too short: {} bytes", len);
    eprintln!("svelte.dev: {} bytes", len);
}

// Test JS execution doesn't crash on loaded pages
#[tokio::test]
async fn test_js_execution_after_navigation() {
    let browser = HeadlessWebBrowser::new();
    let mut guard = browser.lock().unwrap();

    let nav_result = guard.navigate_to("https://example.com").await;
    assert!(nav_result.is_ok(), "Navigation failed: {:?}", nav_result.err());

    // Execute JS on the loaded page
    let js_result = guard.execute_javascript("document.title").await;
    assert!(js_result.is_ok(), "JS execution after nav failed: {:?}", js_result.err());
    eprintln!("example.com title: {}", js_result.unwrap());
}
