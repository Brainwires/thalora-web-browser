use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_complex_object_operations() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test complex object operations
    let result = engine.execute_enhanced(r#"
        const obj = {
            a: 1,
            b: {
                c: 2,
                d: [3, 4, 5]
            },
            method: function() {
                return this.a + this.b.c;
            }
        };

        [
            obj.a === 1,
            obj.b.c === 2,
            obj.b.d.length === 3,
            obj.method() === 3
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
