use thalora::engine::HeadlessWebBrowser;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_canvas_2d_context_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test 2D context drawing methods exist
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        typeof ctx.fillRect === "function" &&
        typeof ctx.fillText === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
