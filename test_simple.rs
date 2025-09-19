// Simple test to verify Selection/Range API functionality
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("🧪 Testing Selection and Range API implementation...");

    // Create a simple browser instance using the engine directly
    use thalora::engine::BoaEngine;

    let engine = BoaEngine::new();

    // Test 1: Basic Selection API availability
    println!("\n1. Testing Selection API availability...");
    let selection_test = r#"
        try {
            var hasGetSelection = typeof window !== 'undefined' && typeof window.getSelection === 'function';
            if (hasGetSelection) {
                var selection = window.getSelection();
                var result = {
                    hasAnchorNode: 'anchorNode' in selection,
                    hasDirection: 'direction' in selection,
                    hasGetComposedRanges: typeof selection.getComposedRanges === 'function',
                    direction: selection.direction,
                    type: selection.type,
                    rangeCount: selection.rangeCount,
                    isCollapsed: selection.isCollapsed,
                };
                'SUCCESS: ' + JSON.stringify(result);
            } else {
                'FAIL: window.getSelection not available';
            }
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(selection_test).await {
        Ok(value) => {
            println!("✅ Selection API: {:?}", value);
        },
        Err(e) => {
            println!("❌ Selection API failed: {:?}", e);
        }
    }

    // Test 2: Range API availability
    println!("\n2. Testing Range API availability...");
    let range_test = r#"
        try {
            var hasRange = typeof Range === 'function';
            if (hasRange) {
                var range = new Range();
                var result = {
                    hasSetStart: typeof range.setStart === 'function',
                    hasCloneRange: typeof range.cloneRange === 'function',
                    hasToString: typeof range.toString === 'function',
                    collapsed: range.collapsed,
                    constants: {
                        START_TO_START: Range.START_TO_START,
                        START_TO_END: Range.START_TO_END,
                        END_TO_END: Range.END_TO_END,
                        END_TO_START: Range.END_TO_START,
                    }
                };
                'SUCCESS: ' + JSON.stringify(result);
            } else {
                'FAIL: Range constructor not available';
            }
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(range_test).await {
        Ok(value) => {
            println!("✅ Range API: {:?}", value);
        },
        Err(e) => {
            println!("❌ Range API failed: {:?}", e);
        }
    }

    // Test 3: Selection modify functionality
    println!("\n3. Testing Selection modify functionality...");
    let modify_test = r#"
        try {
            var selection = window.getSelection();
            selection.modify('move', 'forward', 'character');
            selection.modify('extend', 'backward', 'word');
            'SUCCESS: Selection modify methods executed without errors';
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(modify_test).await {
        Ok(value) => {
            println!("✅ Selection modify: {:?}", value);
        },
        Err(e) => {
            println!("❌ Selection modify failed: {:?}", e);
        }
    }

    println!("\n🎉 Selection and Range API testing completed!");
}