#[tokio::test]
async fn test_webgl_constants() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");
    // Test WebGL constants are defined
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        gl.VERTEX_SHADER === 35633 &&
        gl.FRAGMENT_SHADER === 35632 &&
        gl.ARRAY_BUFFER === 34962 &&
        gl.STATIC_DRAW === 35044 &&
        gl.COLOR_BUFFER_BIT === 16384 &&
        gl.TRIANGLES === 4 &&
        gl.FLOAT === 5126;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
