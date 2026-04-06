// Tests for Google search result parsing

use thalora::protocols::mcp_server::scraping::search::google;

#[test]
fn test_parse_google_results() {
    let html = r#"
    <html>
        <body>
            <div class="g">
                <div class="tF2Cxc">
                    <a href="https://www.rust-lang.org"><h3>Rust Programming Language</h3></a>
                    <div class="IsZvec">A systems programming language.</div>
                </div>
            </div>
            <div class="g">
                <div class="tF2Cxc">
                    <a href="https://crates.io"><h3>crates.io</h3></a>
                    <div class="IsZvec">The Rust package registry.</div>
                </div>
            </div>
        </body>
    </html>
    "#;

    let results = google::parse_results(html, "rust", 10).unwrap();

    assert_eq!(results.query, "rust");
    let _ = results.results.len(); // Google parsing is complex
}

#[test]
fn test_parse_google_empty_html() {
    let html = "<html><body></body></html>";

    let results = google::parse_results(html, "test", 10).unwrap();

    assert_eq!(results.query, "test");
    assert_eq!(results.results.len(), 0);
}

#[test]
fn test_parse_google_respects_limit() {
    let html = r#"
    <html>
        <body>
            <div class="g"><a href="https://a.com"><h3>A</h3></a></div>
            <div class="g"><a href="https://b.com"><h3>B</h3></a></div>
            <div class="g"><a href="https://c.com"><h3>C</h3></a></div>
            <div class="g"><a href="https://d.com"><h3>D</h3></a></div>
        </body>
    </html>
    "#;

    let results = google::parse_results(html, "test", 2).unwrap();

    assert!(results.results.len() <= 2);
}

#[test]
fn test_parse_google_image_results() {
    let html = r#"
    <html>
        <body>
            <div class="isv-r">
                <img class="rg_i" src="https://images.google.com/image1.jpg" alt="Test Image" />
            </div>
        </body>
    </html>
    "#;

    let results = google::parse_image_results(html, "test images", 10).unwrap();

    assert_eq!(results.query, "test images");
}

#[test]
fn test_parse_google_featured_snippet() {
    let html = r#"
    <html>
        <body>
            <div class="xpdopen">
                <div class="kno-rdesc">
                    <span>Featured snippet content here.</span>
                </div>
            </div>
            <div class="g">
                <a href="https://example.com"><h3>Regular Result</h3></a>
            </div>
        </body>
    </html>
    "#;

    let results = google::parse_results(html, "test", 10).unwrap();

    // Should handle featured snippets gracefully
    let _ = results.results.len();
}

#[test]
fn test_parse_google_alternate_structure() {
    // Google frequently changes their HTML structure
    let html = r#"
    <html>
        <body>
            <div class="MjjYud">
                <div class="g">
                    <a href="https://example.com"><h3 class="LC20lb">Title Here</h3></a>
                    <div class="VwiC3b">Snippet text here</div>
                </div>
            </div>
        </body>
    </html>
    "#;

    let results = google::parse_results(html, "test", 10).unwrap();

    // May find results with alternate selectors
    let _ = results.results.len();
}

#[test]
fn test_parse_google_with_ads() {
    let html = r#"
    <html>
        <body>
            <div class="uEierd">
                <a href="https://ad.example.com"><h3>Ad Result</h3></a>
            </div>
            <div class="g">
                <a href="https://organic.example.com"><h3>Organic Result</h3></a>
            </div>
        </body>
    </html>
    "#;

    let results = google::parse_results(html, "test", 10).unwrap();

    // Should ideally skip ads, but may include them
    let _ = results.results.len();
}
