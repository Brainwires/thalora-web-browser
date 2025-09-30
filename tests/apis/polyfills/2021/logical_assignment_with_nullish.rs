#[tokio::test]
async fn test_logical_assignment_with_nullish() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test logical assignment with nullish values
    let result = engine.execute_enhanced(r#"
        let x = null;
        let y = undefined;
        let z = 0;
        let w = '';
        x ??= 'null_replacement';
        y ??= 'undefined_replacement';
        z ??= 'zero_replacement';
        w ??= 'empty_replacement';
        [x, y, z, w]
    "#).await.unwrap();
    assert!(result.is_object());
}
