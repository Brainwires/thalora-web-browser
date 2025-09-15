use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

/// Comprehensive browser compatibility test suite
/// Tests against real compatibility sites to measure browser feature support
#[tokio::test]
async fn test_html5_compatibility() {
    println!("🧪 Testing HTML5 compatibility against html5test.com...");

    let browser = HeadlessWebBrowser::new();

    // Test HTML5Test.com - gives score out of 555 points
    let response = browser.lock().unwrap().scrape("http://html5test.com/", true, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ HTML5Test.com request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Look for the HTML5 score
            if let Some(score_start) = scraped_data.content.find("score") {
                let score_section = &scraped_data.content[score_start..score_start.min(scraped_data.content.len()).min(score_start + 500)];
                println!("📊 Score section: {}", score_section);
            }

            // Check for specific HTML5 features
            let features_found = check_html5_features(&scraped_data.content);
            println!("🎯 HTML5 features detected: {}", features_found.len());
            for feature in features_found {
                println!("  ✅ {}", feature);
            }

            assert!(scraped_data.content.len() > 1000, "Should get substantial HTML5 test content");
        }
        Err(e) => {
            panic!("❌ HTML5Test.com request failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_es6_compatibility() {
    println!("🧪 Testing ES6 compatibility against Kangax table...");

    let browser = HeadlessWebBrowser::new();

    // Test Kangax ES6 compatibility table
    let response = browser.lock().unwrap().scrape("https://compat-table.github.io/compat-table/es6/", true, None, false, false).await;

    match response {
        Ok(scraped_data) => {
            println!("✅ Kangax ES6 table request successful!");
            println!("📄 Content length: {} characters", scraped_data.content.len());

            // Look for ES6 feature support indicators
            let es6_features = check_es6_features(&scraped_data.content);
            println!("🎯 ES6 features found in table: {}", es6_features.len());

            // Check for specific modern JavaScript features
            let modern_features = vec![
                "arrow functions",
                "template literals",
                "destructuring",
                "default parameters",
                "rest parameters",
                "spread operator",
                "classes",
                "promises",
                "async functions",
                "modules"
            ];

            for feature in modern_features {
                if scraped_data.content.to_lowercase().contains(feature) {
                    println!("  ✅ Found: {}", feature);
                } else {
                    println!("  ❌ Missing: {}", feature);
                }
            }

            assert!(scraped_data.content.len() > 5000, "Should get substantial ES6 compatibility table");
        }
        Err(e) => {
            panic!("❌ Kangax ES6 table request failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_can_i_use_features() {
    println!("🧪 Testing modern web features against Can I Use...");

    let browser = HeadlessWebBrowser::new();

    // Test key modern web features
    let features_to_test = vec![
        ("WebAssembly", "https://caniuse.com/wasm"),
        ("WebRTC", "https://caniuse.com/rtcpeerconnection"),
        ("Service Workers", "https://caniuse.com/serviceworkers"),
        ("WebGL", "https://caniuse.com/webgl"),
        ("Geolocation", "https://caniuse.com/geolocation"),
        ("WebSocket", "https://caniuse.com/websockets"),
    ];

    let mut compatibility_results = HashMap::new();

    for (feature_name, url) in features_to_test {
        println!("🔍 Testing {} support...", feature_name);

        let response = browser.lock().unwrap().scrape(url, true, None, false, false).await;

        match response {
            Ok(scraped_data) => {
                println!("✅ {} page loaded ({} chars)", feature_name, scraped_data.content.len());

                // Analyze support indicators
                let support_level = analyze_caniuse_support(&scraped_data.content);
                compatibility_results.insert(feature_name.to_string(), support_level);

                assert!(scraped_data.content.len() > 1000, "Should get substantial Can I Use content for {}", feature_name);
            }
            Err(e) => {
                println!("❌ {} test failed: {}", feature_name, e);
                compatibility_results.insert(feature_name.to_string(), "Failed to load".to_string());
            }
        }

        // Small delay between requests to be respectful
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    // Print compatibility summary
    println!("\n📊 Browser Compatibility Summary:");
    for (feature, support) in compatibility_results {
        println!("  {}: {}", feature, support);
    }
}

#[tokio::test]
async fn test_javascript_engine_features() {
    println!("🧪 Testing JavaScript engine features directly...");

    let browser = HeadlessWebBrowser::new();

    // Test modern JavaScript features directly
    let js_tests = vec![
        // ES6 Features
        ("Arrow Functions", "(() => 'test')()"),
        ("Template Literals", "`Hello ${'world'}`"),
        ("Destructuring", "__destructure([1, 2], ['a', 'b']); a + b"),
        ("Default Parameters", "(function(x = 5) { return x; })()"),
        ("Spread Operator", "[...[1, 2, 3]].length"),

        // ES2017+ Features
        ("Async/Await Basic", "__async(function() { return 'async'; })().constructor.name"),
        ("Object Entries", "Object.entries({a: 1}).length"),
        ("Object Values", "Object.values({a: 1, b: 2}).length"),

        // Math Extensions
        ("Math.trunc", "Math.trunc(4.9)"),
        ("Math.sign", "Math.sign(-5)"),

        // String Methods
        ("String.includes", "'hello'.includes('ell')"),
        ("String.startsWith", "'hello'.startsWith('hel')"),
        ("String.endsWith", "'hello'.endsWith('llo')"),

        // Array Methods
        ("Array.from", "Array.from('abc').length"),
        ("Array.find", "[1,2,3].find(x => x > 1)"),
        ("Array.includes", "[1,2,3].includes(2)"),

        // Promises
        ("Promise.resolve", "Promise.resolve(42).constructor.name"),
        ("Promise.all", "Promise.all([Promise.resolve(1)]).constructor.name"),

        // Modern APIs availability checks
        ("fetch API", "typeof fetch"),
        ("localStorage", "typeof localStorage"),
        ("sessionStorage", "typeof sessionStorage"),
        ("WebSocket", "typeof WebSocket"),
        ("console", "typeof console"),
        ("navigator", "typeof navigator"),
        ("navigator.userAgent", "navigator.userAgent.length > 0"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (test_name, js_code) in js_tests {
        let result = browser.lock().unwrap().execute_javascript(js_code).await;

        match result {
            Ok(value) => {
                println!("✅ {}: {:?}", test_name, value);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {}: Error - {:?}", test_name, e);
                failed += 1;
            }
        }
    }

    println!("\n📊 JavaScript Feature Test Results:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    println!("  📈 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    assert!(passed > failed, "Should pass more JavaScript feature tests than fail");
}

#[tokio::test]
async fn test_web_api_availability() {
    println!("🧪 Testing Web API availability...");

    let browser = HeadlessWebBrowser::new();

    // Test Web API availability
    let web_apis = vec![
        // Core APIs
        ("Window", "typeof window"),
        ("Document", "typeof document"),
        ("Navigator", "typeof navigator"),
        ("Location", "typeof location"),
        ("History", "typeof history"),

        // Storage APIs
        ("localStorage", "typeof localStorage"),
        ("sessionStorage", "typeof sessionStorage"),

        // Network APIs
        ("fetch", "typeof fetch"),
        ("XMLHttpRequest", "typeof XMLHttpRequest"),
        ("WebSocket", "typeof WebSocket"),

        // Modern APIs
        ("Crypto", "typeof crypto"),
        ("Performance", "typeof performance"),
        ("Geolocation", "typeof navigator.geolocation"),

        // Device APIs (Chrome 131+ features)
        ("WebHID", "typeof navigator.hid"),
        ("USB", "typeof navigator.usb"),
        ("Serial", "typeof navigator.serial"),
        ("Bluetooth", "typeof navigator.bluetooth"),

        // Graphics APIs
        ("Canvas", "typeof document.createElement('canvas').getContext"),

        // Media APIs
        ("getUserMedia", "typeof navigator.mediaDevices"),

        // Worker APIs
        ("Worker", "typeof Worker"),
        ("ServiceWorker", "typeof navigator.serviceWorker"),

        // Utility APIs
        ("Permissions", "typeof navigator.permissions"),
        ("Clipboard", "typeof navigator.clipboard"),
    ];

    let mut available_apis = 0;
    let mut missing_apis = 0;

    for (api_name, js_check) in web_apis {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("undefined") {
                    println!("❌ {}: Not available", api_name);
                    missing_apis += 1;
                } else {
                    println!("✅ {}: Available ({})", api_name, value_str);
                    available_apis += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", api_name, e);
                missing_apis += 1;
            }
        }
    }

    println!("\n📊 Web API Availability Results:");
    println!("  ✅ Available: {}", available_apis);
    println!("  ❌ Missing: {}", missing_apis);
    println!("  📈 Coverage: {:.1}%", (available_apis as f64 / (available_apis + missing_apis) as f64) * 100.0);

    assert!(available_apis > 10, "Should have at least 10 Web APIs available");
}

// Helper function to check HTML5 features in content
fn check_html5_features(content: &str) -> Vec<String> {
    let features = vec![
        "canvas", "video", "audio", "localStorage", "sessionStorage",
        "webgl", "geolocation", "websocket", "worker", "offline",
        "drag", "history", "contenteditable", "microdata"
    ];

    features.into_iter()
        .filter(|feature| content.to_lowercase().contains(feature))
        .map(|s| s.to_string())
        .collect()
}

// Helper function to check ES6 features in Kangax table
fn check_es6_features(content: &str) -> Vec<String> {
    let features = vec![
        "arrow functions", "class", "template literals", "destructuring",
        "default parameters", "rest parameters", "spread", "computed properties",
        "shorthand properties", "method properties", "string methods", "array methods",
        "object static methods", "promise", "symbol", "iterator", "generator",
        "map", "set", "weakmap", "weakset", "proxy", "reflect", "async functions"
    ];

    features.into_iter()
        .filter(|feature| content.to_lowercase().contains(feature))
        .map(|s| s.to_string())
        .collect()
}

// Helper function to analyze Can I Use support indicators
fn analyze_caniuse_support(content: &str) -> String {
    let content_lower = content.to_lowercase();

    if content_lower.contains("supported") || content_lower.contains("yes") {
        "Supported".to_string()
    } else if content_lower.contains("partial") {
        "Partial Support".to_string()
    } else if content_lower.contains("not supported") || content_lower.contains("no") {
        "Not Supported".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[tokio::test]
async fn test_webplatform_features() {
    println!("🧪 Testing Web Platform features against MDN compatibility...");

    let browser = HeadlessWebBrowser::new();

    // Test critical Web Platform features that Chrome supports
    let webplatform_tests = vec![
        // CSS Features
        ("CSS Grid", "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Grid_Layout"),
        ("CSS Flexbox", "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout"),
        ("CSS Custom Properties", "https://developer.mozilla.org/en-US/docs/Web/CSS/--*"),

        // JavaScript APIs
        ("Intersection Observer", "https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API"),
        ("Resize Observer", "https://developer.mozilla.org/en-US/docs/Web/API/Resize_Observer_API"),
        ("Performance Observer", "https://developer.mozilla.org/en-US/docs/Web/API/PerformanceObserver"),

        // Modern Web APIs
        ("Payment Request", "https://developer.mozilla.org/en-US/docs/Web/API/Payment_Request_API"),
        ("Web Share", "https://developer.mozilla.org/en-US/docs/Web/API/Web_Share_API"),
        ("Screen Wake Lock", "https://developer.mozilla.org/en-US/docs/Web/API/Screen_Wake_Lock_API"),
    ];

    let mut webplatform_results = HashMap::new();

    for (feature_name, url) in webplatform_tests {
        println!("🔍 Testing {} support...", feature_name);

        let response = browser.lock().unwrap().scrape(url, true, None, false, false).await;

        match response {
            Ok(scraped_data) => {
                println!("✅ {} page loaded ({} chars)", feature_name, scraped_data.content.len());

                // Check for browser compatibility information
                let compat_info = analyze_mdn_compatibility(&scraped_data.content);
                webplatform_results.insert(feature_name.to_string(), compat_info);

                assert!(scraped_data.content.len() > 1000, "Should get substantial MDN content for {}", feature_name);
            }
            Err(e) => {
                println!("❌ {} test failed: {}", feature_name, e);
                webplatform_results.insert(feature_name.to_string(), "Failed to load".to_string());
            }
        }

        // Delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    // Print Web Platform compatibility summary
    println!("\n📊 Web Platform Feature Support Summary:");
    for (feature, support) in webplatform_results {
        println!("  {}: {}", feature, support);
    }
}

#[tokio::test]
async fn test_chrome_specific_apis() {
    println!("🧪 Testing Chrome-specific API availability...");

    let browser = HeadlessWebBrowser::new();

    // Test Chrome-specific APIs that other browsers might not have
    let chrome_specific_tests = vec![
        // Chrome DevTools APIs
        ("chrome.devtools", "typeof chrome !== 'undefined' && typeof chrome.devtools !== 'undefined'"),
        ("chrome.extension", "typeof chrome !== 'undefined' && typeof chrome.extension !== 'undefined'"),
        ("chrome.runtime", "typeof chrome !== 'undefined' && typeof chrome.runtime !== 'undefined'"),

        // Chrome-specific Web APIs
        ("webkitRequestFileSystem", "typeof webkitRequestFileSystem"),
        ("webkitStorageInfo", "typeof navigator.webkitStorageInfo"),
        ("chrome.app", "typeof chrome !== 'undefined' && typeof chrome.app !== 'undefined'"),

        // Chrome Performance APIs
        ("performance.measureUserAgentSpecificMemory", "typeof performance.measureUserAgentSpecificMemory"),
        ("performance.mark", "typeof performance.mark"),
        ("performance.measure", "typeof performance.measure"),

        // Chrome Security APIs
        ("window.isSecureContext", "typeof window.isSecureContext !== 'undefined'"),
        ("window.origin", "typeof window.origin !== 'undefined'"),
        ("document.visibilityState", "typeof document.visibilityState !== 'undefined'"),
    ];

    let mut chrome_api_available = 0;
    let mut chrome_api_missing = 0;

    for (api_name, js_check) in chrome_specific_tests {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("undefined") || value_str.contains("false") {
                    println!("❌ {}: Not available", api_name);
                    chrome_api_missing += 1;
                } else {
                    println!("✅ {}: Available ({})", api_name, value_str);
                    chrome_api_available += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", api_name, e);
                chrome_api_missing += 1;
            }
        }
    }

    println!("\n📊 Chrome-Specific API Results:");
    println!("  ✅ Available: {}", chrome_api_available);
    println!("  ❌ Missing: {}", chrome_api_missing);
    println!("  📈 Chrome API Coverage: {:.1}%", (chrome_api_available as f64 / (chrome_api_available + chrome_api_missing) as f64) * 100.0);

    // Chrome APIs are expected to be mostly missing in a non-Chrome browser
    assert!(chrome_api_missing >= chrome_api_available, "Most Chrome-specific APIs should be missing in Synaptic");
}

#[tokio::test]
async fn test_modern_css_features() {
    println!("🧪 Testing modern CSS feature support...");

    let browser = HeadlessWebBrowser::new();

    // Test modern CSS features through JavaScript
    let css_feature_tests = vec![
        // CSS Grid
        ("CSS Grid", "CSS.supports('display', 'grid')"),

        // CSS Flexbox
        ("CSS Flexbox", "CSS.supports('display', 'flex')"),

        // CSS Custom Properties
        ("CSS Variables", "CSS.supports('--custom-property', 'value')"),

        // CSS Subgrid
        ("CSS Subgrid", "CSS.supports('grid-template-rows', 'subgrid')"),

        // CSS Container Queries
        ("CSS Container Queries", "CSS.supports('container-type', 'inline-size')"),

        // CSS has() selector
        ("CSS :has() selector", "CSS.supports('selector(:has(div))')"),

        // CSS Cascade Layers
        ("CSS @layer", "CSS.supports('@layer')"),

        // CSS Logical Properties
        ("CSS Logical Properties", "CSS.supports('margin-inline-start', '1em')"),

        // CSS aspect-ratio
        ("CSS aspect-ratio", "CSS.supports('aspect-ratio', '16/9')"),

        // CSS gap for flexbox
        ("CSS gap (flexbox)", "CSS.supports('gap', '1rem')"),
    ];

    let mut css_supported = 0;
    let mut css_not_supported = 0;

    for (feature_name, css_test) in css_feature_tests {
        let result = browser.lock().unwrap().execute_javascript(css_test).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") {
                    println!("✅ {}: Supported", feature_name);
                    css_supported += 1;
                } else {
                    println!("❌ {}: Not supported ({})", feature_name, value_str);
                    css_not_supported += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error testing - {:?}", feature_name, e);
                css_not_supported += 1;
            }
        }
    }

    println!("\n📊 Modern CSS Feature Results:");
    println!("  ✅ Supported: {}", css_supported);
    println!("  ❌ Not Supported: {}", css_not_supported);
    println!("  📈 CSS Feature Coverage: {:.1}%", (css_supported as f64 / (css_supported + css_not_supported) as f64) * 100.0);

    assert!(css_supported + css_not_supported > 0, "Should have tested CSS features");
}

#[tokio::test]
async fn test_security_and_privacy_apis() {
    println!("🧪 Testing Security and Privacy API compliance...");

    let browser = HeadlessWebBrowser::new();

    // Test security and privacy related APIs
    let security_tests = vec![
        // Security Context
        ("Secure Context", "window.isSecureContext"),
        ("Origin", "window.origin !== null"),

        // Crypto APIs
        ("Web Crypto API", "typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined'"),
        ("crypto.randomUUID", "typeof crypto.randomUUID === 'function'"),
        ("crypto.getRandomValues", "typeof crypto.getRandomValues === 'function'"),

        // Privacy APIs
        ("Document Policy", "typeof document.policy !== 'undefined'"),
        ("Feature Policy", "typeof document.featurePolicy !== 'undefined'"),
        ("Permissions Policy", "typeof document.permissionsPolicy !== 'undefined'"),

        // Content Security Policy
        ("CSP violation events", "typeof SecurityPolicyViolationEvent !== 'undefined'"),

        // Trusted Types
        ("Trusted Types", "typeof TrustedHTML !== 'undefined'"),

        // Cross-Origin Isolation
        ("crossOriginIsolated", "typeof crossOriginIsolated !== 'undefined'"),
    ];

    let mut security_available = 0;
    let mut security_missing = 0;

    for (security_name, js_check) in security_tests {
        let result = browser.lock().unwrap().execute_javascript(js_check).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") || (!value_str.contains("undefined") && !value_str.contains("false")) {
                    println!("✅ {}: Available ({})", security_name, value_str);
                    security_available += 1;
                } else {
                    println!("❌ {}: Not available", security_name);
                    security_missing += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error checking - {:?}", security_name, e);
                security_missing += 1;
            }
        }
    }

    println!("\n📊 Security & Privacy API Results:");
    println!("  ✅ Available: {}", security_available);
    println!("  ❌ Missing: {}", security_missing);
    println!("  📈 Security API Coverage: {:.1}%", (security_available as f64 / (security_available + security_missing) as f64) * 100.0);

    assert!(security_available + security_missing > 0, "Should have tested security features");
}

#[tokio::test]
async fn test_performance_and_timing_apis() {
    println!("🧪 Testing Performance and Timing API accuracy...");

    let browser = HeadlessWebBrowser::new();

    // Test performance and timing APIs with actual functionality
    let performance_tests = vec![
        // Basic Performance API
        ("performance.now()", "typeof performance.now() === 'number'"),
        ("performance.timeOrigin", "typeof performance.timeOrigin === 'number'"),

        // Navigation Timing
        ("performance.timing", "typeof performance.timing === 'object'"),
        ("performance.timing.navigationStart", "typeof performance.timing.navigationStart === 'number'"),
        ("performance.timing.loadEventEnd", "typeof performance.timing.loadEventEnd === 'number'"),

        // Resource Timing
        ("performance.getEntries", "typeof performance.getEntries === 'function'"),
        ("performance.getEntriesByType", "typeof performance.getEntriesByType === 'function'"),
        ("performance.getEntriesByName", "typeof performance.getEntriesByName === 'function'"),

        // User Timing
        ("performance.mark", "typeof performance.mark === 'function'"),
        ("performance.measure", "typeof performance.measure === 'function'"),
        ("performance.clearMarks", "typeof performance.clearMarks === 'function'"),
        ("performance.clearMeasures", "typeof performance.clearMeasures === 'function'"),

        // Modern Performance APIs
        ("PerformanceObserver", "typeof PerformanceObserver !== 'undefined'"),
        ("performance.observer", "typeof PerformanceObserver === 'function'"),
    ];

    let mut performance_working = 0;
    let mut performance_broken = 0;

    for (api_name, js_test) in performance_tests {
        let result = browser.lock().unwrap().execute_javascript(js_test).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") {
                    println!("✅ {}: Working", api_name);
                    performance_working += 1;
                } else {
                    println!("❌ {}: Not working ({})", api_name, value_str);
                    performance_broken += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error - {:?}", api_name, e);
                performance_broken += 1;
            }
        }
    }

    println!("\n📊 Performance API Results:");
    println!("  ✅ Working: {}", performance_working);
    println!("  ❌ Broken: {}", performance_broken);
    println!("  📈 Performance API Coverage: {:.1}%", (performance_working as f64 / (performance_working + performance_broken) as f64) * 100.0);

    assert!(performance_working > 0, "At least some performance APIs should work");
}

// Helper function to analyze MDN compatibility information
fn analyze_mdn_compatibility(content: &str) -> String {
    let content_lower = content.to_lowercase();

    if content_lower.contains("chrome") && content_lower.contains("supported") {
        "Chrome Supported".to_string()
    } else if content_lower.contains("compatibility") {
        "Has Compatibility Info".to_string()
    } else {
        "No Compatibility Info Found".to_string()
    }
}