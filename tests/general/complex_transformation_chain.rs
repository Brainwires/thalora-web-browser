use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_complex_transformation_chain() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test multiple transformations working together
    let result = engine.execute_enhanced(r#"
        // Multiple ES features combined
        var data = null;
        data ??= { values: [1_000, 2_000, 3_000] };

        var sum = 0;
        for (var __i = 0; __i < (data.values).length; __i++) {
            var value = (data.values)[__i];
            sum = sum || 0;
            sum += value;
        }

        var result = data?.values?.length === 3 && sum === 6000;
        result
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
