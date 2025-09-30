#[tokio::test]
async fn test_service_worker_messaging() {
    let polyfills = WebApis::new();
    let mut context = Context::default();
    polyfills.setup_all_apis(&mut context).unwrap();
    // Test Service Worker messaging
    let result = context.eval(Source::from_bytes(r#"
        // Register Service Worker and test messaging
        navigator.serviceWorker.register('/messaging-sw.js').then(function(registration) {
            // Test postMessage to Service Worker
            if (registration.active && registration.active.postMessage) {
                registration.active.postMessage('Hello from main thread');
            }
            // Test global postMessage
            if (navigator.serviceWorker.postMessage) {
                navigator.serviceWorker.postMessage({ type: 'test', data: 'test data' });
            }
            return registration;
        });
        // Test message handler setup
        const hasOnMessage = navigator.serviceWorker.hasOwnProperty('onmessage');
        const hasPostMessage = typeof navigator.serviceWorker.postMessage === 'function';
        JSON.stringify({
            hasOnMessage,
            hasPostMessage,
            messagingSupported: hasOnMessage && hasPostMessage
        });
    "#));
    assert!(result.is_ok(), "Service Worker messaging test should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();
    assert_eq!(json_result["hasOnMessage"], true, "onmessage property should exist");
    assert_eq!(json_result["hasPostMessage"], true, "postMessage method should exist");
    assert_eq!(json_result["messagingSupported"], true, "Full messaging should be supported");
    println!("✅ Service Worker messaging is working correctly!");
}
