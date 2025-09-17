#[tokio::test]
async fn test_webgl_rendering_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");
    // Test WebGL rendering methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.viewport === "function" &&
        typeof gl.clearColor === "function" &&
        typeof gl.clear === "function" &&
        typeof gl.drawArrays === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
