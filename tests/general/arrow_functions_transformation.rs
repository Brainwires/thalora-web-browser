use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_arrow_functions_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test arrow functions transformation to regular functions
    let result = engine.execute_enhanced(r#"
        var add = function(a, b) { return a + b; };
        var multiply = function(x, y) { return x * y; };
        var greet = function(name) { return 'Hello, ' + name; };

        [
            add(2, 3) === 5,
            multiply(4, 5) === 20,
            greet('World') === 'Hello, World',
            typeof add === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
