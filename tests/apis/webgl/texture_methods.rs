use thalora::engine::HeadlessWebBrowser;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webgl_texture_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL texture methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createTexture === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test texture creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const texture = gl.createTexture();
        typeof texture === "object" && texture !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
