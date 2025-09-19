use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_selection_api_integration() {
    println!("🧪 Testing Selection API integration with Thalora...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Basic Selection API availability
    let basic_test = r#"
        try {
            // Test window.getSelection
            var hasGetSelection = typeof window !== 'undefined' && typeof window.getSelection === 'function';

            if (hasGetSelection) {
                var selection = window.getSelection();

                // Test Selection properties
                var tests = {
                    hasAnchorNode: 'anchorNode' in selection,
                    hasDirection: 'direction' in selection,
                    hasGetComposedRanges: typeof selection.getComposedRanges === 'function',
                    selectionDirection: selection.direction,
                    selectionType: selection.type,
                    rangeCount: selection.rangeCount,
                    isCollapsed: selection.isCollapsed,
                };

                'SUCCESS: ' + JSON.stringify(tests);
            } else {
                'FAIL: window.getSelection not available';
            }
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(basic_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection API integration: {}", value_str);

            assert!(value_str.contains("SUCCESS"), "Selection API should be functional");
            assert!(value_str.contains("hasAnchorNode\":true"), "Should have anchorNode");
            assert!(value_str.contains("hasDirection\":true"), "Should have direction");
            assert!(value_str.contains("hasGetComposedRanges\":true"), "Should have getComposedRanges");
        },
        Err(e) => {
            panic!("Failed to test Selection API integration: {:?}", e);
        }
    }

    println!("✅ Selection API integration test completed");
}

#[tokio::test]
async fn test_range_api_integration() {
    println!("🧪 Testing Range API integration with Thalora...");

    let browser = HeadlessWebBrowser::new();

    let range_test = r#"
        try {
            // Test Range constructor and methods
            var hasRange = typeof Range === 'function';

            if (hasRange) {
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
            } else {
                'FAIL: Range constructor not available';
            }
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(range_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Range API integration: {}", value_str);

            assert!(value_str.contains("SUCCESS"), "Range API should be functional");
            assert!(value_str.contains("hasSetStart\":true"), "Should have setStart");
            assert!(value_str.contains("hasCloneRange\":true"), "Should have cloneRange");
            assert!(value_str.contains("cloneWorks\":true"), "Range cloning should work");
            assert!(value_str.contains("START_TO_START\":0"), "Should have correct constants");
        },
        Err(e) => {
            panic!("Failed to test Range API integration: {:?}", e);
        }
    }

    println!("✅ Range API integration test completed");
}

#[tokio::test]
async fn test_selection_range_integration() {
    println!("🧪 Testing Selection-Range integration with Thalora...");

    let browser = HeadlessWebBrowser::new();

    let integration_test = r#"
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

            // Test Selection modify method (FrameSelection feature)
            selection.modify('move', 'forward', 'character');
            selection.modify('extend', 'backward', 'word');
            tests.modifyWorked = true;

            'SUCCESS: ' + JSON.stringify(tests);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(integration_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection-Range integration: {}", value_str);

            assert!(value_str.contains("SUCCESS"), "Selection-Range integration should work");
            assert!(value_str.contains("rangesIsArray\":true"), "getComposedRanges should return array");
            assert!(value_str.contains("modifyWorked\":true"), "Selection.modify should work");
        },
        Err(e) => {
            panic!("Failed to test Selection-Range integration: {:?}", e);
        }
    }

    println!("✅ Selection-Range integration test completed");
}

#[tokio::test]
async fn test_frame_selection_architecture_integration() {
    println!("🧪 Testing FrameSelection architecture integration with Thalora...");

    let browser = HeadlessWebBrowser::new();

    let frame_selection_test = r#"
        try {
            var selection = window.getSelection();

            // Test FrameSelection features
            var tests = {
                // Direction property (FrameSelection internal state)
                hasDirection: 'direction' in selection,
                directionType: typeof selection.direction,
                initialDirection: selection.direction,

                // Type property (FrameSelection state)
                hasType: 'type' in selection,
                typeType: typeof selection.type,
                initialType: selection.type,

                // Modify method (FrameSelection operation)
                hasModify: typeof selection.modify === 'function',

                // State consistency
                isCollapsed: selection.isCollapsed,
                rangeCount: selection.rangeCount,
            };

            // Test various granularities (FrameSelection feature)
            selection.modify('move', 'forward', 'character');
            selection.modify('move', 'forward', 'word');
            selection.modify('move', 'forward', 'line');
            selection.modify('move', 'forward', 'paragraph');
            tests.granularityTestPassed = true;

            // Test various directions (FrameSelection feature)
            selection.modify('move', 'left', 'character');
            selection.modify('move', 'right', 'character');
            selection.modify('move', 'backward', 'character');
            tests.directionTestPassed = true;

            // Test alter types (FrameSelection feature)
            selection.modify('move', 'forward', 'character');
            selection.modify('extend', 'forward', 'character');
            tests.alterTestPassed = true;

            'SUCCESS: ' + JSON.stringify(tests);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(frame_selection_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ FrameSelection architecture integration: {}", value_str);

            assert!(value_str.contains("SUCCESS"), "FrameSelection architecture should work");
            assert!(value_str.contains("hasDirection\":true"), "Should have direction property");
            assert!(value_str.contains("hasModify\":true"), "Should have modify method");
            assert!(value_str.contains("granularityTestPassed\":true"), "Granularity tests should pass");
            assert!(value_str.contains("directionTestPassed\":true"), "Direction tests should pass");
            assert!(value_str.contains("alterTestPassed\":true"), "Alter tests should pass");
        },
        Err(e) => {
            panic!("Failed to test FrameSelection architecture integration: {:?}", e);
        }
    }

    println!("✅ FrameSelection architecture integration test completed");
}

#[tokio::test]
async fn test_chrome_137_compliance() {
    println!("🧪 Testing Chrome 137 Selection API compliance...");

    let browser = HeadlessWebBrowser::new();

    let compliance_test = r#"
        try {
            var selection = window.getSelection();

            // Test Chrome 137 specific features
            var tests = {
                // Selection.direction (Chrome 137)
                hasDirection: 'direction' in selection,
                directionValid: ['none', 'forward', 'backward'].includes(selection.direction),

                // Selection.getComposedRanges (Chrome 137)
                hasGetComposedRanges: typeof selection.getComposedRanges === 'function',

                // Selection.modify granularity support (Chrome feature)
                hasModify: typeof selection.modify === 'function',

                // Range API compliance
                hasRange: typeof Range === 'function',
                rangeConstants: {
                    START_TO_START: Range.START_TO_START === 0,
                    START_TO_END: Range.START_TO_END === 1,
                    END_TO_END: Range.END_TO_END === 2,
                    END_TO_START: Range.END_TO_START === 3,
                }
            };

            // Test getComposedRanges functionality
            var ranges = selection.getComposedRanges();
            tests.getComposedRangesReturnsArray = Array.isArray(ranges);

            // Test getComposedRanges with shadow roots parameter
            var rangesWithShadow = selection.getComposedRanges([]);
            tests.getComposedRangesWithShadowWorks = Array.isArray(rangesWithShadow);

            // Test Range constructor
            var range = new Range();
            tests.rangeConstructorWorks = range instanceof Range;
            tests.rangeHasCollapsed = 'collapsed' in range;

            'SUCCESS: ' + JSON.stringify(tests);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(compliance_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Chrome 137 compliance: {}", value_str);

            assert!(value_str.contains("SUCCESS"), "Chrome 137 compliance should pass");
            assert!(value_str.contains("hasDirection\":true"), "Should have direction");
            assert!(value_str.contains("directionValid\":true"), "Direction should be valid");
            assert!(value_str.contains("hasGetComposedRanges\":true"), "Should have getComposedRanges");
            assert!(value_str.contains("getComposedRangesReturnsArray\":true"), "getComposedRanges should return array");
            assert!(value_str.contains("rangeConstructorWorks\":true"), "Range constructor should work");
        },
        Err(e) => {
            panic!("Failed to test Chrome 137 compliance: {:?}", e);
        }
    }

    println!("✅ Chrome 137 compliance test completed");
}