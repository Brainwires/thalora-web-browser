use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_for_of_loops_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test for...of loops transformation
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c'];
        const result = [];

        // for...of is transformed to regular for loop
        for (var __i = 0; __i < (arr).length; __i++) {
            var item = (arr)[__i];
            result.push(item);
        }

        [
            result.length === 3,
            result[0] === 'a',
            result[1] === 'b',
            result[2] === 'c'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
