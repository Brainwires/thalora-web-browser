use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_javascript_safety_filtering() {
    let renderer = RustRenderer::new();

    // Safe JavaScript should pass
    let safe_js = "var x = 1; x + 2;";
    assert!(renderer.is_safe_javascript(safe_js));

    // Modern JavaScript features should now be allowed (standard-compliant)
    let modern_js_patterns = vec![
        "eval('1 + 1')",  // Now allowed - standard JS
        "Function('return 42')()",  // Now allowed - standard JS
        "XMLHttpRequest()",  // Now allowed - standard web API
        "fetch('https://api.example.com')",  // Now allowed - standard web API
        "document.cookie",  // Now allowed - standard DOM API
        "localStorage.setItem('key', 'value')",  // Now allowed - standard web API
        "window.location.href",  // Now allowed - standard DOM API
        "alert('popup')",  // Now allowed - standard web API
        "console.log('debug')",  // Now allowed - standard API
        "addEventListener('click', fn)",  // Now allowed - standard DOM API
        "setTimeout(() => {}, 1000)",  // Now allowed - standard timing API
        "document.getElementById('test')",  // Now allowed - standard DOM API
        "window.history.pushState({},'','/')",  // Now allowed - standard History API
    ];

    for pattern in modern_js_patterns {
        assert!(renderer.is_safe_javascript(pattern), "Pattern should be allowed: {}", pattern);
    }

    // Only truly dangerous Node.js/system patterns should be blocked
    let truly_dangerous_patterns = vec![
        "require('fs').readFileSync('/etc/passwd')",
        "require(\"child_process\").exec('rm -rf /')",
        "process.exit(1)",
        "global.process.kill(1)",
        "__dirname + '/sensitive'",
        "__filename",
        "Buffer.allocUnsafe(1000000)",
    ];

    for pattern in truly_dangerous_patterns {
        assert!(!renderer.is_safe_javascript(pattern), "Pattern should be blocked: {}", pattern);
    }

    // Size limit should still apply
    let too_large = "a".repeat(15_000_000); // Over 10MB limit
    assert!(!renderer.is_safe_javascript(&too_large));
}
