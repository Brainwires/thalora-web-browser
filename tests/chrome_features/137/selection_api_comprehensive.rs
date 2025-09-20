#[tokio::test]
#[ignore]
async fn test_chrome_137_selection_api_comprehensive() {
    println!("🧪 Testing Chrome 137: Comprehensive Selection API...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Basic Selection API availability
    let basic_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test all basic properties
                var tests = {
                    hasAnchorNode: 'anchorNode' in selection,
                    hasAnchorOffset: 'anchorOffset' in selection,
                    hasFocusNode: 'focusNode' in selection,
                    hasFocusOffset: 'focusOffset' in selection,
                    hasIsCollapsed: 'isCollapsed' in selection,
                    hasRangeCount: 'rangeCount' in selection,
                    hasType: 'type' in selection,
                    hasDirection: 'direction' in selection,
                };

                'Basic properties: ' + JSON.stringify(tests);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(basic_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Basic Selection API properties: {}", value_str);
            assert!(value_str.contains("true"), "Basic Selection API properties should be available");
        },
        Err(e) => panic!("Failed to test basic Selection API: {:?}", e),
    }

    // Test 2: Selection API methods
    let methods_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                var methods = {
                    hasAddRange: typeof selection.addRange === 'function',
                    hasRemoveAllRanges: typeof selection.removeAllRanges === 'function',
                    hasGetRangeAt: typeof selection.getRangeAt === 'function',
                    hasGetComposedRanges: typeof selection.getComposedRanges === 'function',
                    hasSetBaseAndExtent: typeof selection.setBaseAndExtent === 'function',
                    hasCollapse: typeof selection.collapse === 'function',
                    hasCollapseToStart: typeof selection.collapseToStart === 'function',
                    hasCollapseToEnd: typeof selection.collapseToEnd === 'function',
                    hasExtend: typeof selection.extend === 'function',
                    hasSelectAllChildren: typeof selection.selectAllChildren === 'function',
                    hasDeleteFromDocument: typeof selection.deleteFromDocument === 'function',
                    hasContainsNode: typeof selection.containsNode === 'function',
                    hasModify: typeof selection.modify === 'function',
                };

                'Methods availability: ' + JSON.stringify(methods);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(methods_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection API methods: {}", value_str);
            assert!(value_str.contains("hasGetComposedRanges\":true"), "getComposedRanges should be available");
        },
        Err(e) => panic!("Failed to test Selection API methods: {:?}", e),
    }

    // Test 3: Range API integration
    let range_test = r#"
        try {
            // Test Range constructor availability
            var hasRange = typeof Range === 'function';

            if (hasRange) {
                var range = new Range();
                var rangeProps = {
                    hasStartContainer: 'startContainer' in range,
                    hasStartOffset: 'startOffset' in range,
                    hasEndContainer: 'endContainer' in range,
                    hasEndOffset: 'endOffset' in range,
                    hasCollapsed: 'collapsed' in range,
                    hasCommonAncestorContainer: 'commonAncestorContainer' in range,
                };

                var rangeMethods = {
                    hasSetStart: typeof range.setStart === 'function',
                    hasSetEnd: typeof range.setEnd === 'function',
                    hasCollapse: typeof range.collapse === 'function',
                    hasSelectNode: typeof range.selectNode === 'function',
                    hasSelectNodeContents: typeof range.selectNodeContents === 'function',
                    hasCompareBoundaryPoints: typeof range.compareBoundaryPoints === 'function',
                    hasCloneRange: typeof range.cloneRange === 'function',
                    hasDetach: typeof range.detach === 'function',
                };

                'Range API: props=' + JSON.stringify(rangeProps) + ', methods=' + JSON.stringify(rangeMethods);
            } else {
                'Range constructor not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(range_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Range API integration: {}", value_str);
            assert!(value_str.contains("hasSetStart\":true"), "Range API methods should be available");
        },
        Err(e) => panic!("Failed to test Range API integration: {:?}", e),
    }

    // Test 4: Selection state management
    let state_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test initial state
                var initialState = {
                    rangeCount: selection.rangeCount,
                    isCollapsed: selection.isCollapsed,
                    type: selection.type,
                    direction: selection.direction,
                };

                'Initial state: ' + JSON.stringify(initialState);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(state_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ Selection state management: {}", value_str);
            // Initial state should show empty selection
            assert!(value_str.contains("rangeCount"), "Selection should have rangeCount property");
        },
        Err(e) => panic!("Failed to test Selection state management: {:?}", e),
    }

    println!("✅ Comprehensive Selection API test completed");
}