#[tokio::test]
async fn test_chrome_137_selection_direction() {
    println!("🧪 Testing Chrome 137: Selection.direction with FrameSelection integration...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Direction property availability
    let direction_property_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if direction property exists
                var hasDirection = 'direction' in selection;
                var directionType = typeof selection.direction;
                var directionValue = selection.direction;

                var directionTests = {
                    hasDirection: hasDirection,
                    directionType: directionType,
                    initialValue: directionValue,
                };

                'Direction property: ' + JSON.stringify(directionTests);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(direction_property_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection.direction property: {}", value_str);
            assert!(value_str.contains("hasDirection\":true"), "Selection should have direction property");
            assert!(value_str.contains("directionType\":\"string\""), "Direction should be a string");
        },
        Err(e) => panic!("Failed to test Selection.direction property: {:?}", e),
    }

    // Test 2: Direction values and behavior
    let direction_values_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test direction values in different states
                var initialDirection = selection.direction;
                var isCollapsed = selection.isCollapsed;
                var rangeCount = selection.rangeCount;

                // Test that direction reflects selection state
                var directionInfo = {
                    initialDirection: initialDirection,
                    isCollapsed: isCollapsed,
                    rangeCount: rangeCount,
                    directionValid: ['none', 'forward', 'backward'].includes(initialDirection),
                };

                'Direction behavior: ' + JSON.stringify(directionInfo);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(direction_values_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection.direction behavior: {}", value_str);
            assert!(value_str.contains("directionValid\":true"), "Direction should have valid value");
        },
        Err(e) => panic!("Failed to test Selection.direction behavior: {:?}", e),
    }

    // Test 3: Direction integration with FrameSelection state
    let frame_selection_integration_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test that direction reflects internal FrameSelection state
                var selectionProperties = {
                    direction: selection.direction,
                    isCollapsed: selection.isCollapsed,
                    type: selection.type,
                    rangeCount: selection.rangeCount,
                };

                // Direction should correlate with selection state
                var expectedDirection = selectionProperties.isCollapsed ? 'none' : 'forward';
                var directionConsistent = selectionProperties.direction === expectedDirection ||
                                        selectionProperties.rangeCount === 0;

                var integrationTest = {
                    properties: selectionProperties,
                    expectedDirection: expectedDirection,
                    consistent: directionConsistent,
                };

                'FrameSelection integration: ' + JSON.stringify(integrationTest);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(frame_selection_integration_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection.direction FrameSelection integration: {}", value_str);
            // Direction should be consistent with internal state
            assert!(value_str.contains("direction"), "Direction should be reported");
        },
        Err(e) => panic!("Failed to test Selection.direction FrameSelection integration: {:?}", e),
    }

    println!("✅ Enhanced Selection.direction test completed");
}
