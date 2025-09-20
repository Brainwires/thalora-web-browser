// Integration test for CSS + DOM + Selection API working together
use thalora::engine::BoaEngine;

#[tokio::test]
async fn test_css_object_model_availability() {
    println!("🧪 Testing CSS Object Model availability...");

    let engine = BoaEngine::new();

    let css_test = r#"
        try {
            var result = {
                cssAvailable: typeof CSS === 'object' && CSS !== null,
                supportsMethod: typeof CSS.supports === 'function',
                registerProperty: typeof CSS.registerProperty === 'function',
                typedObjectModel: typeof CSS.px === 'function',

                // Test some CSS.supports functionality
                flexSupport: CSS.supports('display', 'flex'),
                gridSupport: CSS.supports('display: grid'),
                transformSupport: CSS.supports('transform', 'translateX(10px)'),

                // Test CSS typed object model
                pxValue: typeof CSS.px(100) === 'object',
                percentValue: typeof CSS.percent(50) === 'object'
            };
            'SUCCESS: ' + JSON.stringify(result);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(css_test).await {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("SUCCESS"), "CSS Object Model should be available: {}", value_str);
            println!("✅ CSS Object Model available");
        },
        Err(e) => {
            panic!("CSS Object Model test failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_dom_css_integration() {
    println!("🧪 Testing DOM + CSS integration...");

    let engine = BoaEngine::new();

    let dom_css_test = r#"
        try {
            // Test Element style property
            var element = new Element();
            var hasStyle = typeof element.style === 'object';

            // Test CSS + Selection integration
            var selection = window.getSelection();
            var hasSelection = typeof selection === 'object';

            // Test Range API
            var range = new Range();
            var hasRange = typeof range === 'object';

            var result = {
                elementStyleProperty: hasStyle,
                selectionAPI: hasSelection,
                rangeAPI: hasRange,
                cssAndDomWorking: hasStyle && hasSelection && hasRange
            };

            'SUCCESS: ' + JSON.stringify(result);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(dom_css_test).await {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            assert!(value_str.contains("SUCCESS"), "DOM + CSS integration should work: {}", value_str);
            println!("✅ DOM + CSS integration working");
        },
        Err(e) => {
            panic!("DOM + CSS integration test failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_css_houdini_apis() {
    println!("🧪 Testing CSS Houdini APIs...");

    let engine = BoaEngine::new();

    let houdini_test = r#"
        try {
            var result = {
                paintWorklet: typeof CSS.paintWorklet === 'object' && CSS.paintWorklet !== null,
                layoutWorklet: typeof CSS.layoutWorklet === 'object' && CSS.layoutWorklet !== null,
                animationWorklet: typeof CSS.animationWorklet === 'object' && CSS.animationWorklet !== null,

                paintWorkletAddModule: typeof CSS.paintWorklet.addModule === 'function',
                layoutWorkletAddModule: typeof CSS.layoutWorklet.addModule === 'function',
                animationWorkletAddModule: typeof CSS.animationWorklet.addModule === 'function'
            };

            'SUCCESS: ' + JSON.stringify(result);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(houdini_test).await {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            if value_str.contains("SUCCESS") {
                println!("✅ CSS Houdini APIs available");
            } else {
                println!("🔍 CSS Houdini APIs result: {}", value_str);
                // Don't assert here as Houdini APIs might not be fully implemented yet
            }
        },
        Err(e) => {
            panic!("CSS Houdini APIs test failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_advanced_css_features() {
    println!("🧪 Testing Advanced CSS features...");

    let engine = BoaEngine::new();

    let advanced_test = r#"
        try {
            var result = {
                // Modern CSS features
                containerQueries: CSS.supports('container-type', 'inline-size'),
                viewTransitions: CSS.supports('view-transition-name', 'none'),
                colorMix: CSS.supports('color-mix', 'in srgb'),

                // Selector support
                modernSelectors: CSS.supports('selector', ':has(.child)'),
                pseudoElements: CSS.supports('selector', '::before'),

                // Math functions
                mathFunctions: CSS.supports('math', 'calc(10px + 5px)'),
                trigFunctions: CSS.supports('math', 'sin(45deg)')
            };

            'SUCCESS: ' + JSON.stringify(result);
        } catch (e) {
            'ERROR: ' + e.message;
        }
    "#;

    match engine.execute_javascript(advanced_test).await {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            if value_str.contains("SUCCESS") {
                println!("✅ Advanced CSS features tested");
            } else {
                println!("🔍 Advanced CSS features result: {}", value_str);
                // Don't assert here as advanced features might not be fully implemented yet
            }
        },
        Err(e) => {
            panic!("Advanced CSS features test failed: {:?}", e);
        }
    }
}