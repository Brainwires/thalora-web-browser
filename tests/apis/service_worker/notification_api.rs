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
