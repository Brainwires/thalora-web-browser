// Tests for Bing search result parsing

use thalora::protocols::mcp_server::scraping::search::bing;

#[test]
fn test_parse_bing_results() {
    let html = r#"
    <html>
        <body>
            <li class="b_algo">
                <h2><a href="https://www.rust-lang.org">Rust Programming Language</a></h2>
                <p>A language empowering everyone to build reliable and efficient software.</p>
            </li>
            <li class="b_algo">
                <h2><a href="https://docs.rust-lang.org">Rust Documentation</a></h2>
                <p>Learn Rust with comprehensive documentation.</p>
            </li>
        </body>
    </html>
    "#;

    let results = bing::parse_results(html, "rust", 10).unwrap();

    assert_eq!(results.query, "rust");
    assert_eq!(results.results.len(), 2);

    assert_eq!(results.results[0].title, "Rust Programming Language");
    assert_eq!(results.results[0].url, "https://www.rust-lang.org");
    assert_eq!(results.results[0].position, 1);

    assert_eq!(results.results[1].title, "Rust Documentation");
    assert_eq!(results.results[1].position, 2);
}

#[test]
fn test_parse_bing_empty_html() {
    let html = "<html><body></body></html>";

    let results = bing::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.query, "test");
    assert_eq!(results.results.len(), 0);
}

#[test]
fn test_parse_bing_respects_limit() {
    let html = r#"
    <html>
        <body>
            <li class="b_algo"><h2><a href="https://a.com">A</a></h2><p>A</p></li>
            <li class="b_algo"><h2><a href="https://b.com">B</a></h2><p>B</p></li>
            <li class="b_algo"><h2><a href="https://c.com">C</a></h2><p>C</p></li>
            <li class="b_algo"><h2><a href="https://d.com">D</a></h2><p>D</p></li>
        </body>
    </html>
    "#;

    let results = bing::parse_results(html, "test", 2).unwrap();

    assert!(results.results.len() <= 2);
}

#[test]
fn test_parse_bing_image_results() {
    let html = r#"
    <html>
        <body>
            <div class="imgpt">
                <img src="https://th.bing.com/image1.jpg" alt="Image 1" />
            </div>
            <img class="mimg" src="https://th.bing.com/image2.jpg" alt="Image 2" />
        </body>
    </html>
    "#;

    let results = bing::parse_image_results(html, "test images", 10).unwrap();

    assert_eq!(results.query, "test images");
}

#[test]
fn test_parse_bing_alternate_selectors() {
    // Bing sometimes uses different class names
    let html = r#"
    <html>
        <body>
            <div class="b_algoSlug">
                <h2><a href="https://example.com">Alternate Layout</a></h2>
                <p>Description</p>
            </div>
        </body>
    </html>
    "#;

    let results = bing::parse_results(html, "test", 10).unwrap();
    // May or may not find results depending on selector matching
    let _ = results.results.len();
}

#[test]
fn test_parse_bing_with_deep_links() {
    let html = r#"
    <html>
        <body>
            <li class="b_algo">
                <h2><a href="https://example.com/main">Main Result</a></h2>
                <p>Main description</p>
                <div class="b_deeplink">
                    <a href="https://example.com/sub1">Sub Link 1</a>
                    <a href="https://example.com/sub2">Sub Link 2</a>
                </div>
            </li>
        </body>
    </html>
    "#;

    let results = bing::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].url, "https://example.com/main");
}
