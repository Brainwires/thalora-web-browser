use thalora::engine::HeadlessWebBrowser;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webgl_buffer_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL buffer methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createBuffer === "function" &&
        typeof gl.bindBuffer === "function" &&
        typeof gl.bufferData === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test buffer creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const buffer = gl.createBuffer();
        typeof buffer === "object" && buffer !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
