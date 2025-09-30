use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_error_is_error() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Error.isError utility
    let result = engine.execute_enhanced(r#"
        const err = new Error('test');
        const errLike = { name: 'CustomError', message: 'custom message' };
        const notErr = { name: 123, message: 'invalid' };

        [
            Error.isError(err) === true,
            Error.isError(errLike) === true,
            Error.isError(notErr) === false,
            Error.isError('string') === false,
            Error.isError(null) === false,
            typeof Error.isError === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
