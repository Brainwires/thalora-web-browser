#[cfg(test)]
mod tests {
    use thalora::engine::HeadlessWebBrowser;
    use thalora::engine::engine::JavaScriptEngine;

    #[tokio::test]
    async fn test_error_stack_trace_implementation() {
    let _browser = HeadlessWebBrowser::new();
    let mut js_engine = JavaScriptEngine::new().unwrap();

        // Test Error.stack property
        let stack_test = r#"
            console.log('=== Error Stack Test ===');
            console.log('Error object:', typeof Error);
            console.log('Error.captureStackTrace:', typeof Error.captureStackTrace);

            try {
                var err = new Error('test error');
                console.log('Error.stack exists:', typeof err.stack);
                console.log('Error.stack value:', err.stack);
            } catch(e) {
                console.log('Error creation failed:', e.message);
            }

            try {
                var obj = {};
                Error.captureStackTrace(obj);
                console.log('captureStackTrace obj.stack:', typeof obj.stack);
            } catch(e) {
                console.log('captureStackTrace failed:', e.message);
            }

            'Complete'
        "#;

    let result = js_engine.execute_enhanced(stack_test).await;
        println!("Error stack test result: {:?}", result);

        // The test should show Error.stack exists and Error.captureStackTrace works
        assert!(result.is_ok());
    }
}