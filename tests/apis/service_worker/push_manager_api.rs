use thalora::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_push_manager_api() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    polyfills.setup_all_apis(&mut context).unwrap();

    // Test Push Manager API
    let result = context.eval(Source::from_bytes(r#"
        // Test PushManager existence
        const hasPushManager = typeof PushManager !== 'undefined';
        const hasServiceWorkerPushManager = navigator.serviceWorker && navigator.serviceWorker.pushManager;

        // Test PushManager methods
        let subscribeTest = false;
        let getSubscriptionTest = false;
        let permissionStateTest = false;

        if (hasServiceWorkerPushManager) {
            subscribeTest = typeof navigator.serviceWorker.pushManager.subscribe === 'function';
            getSubscriptionTest = typeof navigator.serviceWorker.pushManager.getSubscription === 'function';
            permissionStateTest = typeof navigator.serviceWorker.pushManager.permissionState === 'function';
        }

        JSON.stringify({
            hasPushManager,
            hasServiceWorkerPushManager: hasServiceWorkerPushManager !== null,
            subscribeTest,
            getSubscriptionTest,
            permissionStateTest
        });
    "#));

    assert!(result.is_ok(), "Push Manager test should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    assert_eq!(json_result["hasPushManager"], true, "PushManager should be available");
    assert_eq!(json_result["hasServiceWorkerPushManager"], true, "Service Worker should have pushManager");
    assert_eq!(json_result["subscribeTest"], true, "subscribe method should be available");
    assert_eq!(json_result["getSubscriptionTest"], true, "getSubscription method should be available");
    assert_eq!(json_result["permissionStateTest"], true, "permissionState method should be available");

    println!("✅ Push Manager API is working correctly!");
}
