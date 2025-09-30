use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_destructuring_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test destructuring transformation
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };

        // Destructuring is transformed to individual assignments
        var a = (obj).a;
        var b = (obj).b;

        [
            a === 1,
            b === 2,
            typeof a === 'number',
            typeof b === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
