use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_javascript_safety_filtering() {
    let renderer = RustRenderer::new();
    
    // Safe JavaScript should pass
    let safe_js = "var x = 1; x + 2;";
    assert!(renderer.is_safe_javascript(safe_js));
    
    // Dangerous patterns should be blocked
    let dangerous_patterns = vec![
        "eval('malicious code')",
        "Function('return this')()",
        "XMLHttpRequest()",
        "fetch('http://evil.com')",
        "document.cookie",
        "localStorage.setItem('key', 'value')",
        "window.location = 'http://evil.com'",
        "alert('popup')",
    ];
    
    for pattern in dangerous_patterns {
        assert!(!renderer.is_safe_javascript(pattern), "Pattern should be blocked: {}", pattern);
    }
}
