use std::sync::{Arc, Mutex};
use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_multiple_script_extraction_and_execution() {
    let browser = HeadlessWebBrowser::new();
    let mut browser_guard = browser.lock().unwrap();

    // HTML with multiple script tags that should all be executed
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Multiple Scripts Test</title>
            <script>
                var globalVar1 = "first script";
                window.testResults = [];
                window.testResults.push(globalVar1);
            </script>
        </head>
        <body>
            <div id="content">Original content</div>
            <script>
                var globalVar2 = "second script";
                window.testResults.push(globalVar2);
                document.getElementById('content').textContent = 'Modified by JavaScript';
            </script>
            <script>
                var globalVar3 = "third script";
                window.testResults.push(globalVar3);
                window.allScriptsExecuted = true;
            </script>
        </body>
        </html>
    "#;

    // Update the document HTML
    if let Some(ref mut renderer) = browser_guard.renderer {
        let _ = renderer.update_document_html(html_content);

        // Extract and execute all JavaScript
        let js_code = browser_guard.extract_safe_javascript(html_content).unwrap();

        // Verify that all script contents are included
        assert!(js_code.contains("first script"));
        assert!(js_code.contains("second script"));
        assert!(js_code.contains("third script"));

        // Execute the combined JavaScript
        let result = renderer.evaluate_javascript(&js_code);
        assert!(result.is_ok(), "JavaScript execution should succeed: {:?}", result);

        // Verify all scripts executed by checking global variables
        let check_execution = r#"
            if (typeof window.testResults !== 'undefined' &&
                window.testResults.length === 3 &&
                window.allScriptsExecuted === true) {
                'ALL_SCRIPTS_EXECUTED';
            } else {
                'SCRIPTS_FAILED: ' + JSON.stringify({
                    testResults: typeof window.testResults !== 'undefined' ? window.testResults : 'undefined',
                    allScriptsExecuted: typeof window.allScriptsExecuted !== 'undefined' ? window.allScriptsExecuted : 'undefined'
                });
            }
        "#;

        let execution_result = renderer.evaluate_javascript(check_execution).unwrap();
        assert!(execution_result.contains("ALL_SCRIPTS_EXECUTED"),
               "All scripts should execute successfully. Got: {}", execution_result);
    }
}

#[tokio::test]
async fn test_modern_javascript_features_allowed() {
    let browser = HeadlessWebBrowser::new();
    let mut browser_guard = browser.lock().unwrap();

    if let Some(ref mut renderer) = browser_guard.renderer {
        // Test that modern JavaScript features are now allowed and work
        let modern_js_tests = vec![
            // Arrow functions
            ("const add = (a, b) => a + b; add(2, 3);", "5"),
            // Template literals
            ("const name = 'World'; `Hello ${name}!`;", "Hello World!"),
            // Destructuring
            ("const [a, b] = [1, 2]; a + b;", "3"),
            // Array methods
            ("[1, 2, 3].map(x => x * 2).join(',');", "2,4,6"),
            // Object methods
            ("Object.keys({a: 1, b: 2}).length;", "2"),
            // Promise (basic)
            ("typeof Promise;", "function"),
            // JSON operations
            ("JSON.stringify({test: 'value'});", "{\"test\":\"value\"}"),
        ];

        for (js_code, expected_contains) in modern_js_tests {
            // First check that it passes security filtering
            assert!(renderer.is_safe_javascript(js_code),
                   "Modern JavaScript should pass security: {}", js_code);

            // Then check that it executes successfully
            let result = renderer.evaluate_javascript(js_code);
            assert!(result.is_ok(), "Modern JavaScript should execute: {}", js_code);

            let result_str = result.unwrap();
            assert!(result_str.contains(expected_contains) || result_str.contains(&format!("\"{}\"", expected_contains)),
                   "Result should contain '{}' for '{}', got: {}", expected_contains, js_code, result_str);
        }
    }
}

#[tokio::test]
async fn test_dom_manipulation_and_events() {
    let browser = HeadlessWebBrowser::new();
    let mut browser_guard = browser.lock().unwrap();

    let html_with_dom = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div id="test-div">Initial content</div>
            <button id="test-button">Click me</button>
        </body>
        </html>
    "#;

    if let Some(ref mut renderer) = browser_guard.renderer {
        let _ = renderer.update_document_html(html_with_dom);

        // Test DOM manipulation
        let dom_js = r#"
            // Test getElementById
            var testDiv = document.getElementById('test-div');
            if (testDiv) {
                testDiv.textContent = 'Modified by JS';
                'DOM_MODIFIED';
            } else {
                'ELEMENT_NOT_FOUND';
            }
        "#;

        assert!(renderer.is_safe_javascript(dom_js));
        let result = renderer.evaluate_javascript(dom_js).unwrap();
        assert!(result.contains("DOM_MODIFIED"), "DOM manipulation should work: {}", result);

        // Test event listener addition (should not crash)
        let event_js = r#"
            try {
                var button = document.getElementById('test-button');
                if (button && typeof button.addEventListener === 'function') {
                    button.addEventListener('click', function() {
                        console.log('Button clicked');
                    });
                    'EVENT_LISTENER_ADDED';
                } else {
                    'BUTTON_NOT_FOUND_OR_NO_ADDEVENTLISTENER';
                }
            } catch (e) {
                'ERROR: ' + e.message;
            }
        "#;

        assert!(renderer.is_safe_javascript(event_js));
        let event_result = renderer.evaluate_javascript(event_js).unwrap();
        assert!(event_result.contains("EVENT_LISTENER_ADDED") || event_result.contains("BUTTON_NOT_FOUND"),
               "Event listener addition should not crash: {}", event_result);
    }
}

#[tokio::test]
async fn test_web_api_compatibility() {
    let browser = HeadlessWebBrowser::new();
    let mut browser_guard = browser.lock().unwrap();

    if let Some(ref mut renderer) = browser_guard.renderer {
        // Test that standard Web APIs are available and security filtering allows them
        let web_api_tests = vec![
            "typeof console",
            "typeof window",
            "typeof document",
            "typeof navigator",
            "typeof location",
            "typeof history",
            "typeof localStorage",
            "typeof sessionStorage",
            "typeof XMLHttpRequest",
            "typeof fetch",
            "typeof setTimeout",
            "typeof setInterval",
        ];

        for api_test in web_api_tests {
            assert!(renderer.is_safe_javascript(api_test),
                   "Web API check should pass security: {}", api_test);

            let result = renderer.evaluate_javascript(api_test);
            assert!(result.is_ok(), "Web API should be available: {}", api_test);

            let result_str = result.unwrap();
            // Should be either "function", "object", or "undefined" - not an error
            assert!(result_str.contains("function") || result_str.contains("object") || result_str.contains("undefined"),
                   "API '{}' should return a valid type, got: {}", api_test, result_str);
        }
    }
}

#[tokio::test]
async fn test_javascript_security_boundary() {
    let browser = HeadlessWebBrowser::new();
    let browser_guard = browser.lock().unwrap();

    if let Some(ref renderer) = browser_guard.renderer {
        // These should still be blocked as they're Node.js/system specific
        let blocked_patterns = vec![
            "require('fs')",
            "process.exit(0)",
            "__dirname",
            "__filename",
            "global.process",
            "Buffer.allocUnsafe(1000)",
        ];

        for pattern in blocked_patterns {
            assert!(!renderer.is_safe_javascript(pattern),
                   "Dangerous system pattern should be blocked: {}", pattern);
        }

        // These should be allowed as they're standard web APIs
        let allowed_patterns = vec![
            "window.location.href",
            "document.cookie = 'test=value'",
            "localStorage.setItem('key', 'value')",
            "fetch('https://api.example.com')",
            "new XMLHttpRequest()",
            "eval('1 + 1')",
            "setTimeout(() => {}, 100)",
        ];

        for pattern in allowed_patterns {
            assert!(renderer.is_safe_javascript(pattern),
                   "Standard web API should be allowed: {}", pattern);
        }
    }
}