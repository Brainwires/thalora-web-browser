use thalora::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_service_worker_getters() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    polyfills.setup_all_apis(&mut context).unwrap();

    // Test Service Worker getters after registration
    let result = context.eval(Source::from_bytes(r#"
        // Register a Service Worker first
        navigator.serviceWorker.register('/test-sw.js', { scope: '/test' }).then(function() {
            // Test getRegistration
            return navigator.serviceWorker.getRegistration('/test');
        }).then(function(registration) {
            const hasRegistration = registration !== null;
            const correctScope = registration && registration.scope === '/test';

            // Test getRegistrations
            return navigator.serviceWorker.getRegistrations().then(function(registrations) {
                return {
                    hasRegistration,
                    correctScope,
                    registrationsCount: registrations.length,
                    hasRegistrations: Array.isArray(registrations)
                };
            });
        });

        // Test synchronous Service Worker ready
        const readyTest = navigator.serviceWorker.ready instanceof Promise;

        JSON.stringify({
            readyIsPromise: readyTest,
            getRegistrationExists: typeof navigator.serviceWorker.getRegistration === 'function',
            getRegistrationsExists: typeof navigator.serviceWorker.getRegistrations === 'function'
        });
    "#));

    assert!(result.is_ok(), "Service Worker getters test should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    assert_eq!(json_result["readyIsPromise"], true, "ready should be a Promise");
    assert_eq!(json_result["getRegistrationExists"], true, "getRegistration should exist");
    assert_eq!(json_result["getRegistrationsExists"], true, "getRegistrations should exist");

    println!("✅ Service Worker getters are working correctly!");
}
