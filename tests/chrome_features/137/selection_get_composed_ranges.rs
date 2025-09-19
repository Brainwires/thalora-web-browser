#[tokio::test]
async fn test_chrome_137_selection_get_composed_ranges() {
    println!("🧪 Testing Chrome 137: Selection.getComposedRanges() with Shadow DOM support...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Basic getComposedRanges availability
    let availability_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if getComposedRanges method exists
                var hasGetComposedRanges = typeof selection.getComposedRanges === 'function';

                if (hasGetComposedRanges) {
                    'Selection.getComposedRanges method available: true';
                } else {
                    'Selection.getComposedRanges method not available';
                }
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(availability_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ getComposedRanges availability: {}", value_str);
            assert!(value_str.contains("available: true"), "getComposedRanges should be available");
        },
        Err(e) => panic!("Failed to test getComposedRanges availability: {:?}", e),
    }

    // Test 2: getComposedRanges functionality
    let functionality_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                if (typeof selection.getComposedRanges === 'function') {
                    // Test calling getComposedRanges without arguments
                    var ranges1 = selection.getComposedRanges();
                    var isArray1 = Array.isArray(ranges1);
                    var length1 = ranges1.length;

                    // Test calling getComposedRanges with empty array
                    var ranges2 = selection.getComposedRanges([]);
                    var isArray2 = Array.isArray(ranges2);
                    var length2 = ranges2.length;

                    // Test return type
                    var results = {
                        noArgs: { isArray: isArray1, length: length1 },
                        emptyArray: { isArray: isArray2, length: length2 },
                    };

                    'getComposedRanges results: ' + JSON.stringify(results);
                } else {
                    'getComposedRanges method not available';
                }
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(functionality_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ getComposedRanges functionality: {}", value_str);
            assert!(value_str.contains("isArray\":true"), "getComposedRanges should return an array");
        },
        Err(e) => panic!("Failed to test getComposedRanges functionality: {:?}", e),
    }

    // Test 3: Range object integration
    let range_integration_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection && typeof Range === 'function') {
                var selection = window.getSelection();

                if (typeof selection.getComposedRanges === 'function') {
                    var ranges = selection.getComposedRanges();

                    // Test if returned objects are Range instances (when selection exists)
                    var rangeTests = {
                        hasRanges: ranges.length >= 0,
                        arrayType: typeof ranges,
                        isArray: Array.isArray(ranges),
                    };

                    // If we have ranges, test their properties
                    if (ranges.length > 0) {
                        var firstRange = ranges[0];
                        rangeTests.firstRangeType = typeof firstRange;
                        rangeTests.hasStartContainer = 'startContainer' in firstRange;
                        rangeTests.hasCollapsed = 'collapsed' in firstRange;
                    }

                    'Range integration: ' + JSON.stringify(rangeTests);
                } else {
                    'getComposedRanges method not available';
                }
            } else {
                'Prerequisites not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(range_integration_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ getComposedRanges Range integration: {}", value_str);
            assert!(value_str.contains("isArray\":true"), "getComposedRanges should integrate with Range objects");
        },
        Err(e) => panic!("Failed to test getComposedRanges Range integration: {:?}", e),
    }

    println!("✅ Enhanced getComposedRanges test completed");
}
