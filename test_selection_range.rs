use thalora::HeadlessWebBrowser;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Selection and Range API implementation...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Basic Selection API availability
    println!("\n1. Testing Selection API availability...");
    let selection_test = r#"
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

    let result = browser.lock().unwrap().execute_javascript(selection_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection API: {}", value_str);
        },
        Err(e) => {
            println!("❌ Selection API failed: {:?}", e);
        }
    }

    // Test 2: Range API availability
    println!("\n2. Testing Range API availability...");
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
            println!("✅ Range API: {}", value_str);
        },
        Err(e) => {
            println!("❌ Range API failed: {:?}", e);
        }
    }

    // Test 3: Selection-Range integration
    println!("\n3. Testing Selection-Range integration...");
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
        },
        Err(e) => {
            println!("❌ Selection-Range integration failed: {:?}", e);
        }
    }

    // Test 4: Chrome 137 compliance
    println!("\n4. Testing Chrome 137 compliance...");
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
        },
        Err(e) => {
            println!("❌ Chrome 137 compliance failed: {:?}", e);
        }
    }

    println!("\n🎉 Selection and Range API testing completed!");
}