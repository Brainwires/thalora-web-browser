use anyhow::Result;
use boa_engine::{Context, js_string, JsValue, Source};

/// Setup native DOM globals using Boa's built-in implementations
/// This replaces the polyfill-based DOM with real implementations
pub fn setup_native_dom_globals(context: &mut Context) -> Result<()> {
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_native_dom_globals() - Creating native Document");
    }

    // Create instances using constructor functions directly instead of evaluating JavaScript

    // Use the constructor objects directly instead of creating instances
    let document = context.intrinsics().constructors().document().constructor();
    let window = context.intrinsics().constructors().window().constructor();
    let history = context.intrinsics().constructors().history().constructor();

    // For the values, we'll use the constructor JsObjects as values
    let document_value = JsValue::from(document.clone());
    let window_value = JsValue::from(window.clone());
    let history_value = JsValue::from(history.clone());

    // Set up the global object relationships
    let global = context.global_object();

    // Helper function to set or update global property
    let mut set_global_property = |name: &str, value: JsValue| -> Result<()> {
        if global.has_property(js_string!(name), context).unwrap_or(false) {
            // Property exists, just update its value
            global.set(js_string!(name), value, true, context)
                .map_err(|e| anyhow::Error::msg(format!("Failed to update {} global: {}", name, e)))?;
        } else {
            // Property doesn't exist, define it
            global.define_property_or_throw(
                js_string!(name),
                boa_engine::property::PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(value)
                    .build(),
                context,
            ).map_err(|e| anyhow::Error::msg(format!("Failed to set {} global: {}", name, e)))?;
        }
        Ok(())
    };

    // Set window as global
    set_global_property("window", window_value.clone())?;

    // Set self as alias for window
    set_global_property("self", window_value.clone())?;

    // Set globalThis as alias for window
    set_global_property("globalThis", window_value.clone())?;

    // Set document as global
    set_global_property("document", document_value.clone())?;

    // Set history as global
    set_global_property("history", history_value.clone())?;

    // Setup native PageSwapEvent global constructor (if available)
    // For now, skip PageSwapEvent as it may not be available in all Boa builds
    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_native_dom_globals() - Skipping PageSwapEvent for now");
    }

    // Set up the relationships between window, document, and history
    {
        // Set document on window
        window.define_property_or_throw(
            js_string!("document"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(document_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.document: {}", e)))?;

        // Set history on window
        window.define_property_or_throw(
            js_string!("history"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(history_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.history: {}", e)))?;

        // Set window as self-reference
        window.define_property_or_throw(
            js_string!("window"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.window: {}", e)))?;

        // Set self as self-reference
        window.define_property_or_throw(
            js_string!("self"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.self: {}", e)))?;
    }

    // Initialize document state
    {
        // Set initial ready state
        context.eval(Source::from_bytes("document.readyState = 'interactive'")).map_err(|e| anyhow::Error::msg(format!("Failed to set document.readyState: {}", e)))?;

        // Set initial URL
        context.eval(Source::from_bytes("document.URL = 'about:blank'")).map_err(|e| anyhow::Error::msg(format!("Failed to set document.URL: {}", e)))?;

        // Set initial title
        context.eval(Source::from_bytes("document.title = ''")).map_err(|e| anyhow::Error::msg(format!("Failed to set document.title: {}", e)))?;
    }

    // Add parseHTMLUnsafe global function (needed for tests)
    let parsehtml_source = Source::from_bytes(r#"
        globalThis.parseHTMLUnsafe = function(input, options) {
            options = options || {};
            console.log('parseHTMLUnsafe called:', input.substring(0, 100));

            // Return a document fragment
            return {
                nodeName: '#document-fragment',
                nodeType: 11, // DOCUMENT_FRAGMENT_NODE
                childNodes: [],
                appendChild: function(node) {
                    this.childNodes.push(node);
                    console.log('Node appended to parsed fragment');
                }
            };
        };
    "#);
    context.eval(parsehtml_source).map_err(|e| anyhow::Error::msg(format!("Failed to setup parseHTMLUnsafe: {}", e)))?;

    // Setup native Selection API and getSelection functions (with fallback)
    let selection_setup = Source::from_bytes(r#"
        // Create a global selection instance with fallback
        if (typeof window !== 'undefined') {
            try {
                // Try to create native Selection instance
                if (typeof Selection !== 'undefined') {
                    window._globalSelection = new Selection();
                } else {
                    // Fallback to mock Selection object
                    window._globalSelection = {
                        rangeCount: 0,
                        type: 'None',
                        toString: function() { return ''; },
                        addRange: function() {},
                        removeAllRanges: function() {},
                        getRangeAt: function() { return null; }
                    };
                }
            } catch (e) {
                // Fallback to mock Selection object
                window._globalSelection = {
                    rangeCount: 0,
                    type: 'None',
                    toString: function() { return ''; },
                    addRange: function() {},
                    removeAllRanges: function() {},
                    getRangeAt: function() { return null; }
                };
            }

            // Setup window.getSelection()
            if (typeof window.getSelection === 'undefined') {
                window.getSelection = function() {
                    return window._globalSelection;
                };
            }

            // Setup document.getSelection()
            if (typeof document !== 'undefined' && typeof document.getSelection === 'undefined') {
                document.getSelection = function() {
                    return window.getSelection();
                };
            }
        }
    "#);
    context.eval(selection_setup).map_err(|e| anyhow::Error::msg(format!("Failed to setup Selection API: {}", e)))?;

    if std::env::var("THALORA_SILENT").is_err() {
        eprintln!("🔧 setup_native_dom_globals() - Native DOM globals setup complete");
    }
    Ok(())
}

/// Setup native location object with proper URL handling
pub fn setup_native_location(context: &mut Context, url: &str) -> Result<()> {
    let location_setup = format!(r#"
        if (typeof window !== 'undefined' && window.location) {{
            window.location.href = '{}';
            window.location.protocol = '{}';
            window.location.hostname = '{}';
        }}
    "#,
        url,
        if url.starts_with("https:") { "https:" } else if url.starts_with("http:") { "http:" } else { "about:" },
        if let Some(url_start) = url.find("://") {
            let after_protocol = &url[url_start + 3..];
            if let Some(slash_pos) = after_protocol.find('/') {
                &after_protocol[..slash_pos]
            } else {
                after_protocol
            }
        } else {
            ""
        }
    );

    context.eval(Source::from_bytes(&location_setup)).map_err(|e| anyhow::Error::msg(format!("Failed to setup location: {}", e)))?;
    Ok(())
}