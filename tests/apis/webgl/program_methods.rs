#[tokio::test]
async fn test_webgl_program_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");
    // Test WebGL program methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createProgram === "function" &&
        typeof gl.attachShader === "function" &&
        typeof gl.linkProgram === "function" &&
        typeof gl.useProgram === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test program creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const program = gl.createProgram();
        typeof program === "object" && program !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
