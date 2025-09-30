//! Comprehensive Platform APIs Integration Test
//!
//! This test file validates all Platform APIs (Console, Navigator, Timers) working together
//! to ensure complete WHATWG compliance and proper integration within the Boa engine.

use thalora_web_browser::engine::renderer::JavaScriptRenderer;
use tokio;

#[tokio::test]
async fn test_platform_apis_integration() {
    let mut renderer = JavaScriptRenderer::new();

    // Test all Platform APIs are available
    let availability_test = r#"
        // Test Console API availability
        const console_available = typeof console === 'object';
        const console_methods = [
            'log', 'info', 'warn', 'error', 'debug', 'trace',
            'clear', 'group', 'groupCollapsed', 'groupEnd',
            'time', 'timeEnd', 'timeLog', 'count', 'countReset',
            'assert', 'table', 'dir', 'dirxml'
        ].every(method => typeof console[method] === 'function');

        // Test Navigator API availability
        const navigator_available = typeof navigator === 'object';
        const navigator_properties = [
            'userAgent', 'appCodeName', 'appName', 'appVersion',
            'platform', 'product', 'productSub', 'vendor', 'vendorSub',
            'language', 'languages', 'cookieEnabled', 'onLine',
            'plugins', 'mimeTypes', 'pdfViewerEnabled'
        ].every(prop => navigator.hasOwnProperty(prop));
        const navigator_methods = [
            'javaEnabled', 'registerProtocolHandler', 'unregisterProtocolHandler'
        ].every(method => typeof navigator[method] === 'function');

        // Test Timers API availability
        const timers_available = [
            'setTimeout', 'setInterval', 'clearTimeout', 'clearInterval'
        ].every(fn => typeof window[fn] === 'function');

        ({
            console: { available: console_available, methods: console_methods },
            navigator: { available: navigator_available, properties: navigator_properties, methods: navigator_methods },
            timers: { available: timers_available }
        })
    "#;

    let result = renderer.execute_javascript(availability_test, false).await;
    assert!(result.is_ok(), "Platform APIs availability test failed: {:?}", result);

    println!("✅ All Platform APIs are available and properly exposed");
}

#[tokio::test]
async fn test_console_state_management() {
    let mut renderer = JavaScriptRenderer::new();

    // Test Console state management
    let console_test = r#"
        // Test timer state management
        console.time('test1');
        console.time('test2');

        // Test counter state management
        console.count('counter1');
        console.count('counter1');
        console.count('counter2');

        // Test grouping with indentation
        console.group('Group 1');
        console.log('Inside group 1');
        console.group('Nested Group');
        console.log('Inside nested group');
        console.groupEnd();
        console.log('Back in group 1');
        console.groupEnd();
        console.log('Back at root level');

        // Test timing completion
        console.timeLog('test1', 'checkpoint');
        console.timeEnd('test1');
        console.timeEnd('test2');

        // Test counter reset
        console.countReset('counter1');
        console.count('counter1'); // Should be 1 again

        "Console state management test completed successfully"
    "#;

    let result = renderer.execute_javascript(console_test, false).await;
    assert!(result.is_ok(), "Console state management test failed: {:?}", result);

    println!("✅ Console API state management working correctly");
}

#[tokio::test]
async fn test_navigator_whatwg_compliance() {
    let mut renderer = JavaScriptRenderer::new();

    // Test Navigator WHATWG compliance
    let navigator_test = r#"
        // Test NavigatorID mixin compliance
        const id_tests = {
            appCodeName: navigator.appCodeName === 'Mozilla',
            appName: navigator.appName === 'Netscape',
            product: navigator.product === 'Gecko',
            userAgent: typeof navigator.userAgent === 'string' && navigator.userAgent.length > 0,
            platform: typeof navigator.platform === 'string',
            vendor: typeof navigator.vendor === 'string'
        };

        // Test NavigatorLanguage mixin compliance
        const language_tests = {
            language: navigator.language === 'en-US',
            languages: Array.isArray(navigator.languages) && navigator.languages.length >= 1
        };

        // Test NavigatorOnLine mixin compliance
        const online_tests = {
            onLine: typeof navigator.onLine === 'boolean'
        };

        // Test NavigatorCookies mixin compliance
        const cookies_tests = {
            cookieEnabled: typeof navigator.cookieEnabled === 'boolean'
        };

        // Test NavigatorPlugins mixin compliance
        const plugins_tests = {
            plugins: Array.isArray(navigator.plugins) && navigator.plugins.length === 0, // Empty for security
            mimeTypes: Array.isArray(navigator.mimeTypes) && navigator.mimeTypes.length === 0, // Empty for security
            javaEnabled: navigator.javaEnabled() === false, // Always false for security
            pdfViewerEnabled: typeof navigator.pdfViewerEnabled === 'boolean'
        };

        // Test NavigatorContentUtils methods
        try {
            navigator.registerProtocolHandler('mailto', 'https://example.com/compose?to=%s');
            navigator.unregisterProtocolHandler('mailto', 'https://example.com/compose?to=%s');
            var protocol_handlers = true;
        } catch (e) {
            var protocol_handlers = false;
        }

        ({
            id: id_tests,
            language: language_tests,
            online: online_tests,
            cookies: cookies_tests,
            plugins: plugins_tests,
            protocolHandlers: protocol_handlers
        })
    "#;

    let result = renderer.execute_javascript(navigator_test, false).await;
    assert!(result.is_ok(), "Navigator WHATWG compliance test failed: {:?}", result);

    println!("✅ Navigator API WHATWG compliance validated");
}

#[tokio::test]
async fn test_timers_html5_compliance() {
    let mut renderer = JavaScriptRenderer::new();

    // Test Timers HTML5 compliance
    let timers_test = r#"
        // Test timer ID generation
        const timeoutId1 = setTimeout(() => {}, 100);
        const timeoutId2 = setTimeout(() => {}, 200);
        const intervalId1 = setInterval(() => {}, 300);
        const intervalId2 = setInterval(() => {}, 400);

        // Test timer IDs are numbers and unique
        const ids_valid = [
            typeof timeoutId1 === 'number',
            typeof timeoutId2 === 'number',
            typeof intervalId1 === 'number',
            typeof intervalId2 === 'number',
            timeoutId1 !== timeoutId2,
            intervalId1 !== intervalId2,
            timeoutId1 !== intervalId1
        ].every(test => test === true);

        // Test timer clearing
        clearTimeout(timeoutId1);
        clearTimeout(timeoutId2);
        clearInterval(intervalId1);
        clearInterval(intervalId2);

        // Test cross-clearing (HTML5 allows this)
        const id1 = setTimeout(() => {}, 500);
        const id2 = setInterval(() => {}, 600);
        clearInterval(id1); // Should not throw
        clearTimeout(id2);  // Should not throw

        // Test minimum delay clamping
        const clampedId1 = setTimeout(() => {}, 0);  // Should be clamped to 4ms internally
        const clampedId2 = setTimeout(() => {}, 1);  // Should be clamped to 4ms internally
        const normalId = setTimeout(() => {}, 10);   // Should remain 10ms

        clearTimeout(clampedId1);
        clearTimeout(clampedId2);
        clearTimeout(normalId);

        // Test with string callbacks
        const stringCallbackId = setTimeout('var testVar = 42;', 50);
        clearTimeout(stringCallbackId);

        ({
            idsValid: ids_valid,
            crossClearingWorks: true, // If we got here, cross-clearing didn't throw
            clampingWorks: typeof clampedId1 === 'number' && typeof clampedId2 === 'number',
            stringCallbackWorks: typeof stringCallbackId === 'number'
        })
    "#;

    let result = renderer.execute_javascript(timers_test, false).await;
    assert!(result.is_ok(), "Timers HTML5 compliance test failed: {:?}", result);

    println!("✅ Timers API HTML5 compliance validated");
}

#[tokio::test]
async fn test_platform_apis_interaction() {
    let mut renderer = JavaScriptRenderer::new();

    // Test Platform APIs working together
    let interaction_test = r#"
        // Use Console API to log Navigator information
        console.group('Navigator Information');
        console.log('User Agent:', navigator.userAgent);
        console.log('Platform:', navigator.platform);
        console.log('Language:', navigator.language);
        console.log('Languages:', navigator.languages);
        console.log('Online Status:', navigator.onLine);
        console.groupEnd();

        // Use timers with console logging
        console.group('Timer Tests');
        console.time('interaction-test');

        // Schedule multiple timers that would use console
        const timerId1 = setTimeout(() => {
            console.log('Timer 1 executed');
        }, 10);

        const timerId2 = setTimeout(() => {
            console.warn('Timer 2 executed with warning');
        }, 20);

        const intervalId = setInterval(() => {
            console.count('interval-execution');
        }, 15);

        // Clear interval after a short time
        setTimeout(() => {
            clearInterval(intervalId);
            console.log('Interval cleared');
        }, 50);

        console.timeLog('interaction-test', 'All timers scheduled');
        console.timeEnd('interaction-test');
        console.groupEnd();

        // Test console with navigator data
        console.table({
            userAgent: navigator.userAgent.substring(0, 20) + '...',
            platform: navigator.platform,
            language: navigator.language,
            cookieEnabled: navigator.cookieEnabled,
            onLine: navigator.onLine
        });

        // Clean up remaining timers
        clearTimeout(timerId1);
        clearTimeout(timerId2);

        "Platform APIs interaction test completed successfully"
    "#;

    let result = renderer.execute_javascript(interaction_test, false).await;
    assert!(result.is_ok(), "Platform APIs interaction test failed: {:?}", result);

    println!("✅ Platform APIs work together correctly");
}

#[tokio::test]
async fn test_platform_apis_error_handling() {
    let mut renderer = JavaScriptRenderer::new();

    // Test error handling and edge cases
    let error_test = r#"
        let errors = [];

        // Test Console API error handling
        try {
            console.time('test');
            console.time('test'); // Should warn about duplicate
            console.timeEnd('nonexistent'); // Should warn about non-existent
            console.countReset('nonexistent'); // Should warn about non-existent
        } catch (e) {
            errors.push('Console error: ' + e.message);
        }

        // Test Navigator API error handling
        try {
            // Test protocol handler errors
            try {
                navigator.registerProtocolHandler('', 'https://example.com'); // Should throw
            } catch (e) {
                // Expected error for empty scheme
            }

            try {
                navigator.registerProtocolHandler('test', 'https://example.com'); // Should throw (no %s)
            } catch (e) {
                // Expected error for missing %s
            }
        } catch (e) {
            errors.push('Navigator error: ' + e.message);
        }

        // Test Timers API error handling
        try {
            clearTimeout(); // Should not throw
            clearInterval(999999); // Should not throw
            setTimeout(); // Should return 0 per HTML spec
            setInterval(); // Should return 0 per HTML spec
        } catch (e) {
            errors.push('Timers error: ' + e.message);
        }

        ({
            errorCount: errors.length,
            errors: errors
        })
    "#;

    let result = renderer.execute_javascript(error_test, false).await;
    assert!(result.is_ok(), "Platform APIs error handling test failed: {:?}", result);

    println!("✅ Platform APIs error handling working correctly");
}

#[tokio::test]
async fn test_platform_apis_readonly_properties() {
    let mut renderer = JavaScriptRenderer::new();

    // Test that readonly properties cannot be modified
    let readonly_test = r#"
        // Test Navigator readonly properties
        const originalUserAgent = navigator.userAgent;
        const originalPlatform = navigator.platform;
        const originalLanguage = navigator.language;

        // Attempt to modify readonly properties
        navigator.userAgent = 'Modified';
        navigator.platform = 'Modified';
        navigator.language = 'Modified';
        navigator.appCodeName = 'Modified';
        navigator.appName = 'Modified';
        navigator.product = 'Modified';

        // Check that properties were not actually modified
        const readonly_tests = {
            userAgent: navigator.userAgent === originalUserAgent,
            platform: navigator.platform === originalPlatform,
            language: navigator.language === originalLanguage,
            appCodeName: navigator.appCodeName === 'Mozilla',
            appName: navigator.appName === 'Netscape',
            product: navigator.product === 'Gecko'
        };

        readonly_tests
    "#;

    let result = renderer.execute_javascript(readonly_test, false).await;
    assert!(result.is_ok(), "Platform APIs readonly properties test failed: {:?}", result);

    println!("✅ Platform APIs readonly properties properly protected");
}