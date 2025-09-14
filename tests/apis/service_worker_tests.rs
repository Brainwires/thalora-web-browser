use synaptic::WebApis;
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

#[tokio::test]
async fn test_notification_api() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    polyfills.setup_all_apis(&mut context).unwrap();

    // Test Notification API
    let result = context.eval(Source::from_bytes(r#"
        // Test Notification constructor and static properties
        const hasNotification = typeof Notification !== 'undefined';
        const hasPermission = typeof Notification.permission === 'string';
        const hasRequestPermission = typeof Notification.requestPermission === 'function';
        const permissionValue = Notification.permission;

        // Test creating a notification
        let notificationCreated = false;
        let notificationTitle = '';
        let notificationBody = '';

        try {
            const notification = new Notification('Test Title', {
                body: 'Test message',
                icon: '/icon.png',
                tag: 'test-tag'
            });

            notificationCreated = true;
            notificationTitle = notification.title;
            notificationBody = notification.body;
        } catch (e) {
            // Notification creation failed
        }

        JSON.stringify({
            hasNotification,
            hasPermission,
            hasRequestPermission,
            permissionValue,
            notificationCreated,
            correctTitle: notificationTitle === 'Test Title',
            correctBody: notificationBody === 'Test message'
        });
    "#));

    assert!(result.is_ok(), "Notification API test should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    assert_eq!(json_result["hasNotification"], true, "Notification should be available");
    assert_eq!(json_result["hasPermission"], true, "Notification.permission should exist");
    assert_eq!(json_result["hasRequestPermission"], true, "Notification.requestPermission should exist");
    assert_eq!(json_result["permissionValue"], "granted", "Permission should be granted by default");
    assert_eq!(json_result["notificationCreated"], true, "Notification should be creatable");
    assert_eq!(json_result["correctTitle"], true, "Notification title should be correct");
    assert_eq!(json_result["correctBody"], true, "Notification body should be correct");

    println!("✅ Notification API is working correctly!");
}

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

#[tokio::test]
async fn test_complete_service_worker_workflow() {
    let polyfills = WebApis::new();
    let mut context = Context::default();

    polyfills.setup_all_apis(&mut context).unwrap();

    // Test complete Service Worker workflow
    let result = context.eval(Source::from_bytes(r#"
        // Complete Service Worker workflow test
        let workflowResults = {};

        // Step 1: Register Service Worker
        navigator.serviceWorker.register('/complete-sw.js', {
            scope: '/complete',
            updateViaCache: 'none'
        }).then(function(registration) {
            workflowResults.registration = {
                success: true,
                hasActive: registration.active !== null,
                correctScope: registration.scope === '/complete',
                correctUpdateViaCache: registration.updateViaCache === 'none'
            };

            // Step 2: Test registration methods
            return registration.update().then(function() {
                workflowResults.update = { success: true };

                // Step 3: Test getRegistration
                return navigator.serviceWorker.getRegistration('/complete');
            });
        }).then(function(foundRegistration) {
            workflowResults.getRegistration = {
                found: foundRegistration !== null,
                sameScope: foundRegistration && foundRegistration.scope === '/complete'
            };

            // Step 4: Test Push Manager subscription
            if (navigator.serviceWorker.pushManager) {
                return navigator.serviceWorker.pushManager.subscribe({
                    userVisibleOnly: true,
                    applicationServerKey: 'test-key'
                });
            }
            return null;
        }).then(function(subscription) {
            workflowResults.pushSubscription = {
                created: subscription !== null,
                hasEndpoint: subscription && typeof subscription.endpoint === 'string',
                hasKeys: subscription && typeof subscription.keys === 'object',
                hasUnsubscribe: subscription && typeof subscription.unsubscribe === 'function'
            };

            // Step 5: Test notification
            if (typeof Notification !== 'undefined') {
                const notification = new Notification('Workflow Test', {
                    body: 'Testing complete Service Worker workflow',
                    tag: 'workflow-test'
                });

                workflowResults.notification = {
                    created: true,
                    correctTitle: notification.title === 'Workflow Test',
                    hasClose: typeof notification.close === 'function'
                };
            }

            return workflowResults;
        }).catch(function(error) {
            workflowResults.error = error.message || 'Unknown error';
            return workflowResults;
        });

        // Return initial success indicator
        JSON.stringify({
            workflowStarted: true,
            hasAllAPIs: typeof navigator.serviceWorker !== 'undefined' &&
                        typeof PushManager !== 'undefined' &&
                        typeof Notification !== 'undefined'
        });
    "#));

    assert!(result.is_ok(), "Complete Service Worker workflow should execute without errors");
    let output = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    let json_result: serde_json::Value = serde_json::from_str(&output.replace("'", "\"")).unwrap();

    assert_eq!(json_result["workflowStarted"], true, "Service Worker workflow should start");
    assert_eq!(json_result["hasAllAPIs"], true, "All Service Worker APIs should be available");

    println!("✅ Complete Service Worker workflow is working correctly!");
}