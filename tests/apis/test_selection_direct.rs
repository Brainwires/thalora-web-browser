use std::sync::{Arc, Mutex};
use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_selection_api_direct() {
    println!("🧪 Testing Selection API directly...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Basic Selection API availability
    let test_code = r#"
        try {
            // Test window.getSelection
            var hasGetSelection = typeof window !== 'undefined' && typeof window.getSelection === 'function';

            if (hasGetSelection) {
                var selection = window.getSelection();

                // Test Selection properties
                var hasAnchorNode = 'anchorNode' in selection;
                var hasDirection = 'direction' in selection;
                var hasGetComposedRanges = typeof selection.getComposedRanges === 'function';

                // Test Range constructor
                var hasRange = typeof Range === 'function';
                var range = hasRange ? new Range() : null;
                var rangeHasCollapsed = range ? 'collapsed' in range : false;

                var results = {
                    hasGetSelection: hasGetSelection,
                    hasAnchorNode: hasAnchorNode,
                    hasDirection: hasDirection,
                    hasGetComposedRanges: hasGetComposedRanges,
                    hasRange: hasRange,
                    rangeHasCollapsed: rangeHasCollapsed,
                    selectionDirection: selection.direction,
                    selectionType: selection.type,
                    rangeCount: selection.rangeCount,
                };

                'SUCCESS: ' + JSON.stringify(results);
            } else {
                'FAIL: window.getSelection not available';
            }
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(test_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection API test result: {}", value_str);

            // Check that our implementation is working
            assert!(value_str.contains("SUCCESS"), "Selection API should be functional");
            assert!(value_str.contains("hasGetSelection\":true"), "Should have getSelection");
            assert!(value_str.contains("hasAnchorNode\":true"), "Should have anchorNode");
            assert!(value_str.contains("hasDirection\":true"), "Should have direction");
            assert!(value_str.contains("hasGetComposedRanges\":true"), "Should have getComposedRanges");
            assert!(value_str.contains("hasRange\":true"), "Should have Range constructor");
            assert!(value_str.contains("rangeHasCollapsed\":true"), "Range should have collapsed property");
        },
        Err(e) => {
            panic!("Failed to test Selection API directly: {:?}", e);
        }
    }

    println!("✅ Direct Selection API test completed successfully");
}

#[tokio::test]
async fn test_range_api_direct() {
    println!("🧪 Testing Range API directly...");

    let browser = HeadlessWebBrowser::new();

    let test_code = r#"
        try {
            // Test Range constructor and methods
            var range = new Range();

            var tests = {
                hasSetStart: typeof range.setStart === 'function',
                hasSetEnd: typeof range.setEnd === 'function',
                hasCollapse: typeof range.collapse === 'function',
                hasSelectNode: typeof range.selectNode === 'function',
                hasCloneRange: typeof range.cloneRange === 'function',
                hasToString: typeof range.toString === 'function',
                initialCollapsed: range.collapsed,
                constants: {
                    START_TO_START: Range.START_TO_START,
                    START_TO_END: Range.START_TO_END,
                    END_TO_END: Range.END_TO_END,
                    END_TO_START: Range.END_TO_START,
                }
            };

            // Test cloning
            var cloned = range.cloneRange();
            tests.cloneWorks = cloned instanceof Range;
            tests.cloneCollapsed = cloned.collapsed;

            // Test toString
            var str = range.toString();
            tests.toStringResult = typeof str === 'string' ? 'string' : typeof str;

            'SUCCESS: ' + JSON.stringify(tests);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(test_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Range API test result: {}", value_str);

            // Check that our Range implementation is working
            assert!(value_str.contains("SUCCESS"), "Range API should be functional");
            assert!(value_str.contains("hasSetStart\":true"), "Should have setStart");
            assert!(value_str.contains("hasCloneRange\":true"), "Should have cloneRange");
            assert!(value_str.contains("cloneWorks\":true"), "Range cloning should work");
            assert!(value_str.contains("START_TO_START\":0"), "Should have correct constants");
        },
        Err(e) => {
            panic!("Failed to test Range API directly: {:?}", e);
        }
    }

    println!("✅ Direct Range API test completed successfully");
}

#[tokio::test]
async fn test_selection_range_integration() {
    println!("🧪 Testing Selection-Range integration...");

    let browser = HeadlessWebBrowser::new();

    let test_code = r#"
        try {
            var selection = window.getSelection();

            // Test getComposedRanges returns proper Range objects
            var ranges = selection.getComposedRanges();

            var tests = {
                rangesIsArray: Array.isArray(ranges),
                rangesLength: ranges.length,
                rangesType: typeof ranges,
            };

            // Test with empty array parameter
            var rangesWithParam = selection.getComposedRanges([]);
            tests.withParamIsArray = Array.isArray(rangesWithParam);
            tests.withParamLength = rangesWithParam.length;

            // Test that ranges are proper Range objects when they exist
            if (ranges.length > 0) {
                tests.firstRangeIsRange = ranges[0] instanceof Range;
                tests.firstRangeHasCollapsed = 'collapsed' in ranges[0];
            }

            'SUCCESS: ' + JSON.stringify(tests);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(test_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection-Range integration test result: {}", value_str);

            // Check that Selection and Range integration works
            assert!(value_str.contains("SUCCESS"), "Selection-Range integration should work");
            assert!(value_str.contains("rangesIsArray\":true"), "getComposedRanges should return array");
        },
        Err(e) => {
            panic!("Failed to test Selection-Range integration: {:?}", e);
        }
    }

    println!("✅ Selection-Range integration test completed successfully");
}