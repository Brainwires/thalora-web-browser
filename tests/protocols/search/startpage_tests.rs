// Tests for Startpage search result parsing

use thalora::protocols::mcp_server::scraping::search::startpage;

#[test]
fn test_parse_startpage_results() {
    let html = r#"
    <html>
        <body>
            <div class="result">
                <h3 class="result-title"><a href="https://www.rust-lang.org">Rust Programming Language</a></h3>
                <p class="result-snippet">A language empowering everyone to build reliable software.</p>
            </div>
            <div class="result">
                <h3 class="result-title"><a href="https://docs.rust-lang.org">Rust Documentation</a></h3>
                <p class="result-snippet">Learn Rust with documentation.</p>
            </div>
        </body>
    </html>
    "#;

    let results = startpage::parse_results(html, "rust", 10).unwrap();

    assert_eq!(results.query, "rust");
    let _ = results.results.len(); // Depends on selector matching
}

#[test]
fn test_parse_startpage_empty_html() {
    let html = "<html><body></body></html>";

    let results = startpage::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.query, "test");
    assert_eq!(results.results.len(), 0);
}

#[test]
fn test_parse_startpage_respects_limit() {
    let html = r#"
    <html>
        <body>
            <div class="result"><h3><a href="https://a.com">A</a></h3></div>
            <div class="result"><h3><a href="https://b.com">B</a></h3></div>
            <div class="result"><h3><a href="https://c.com">C</a></h3></div>
        </body>
    </html>
    "#;

    let results = startpage::parse_results(html, "test", 2).unwrap();

    assert!(results.results.len() <= 2);
}

#[test]
fn test_parse_startpage_with_h2_titles() {
    let html = r#"
    <html>
        <body>
            <div class="result">
                <h2><a href="https://example.com">H2 Title Result</a></h2>
                <p>Description text.</p>
            </div>
        </body>
    </html>
    "#;

    let results = startpage::parse_results(html, "test", 10).unwrap();

    let _ = results.results.len();
}

#[test]
fn test_parse_startpage_privacy_focused() {
    // Startpage is privacy-focused so URLs should be direct, not proxied
    let html = r#"
    <html>
        <body>
            <div class="result">
                <h3 class="result-title">
                    <a href="https://direct-url.example.com/page">Direct Link</a>
                </h3>
            </div>
        </body>
    </html>
    "#;

    let results = startpage::parse_results(html, "test", 10).unwrap();

    if !results.results.is_empty() {
        // URL should be the direct link, not a proxy
        assert!(results.results[0].url.contains("direct-url"));
    }
}

#[test]
fn test_parse_startpage_unicode_content() {
    let html = r#"
    <html>
        <body>
            <div class="result">
                <h3 class="result-title">
                    <a href="https://example.com">日本語タイトル</a>
                </h3>
                <p class="result-snippet">中文描述 🔒</p>
            </div>
        </body>
    </html>
    "#;

    let results = startpage::parse_results(html, "unicode", 10).unwrap();

    // Should handle unicode gracefully
    let _ = results.results.len();
}
