#[tokio::test]
async fn test_webgl_extensions() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");
    // Test getSupportedExtensions method
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const extensions = gl.getSupportedExtensions();
        typeof extensions === "object" && extensions.length === 5;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test getExtension method for debug renderer info
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
        typeof debugInfo === "object" &&
        debugInfo.UNMASKED_VENDOR_WEBGL === 37445 &&
        debugInfo.UNMASKED_RENDERER_WEBGL === 37446;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test getExtension method for lose context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const loseContext = gl.getExtension('WEBGL_lose_context');
        typeof loseContext === "object" &&
        typeof loseContext.loseContext === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test getExtension returns null for unsupported extensions
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const unsupported = gl.getExtension('UNSUPPORTED_EXTENSION');
        unsupported === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
