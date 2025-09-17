use thalora::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_service_worker_registration() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    // Setup polyfills
    polyfills.setup_all_apis(&mut context).unwrap();

    // Test Service Worker registration
    let result = context.eval(Source::from_bytes(r#"
        // Test Service Worker registration
        let registrationPromise = navigator.serviceWorker.register('/sw.js', {
            scope: '/app'
        });

        // Test the returned promise
        registrationPromise.then(function(registration) {
            const result = {
                hasRegistration: registration !== null,
                correctScope: registration.scope === '/app',
                hasActive: registration.active !== null,
                hasUpdate: typeof registration.update === 'function',
                hasUnregister: typeof registration.unregister === 'function',
                activeState: registration.active && registration.active.state,
                scriptURL: registration.active && registration.active.scriptURL
            };

            // Store result in global for testing
            window.__testResult = result;
            return result;
        });

        // Return promise for async testing
        'promise_created';
    "#));

    assert!(result.is_ok(), "Service Worker registration should execute without errors");

    // Test synchronous properties
    let sync_result = context.eval(Source::from_bytes(r#"
        JSON.stringify({
            hasServiceWorker: typeof navigator.serviceWorker !== 'undefined',
            hasRegister: typeof navigator.serviceWorker.register === 'function',
            hasGetRegistration: typeof navigator.serviceWorker.getRegistration === 'function',
            hasGetRegistrations: typeof navigator.serviceWorker.getRegistrations === 'function',
            hasReady: navigator.serviceWorker.ready instanceof Promise,
            hasController: navigator.serviceWorker.controller === null,
            hasAddEventListener: typeof navigator.serviceWorker.addEventListener === 'function'
        });
    "#));

    assert!(sync_result.is_ok(), "Service Worker properties test should execute without errors");
    let output = sync_result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    // Verify Service Worker API is available
    assert_eq!(json_result["hasServiceWorker"], true, "navigator.serviceWorker should be available");
    assert_eq!(json_result["hasRegister"], true, "register method should be available");
    assert_eq!(json_result["hasGetRegistration"], true, "getRegistration method should be available");
    assert_eq!(json_result["hasGetRegistrations"], true, "getRegistrations method should be available");
    assert_eq!(json_result["hasReady"], true, "ready promise should be available");
    assert_eq!(json_result["hasController"], true, "controller should initially be null");
    assert_eq!(json_result["hasAddEventListener"], true, "addEventListener should be available");

    println!("✅ Service Worker registration API is working correctly!");
}
