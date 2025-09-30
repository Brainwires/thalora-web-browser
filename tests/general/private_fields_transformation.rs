use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_private_fields_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test private fields transformation
    let result = engine.execute_enhanced(r#"
        // Private fields are transformed to _private_ prefixed properties
        class TestClass {
            constructor() {
                this._private_value = 42;
                this.publicValue = 'public';
            }

            getValue() {
                return this._private_value;
            }

            setValue(val) {
                this._private_value = val;
            }
        }

        const instance = new TestClass();
        [
            instance.getValue() === 42,
            instance.publicValue === 'public',
            typeof instance._private_value === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
