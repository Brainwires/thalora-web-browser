#[tokio::test]
async fn test_object_has_own() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.hasOwn
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2 };
        const inherited = Object.create(obj);
        inherited.c = 3;
        [
            Object.hasOwn(obj, 'a'),
            Object.hasOwn(obj, 'toString'),
            Object.hasOwn(inherited, 'c'),
            Object.hasOwn(inherited, 'a')
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
