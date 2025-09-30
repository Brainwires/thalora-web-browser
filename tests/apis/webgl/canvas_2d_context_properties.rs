#[tokio::test]
async fn test_canvas_2d_context_properties() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");
    // Test 2D context properties
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        ctx.fillStyle === "#000000" &&
        ctx.strokeStyle === "#000000" &&
        ctx.lineWidth === 1 &&
        ctx.font === "10px sans-serif";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
