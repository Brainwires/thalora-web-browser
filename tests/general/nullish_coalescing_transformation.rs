use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_nullish_coalescing_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test nullish coalescing operator transformation
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = undefined;
        const c = 0;
        const d = false;
        const e = '';

        [
            a ?? 'default',
            b ?? 'default',
            c ?? 'default',
            d ?? 'default',
            e ?? 'default'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
