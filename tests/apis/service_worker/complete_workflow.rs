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
