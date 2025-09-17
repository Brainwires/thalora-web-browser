#[tokio::test]
async fn test_nullish_coalescing() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test nullish coalescing operator (??)
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = undefined;
        const c = 0;
        const d = '';
        [
            a ?? 'default',
            b ?? 'default',
            c ?? 'default',
            d ?? 'default'
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
