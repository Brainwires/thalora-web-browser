use thalora::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_service_worker_global_scope() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    polyfills.setup_all_apis(&mut context).unwrap();

    // Test ServiceWorkerGlobalScope
    let result = context.eval(Source::from_bytes(r#"
        // Test ServiceWorkerGlobalScope existence
        const hasServiceWorkerGlobalScope = typeof ServiceWorkerGlobalScope !== 'undefined';

        let scopeProperties = {};

        if (hasServiceWorkerGlobalScope) {
            // Create instance to test properties
            const scope = new ServiceWorkerGlobalScope();

            scopeProperties = {
                hasRegistration: scope.registration !== null,
                hasClients: typeof scope.clients === 'object',
                hasSkipWaiting: typeof scope.skipWaiting === 'function',
                hasClientsGet: typeof scope.clients.get === 'function',
                hasClientsMatchAll: typeof scope.clients.matchAll === 'function',
                hasClientsOpenWindow: typeof scope.clients.openWindow === 'function',
                hasClientsClaim: typeof scope.clients.claim === 'function'
            };
        }

        JSON.stringify({
            hasServiceWorkerGlobalScope,
            ...scopeProperties
        });
    "#));

    assert!(result.is_ok(), "ServiceWorkerGlobalScope test should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    assert_eq!(json_result["hasServiceWorkerGlobalScope"], true, "ServiceWorkerGlobalScope should be available");
    assert_eq!(json_result["hasRegistration"], true, "ServiceWorkerGlobalScope should have registration");
    assert_eq!(json_result["hasClients"], true, "ServiceWorkerGlobalScope should have clients");
    assert_eq!(json_result["hasSkipWaiting"], true, "ServiceWorkerGlobalScope should have skipWaiting");
    assert_eq!(json_result["hasClientsGet"], true, "clients.get should be available");
    assert_eq!(json_result["hasClientsMatchAll"], true, "clients.matchAll should be available");
    assert_eq!(json_result["hasClientsOpenWindow"], true, "clients.openWindow should be available");
    assert_eq!(json_result["hasClientsClaim"], true, "clients.claim should be available");

    println!("✅ ServiceWorkerGlobalScope is working correctly!");
}
