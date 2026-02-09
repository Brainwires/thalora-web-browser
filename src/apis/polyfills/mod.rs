// JavaScript polyfills for browser APIs only
// NOTE: All ES6-ES2023 language features are now natively handled by Boa engine
// NOTE: Console is now handled by Boa's native console implementation
pub mod web_apis;
pub mod syntax_transformer;
pub mod console;

// Modular polyfill components
pub mod performance;
pub mod security;
// DOM and CSS are now natively implemented in Boa engine
pub mod worker;
pub mod chrome_features;
pub mod dynamic_scripts;

// Only experimental/proposal polyfills remain
pub mod es2024_polyfills;
pub mod es2025_experimental;


use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, Source};
// timers API is now natively implemented in Boa engine

/// Setup JavaScript polyfills for browser APIs
/// NOTE: ES6-ES2023 language features are natively handled by Boa engine
/// NOTE: Console is now handled by Boa's native console implementation
pub fn setup_all_polyfills(context: &mut Context) -> Result<()> {

    // Console is now handled by Boa's native runtime console

    // timers (setTimeout/setInterval) are now natively handled by Boa engine

    // Web APIs (fetch, websocket, etc.)
    web_apis::setup_web_apis(context).map_err(|e| anyhow::Error::msg(format!("Web API setup failed: {:?}", e)))?;

    // Only experimental/future proposal polyfills remain
    es2024_polyfills::setup_es2024_polyfills(context).map_err(|e| anyhow::Error::msg(format!("ES2024 setup failed: {:?}", e)))?;

    es2025_experimental::setup_es2025_experimental(context).map_err(|e| anyhow::Error::msg(format!("ES2025 setup failed: {:?}", e)))?;

    // Defensive wrappers for Object static methods — prevents TypeError when
    // SPA frameworks (Vue, React) call Object.keys(null) etc. during rendering.
    context.eval(Source::from_bytes(r#"
    (function() {
        var _keys = Object.keys;
        var _values = Object.values;
        var _entries = Object.entries;
        var _getOwnPropertyNames = Object.getOwnPropertyNames;
        var _getOwnPropertyDescriptors = Object.getOwnPropertyDescriptors;

        Object.keys = function(obj) {
            return (obj === null || obj === undefined) ? [] : _keys(obj);
        };
        Object.values = function(obj) {
            return (obj === null || obj === undefined) ? [] : _values(obj);
        };
        Object.entries = function(obj) {
            return (obj === null || obj === undefined) ? [] : _entries(obj);
        };
        Object.getOwnPropertyNames = function(obj) {
            return (obj === null || obj === undefined) ? [] : _getOwnPropertyNames(obj);
        };
        Object.getOwnPropertyDescriptors = function(obj) {
            return (obj === null || obj === undefined) ? {} : _getOwnPropertyDescriptors(obj);
        };
    })();
    "#)).map_err(|e| anyhow::Error::msg(format!("Object null-safety polyfill failed: {:?}", e)))?;

    Ok(())
}

/// Setup dynamic script execution hooks
/// This should be called AFTER the DOM is fully initialized
pub fn setup_dynamic_script_hooks(context: &mut Context) -> Result<()> {
    dynamic_scripts::setup_dynamic_script_execution(context)
        .map_err(|e| anyhow::Error::msg(format!("Dynamic script hooks setup failed: {:?}", e)))?;
    Ok(())
}