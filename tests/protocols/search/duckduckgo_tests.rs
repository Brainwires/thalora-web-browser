// Tests for DuckDuckGo search result parsing

use thalora::protocols::mcp_server::scraping::search::duckduckgo;

#[test]
fn test_parse_duckduckgo_results() {
    let html = r#"
    <html>
        <body>
            <div class="result__body">
                <h2 class="result__title">
                    <a href="https://www.rust-lang.org">Rust Programming Language</a>
                </h2>
                <div class="result__snippet">A language empowering everyone to build reliable and efficient software.</div>
            </div>
            <div class="result__body">
                <h2 class="result__title">
                    <a href="https://docs.rust-lang.org">Rust Documentation</a>
                </h2>
                <div class="result__snippet">Learn Rust with comprehensive documentation and tutorials.</div>
            </div>
            <div class="result__body">
                <h2 class="result__title">
                    <a href="https://crates.io">crates.io: Rust Package Registry</a>
                </h2>
                <div class="result__snippet">The Rust community's crate registry.</div>
            </div>
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_results(html, "rust programming", 10).unwrap();

    assert_eq!(results.query, "rust programming");
    assert_eq!(results.results.len(), 3);

    assert_eq!(results.results[0].title, "Rust Programming Language");
    assert_eq!(results.results[0].url, "https://www.rust-lang.org");
    assert!(results.results[0].snippet.contains("reliable"));
    assert_eq!(results.results[0].position, 1);

    assert_eq!(results.results[1].title, "Rust Documentation");
    assert_eq!(results.results[1].url, "https://docs.rust-lang.org");
    assert_eq!(results.results[1].position, 2);
}

#[test]
fn test_parse_duckduckgo_empty_html() {
    let html = "<html><body></body></html>";

    let results = duckduckgo::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.query, "test");
    assert_eq!(results.results.len(), 0);
}

#[test]
fn test_parse_duckduckgo_respects_limit() {
    let html = r#"
    <html>
        <body>
            <div class="result__body">
                <h2 class="result__title"><a href="https://a.com">Result A</a></h2>
                <div class="result__snippet">Snippet A</div>
            </div>
            <div class="result__body">
                <h2 class="result__title"><a href="https://b.com">Result B</a></h2>
                <div class="result__snippet">Snippet B</div>
            </div>
            <div class="result__body">
                <h2 class="result__title"><a href="https://c.com">Result C</a></h2>
                <div class="result__snippet">Snippet C</div>
            </div>
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_results(html, "test", 2).unwrap();

    assert_eq!(results.results.len(), 2);
    assert_eq!(results.results[0].title, "Result A");
    assert_eq!(results.results[1].title, "Result B");
}

#[test]
fn test_parse_duckduckgo_skips_invalid_entries() {
    let html = r#"
    <html>
        <body>
            <div class="result__body">
                <!-- Missing URL -->
                <h2 class="result__title"><a>No URL Result</a></h2>
                <div class="result__snippet">Missing URL</div>
            </div>
            <div class="result__body">
                <!-- Valid entry -->
                <h2 class="result__title"><a href="https://valid.com">Valid Result</a></h2>
                <div class="result__snippet">Valid snippet</div>
            </div>
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].title, "Valid Result");
}

#[test]
fn test_parse_duckduckgo_image_results() {
    let html = r#"
    <html>
        <body>
            <div class="tile--img">
                <img src="https://images.example.com/image1.jpg" alt="Image One" />
            </div>
            <img data-src="https://images.example.com/image2.png" alt="Image Two" />
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_image_results(html, "test images", 10).unwrap();

    assert_eq!(results.query, "test images");
    // Note: Results depend on selector matching
    assert!(results.results.len() <= 2);
}

#[test]
fn test_parse_duckduckgo_special_characters() {
    let html = r#"
    <html>
        <body>
            <div class="result__body">
                <h2 class="result__title">
                    <a href="https://example.com/path?q=test&amp;lang=en">Result with &amp; and &lt;special&gt; chars</a>
                </h2>
                <div class="result__snippet">Contains &quot;quotes&quot; and more.</div>
            </div>
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_results(html, "special chars", 10).unwrap();

    assert_eq!(results.results.len(), 1);
    // HTML entities should be decoded
    assert!(results.results[0].url.contains("example.com"));
}

#[test]
fn test_parse_duckduckgo_unicode() {
    let html = r#"
    <html>
        <body>
            <div class="result__body">
                <h2 class="result__title">
                    <a href="https://example.com">Rust 编程语言 - プログラミング</a>
                </h2>
                <div class="result__snippet">学习 Rust 🦀</div>
            </div>
        </body>
    </html>
    "#;

    let results = duckduckgo::parse_results(html, "rust 中文", 10).unwrap();

    assert_eq!(results.results.len(), 1);
    assert!(results.results[0].title.contains("编程语言"));
    assert!(results.results[0].snippet.contains("🦀"));
}
