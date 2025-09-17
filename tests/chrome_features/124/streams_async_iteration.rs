#[tokio::test]
async fn test_chrome_124_streams_async_iteration() {
    println!("🧪 Testing Chrome 124: Streams API Async Iteration...");

    let browser = HeadlessWebBrowser::new();

    // Test ReadableStream has Symbol.asyncIterator
    let result = browser.lock().unwrap().execute_javascript("typeof ReadableStream.prototype[Symbol.asyncIterator]").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ReadableStream async iterator type: {}", value_str);
            assert!(value_str.contains("function"), "ReadableStream should have async iterator method, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check ReadableStream async iterator: {:?}", e),
    }

    // Test that we can create a ReadableStream that's async iterable
    let js_code = r#"
        const stream = new ReadableStream({
            start(controller) {
                controller.enqueue("hello");
                controller.enqueue("world");
                controller.close();
            }
        });
        typeof stream[Symbol.asyncIterator]
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("ReadableStream instance async iterator: {:?}", value);
            assert!(format!("{:?}", value).contains("function"), "ReadableStream instance should have async iterator");
        },
        Err(e) => panic!("Failed to test ReadableStream async iteration: {:?}", e),
    }

    println!("✅ Streams async iteration test completed");
}
