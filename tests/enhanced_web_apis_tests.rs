use synaptic::renderer::RustRenderer;
use anyhow::Result;

#[tokio::test]
async fn test_enhanced_web_apis_available() -> Result<()> {
    let mut renderer = RustRenderer::new();
    renderer.setup_enhanced_web_apis()?;
    
    // Setup enhanced web APIs
    renderer.setup_enhanced_web_apis()?;
    
    // Test that enhanced APIs are available after setup
    let result = renderer.evaluate_javascript("typeof window.fetch");
    assert!(result.is_ok(), "Enhanced web APIs should be available");
    let fetch_type = result?;
    assert_eq!(fetch_type, "function", "window.fetch should be a function");
    
    Ok(())
}

#[tokio::test]
async fn test_fetch_api_functionality() -> Result<()> {
    let mut renderer = RustRenderer::new();
    renderer.setup_enhanced_web_apis()?;
    
    // Test that fetch API returns a Promise-like object
    let result = renderer.evaluate_javascript(r#"
        var fetchResult = window.fetch('https://example.com');
        typeof fetchResult;
    "#);
    
    assert!(result.is_ok(), "Fetch should execute without error");
    let result_type = result?;
    assert_eq!(result_type, "object", "fetch should return an object (Promise)");
    
    Ok(())
}

#[tokio::test]
async fn test_url_api_functionality() -> Result<()> {
    let mut renderer = RustRenderer::new();
    renderer.setup_enhanced_web_apis()?;
    
    // Test URL constructor
    let result = renderer.evaluate_javascript(r#"
        try {
            var url = new URL('https://example.com/path');
            url.href;
        } catch(e) {
            'error: ' + e.message;
        }
    "#);
    
    assert!(result.is_ok(), "URL constructor should work");
    let url_result = result?;
    assert!(url_result.contains("https://example.com"), "URL should parse correctly");
    
    Ok(())
}

#[tokio::test]
async fn test_crypto_api_functionality() -> Result<()> {
    let mut renderer = RustRenderer::new();
    renderer.setup_enhanced_web_apis()?;
    
    // Test crypto.randomUUID
    let result = renderer.evaluate_javascript(r#"
        try {
            var uuid = window.crypto.randomUUID();
            typeof uuid;
        } catch(e) {
            'error';
        }
    "#);
    
    assert!(result.is_ok(), "Crypto API should work");
    let result_type = result?;
    assert_eq!(result_type, "string", "randomUUID should return a string");
    
    Ok(())
}

#[tokio::test]
async fn test_service_worker_api_functionality() -> Result<()> {
    let mut renderer = RustRenderer::new();
    renderer.setup_enhanced_web_apis()?;
    
    // Test Service Worker API
    let result = renderer.evaluate_javascript(r#"
        typeof navigator.serviceWorker;
    "#);
    
    assert!(result.is_ok(), "Service Worker API should be available");
    let sw_type = result?;
    assert_eq!(sw_type, "object", "navigator.serviceWorker should be an object");
    
    Ok(())
}