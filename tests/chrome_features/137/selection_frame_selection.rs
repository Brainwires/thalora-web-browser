#[tokio::test]
#[ignore]
async fn test_chrome_137_selection_frame_selection_architecture() {
    println!("🧪 Testing Chrome 137: FrameSelection Architecture...");

    let browser = HeadlessWebBrowser::new();

    // Test 1: Selection granularity and modification (Chrome's FrameSelection features)
    let granularity_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test modify method (Chrome's FrameSelection feature)
                var hasModify = typeof selection.modify === 'function';

                if (hasModify) {
                    // Test that modify method accepts proper parameters
                    try {
                        // This should not throw errors even with empty document
                        selection.modify('move', 'forward', 'character');
                        selection.modify('extend', 'backward', 'word');
                        selection.modify('move', 'left', 'line');

                        var modifyWorking = true;
                    } catch (modifyError) {
                        var modifyWorking = false;
                    }

                    'Modify method: available=' + hasModify + ', working=' + modifyWorking;
                } else {
                    'Modify method not available';
                }
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(granularity_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ FrameSelection modify method: {}", value_str);
            assert!(value_str.contains("available=true"), "Selection.modify should be available");
        },
        Err(e) => panic!("Failed to test FrameSelection modify method: {:?}", e),
    }

    // Test 2: Selection direction (FrameSelection internal state)
    let direction_test = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test direction property (reflects FrameSelection internal state)
                var directionType = typeof selection.direction;
                var directionValue = selection.direction;

                var directionTests = {
                    hasDirection: 'direction' in selection,
                    directionType: directionType,
                    initialDirection: directionValue,
                };

                'Direction tests: ' + JSON.stringify(directionTests);
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(direction_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ FrameSelection direction: {}", value_str);
            assert!(value_str.contains("hasDirection\":true"), "Selection should have direction property");
        },
        Err(e) => panic!("Failed to test FrameSelection direction: {:?}", e),
    }

    // Test 3: Selection change event system (FrameSelection event management)
    let events_test = r#"
        try {
            if (typeof document !== 'undefined') {
                // Test selection change event support
                var hasSelectionChange = 'onselectionchange' in document;
                var eventSupport = typeof document.addEventListener === 'function';

                var eventTests = {
                    hasOnSelectionChange: hasSelectionChange,
                    hasAddEventListener: eventSupport,
                };

                // Test adding selection change listener
                if (eventSupport) {
                    try {
                        var listenerAdded = false;
                        document.addEventListener('selectionchange', function() {
                            // Event listener for selection changes
                        });
                        listenerAdded = true;
                        eventTests.canAddListener = listenerAdded;
                    } catch (listenerError) {
                        eventTests.canAddListener = false;
                    }
                }

                'Event system: ' + JSON.stringify(eventTests);
            } else {
                'document not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(events_test).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("✅ FrameSelection event system: {}", value_str);
            // Event system should be functional
            assert!(value_str.contains("hasAddEventListener\":true"), "Event system should be available");
        },
        Err(e) => panic!("Failed to test FrameSelection event system: {:?}", e),
    }

    println!("✅ FrameSelection architecture test completed");
}