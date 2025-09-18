use anyhow::Result;
use boa_engine::{Context, js_string, JsValue, Source};

/// Setup native DOM globals using Boa's built-in implementations
/// This replaces the polyfill-based DOM with real implementations
pub fn setup_native_dom_globals(context: &mut Context) -> Result<()> {
    println!("🔧 setup_native_dom_globals() - Creating native Document");

    // Create a Document instance
    let document_source = Source::from_bytes("new Document()");
    let document = context.eval(document_source).map_err(|e| anyhow::Error::msg(format!("Failed to create Document: {}", e)))?;

    // Create a Window instance
    let window_source = Source::from_bytes("new Window()");
    let window = context.eval(window_source).map_err(|e| anyhow::Error::msg(format!("Failed to create Window: {}", e)))?;

    // Create a History instance
    let history_source = Source::from_bytes("new History()");
    let history = context.eval(history_source).map_err(|e| anyhow::Error::msg(format!("Failed to create History: {}", e)))?;

    // Set up the global object relationships
    let global = context.global_object();

    // Set window as global
    global.define_property_or_throw(
        js_string!("window"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(window.clone())
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set window global: {}", e)))?;

    // Set self as alias for window
    global.define_property_or_throw(
        js_string!("self"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(window.clone())
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set self global: {}", e)))?;

    // Set globalThis as alias for window
    global.define_property_or_throw(
        js_string!("globalThis"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(window.clone())
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set globalThis global: {}", e)))?;

    // Set document as global
    global.define_property_or_throw(
        js_string!("document"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(document.clone())
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set document global: {}", e)))?;

    // Set history as global
    global.define_property_or_throw(
        js_string!("history"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(history.clone())
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set history global: {}", e)))?;

    // Setup native PageSwapEvent global constructor
    let pageswap_event_source = Source::from_bytes("PageSwapEvent");
    let pageswap_event_constructor = context.eval(pageswap_event_source).map_err(|e| anyhow::Error::msg(format!("Failed to get PageSwapEvent constructor: {}", e)))?;

    global.define_property_or_throw(
        js_string!("PageSwapEvent"),
        boa_engine::property::PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(pageswap_event_constructor)
            .build(),
        context,
    ).map_err(|e| anyhow::Error::msg(format!("Failed to set PageSwapEvent global: {}", e)))?;

    // Set up the relationships between window, document, and history
    if let Some(window_obj) = window.as_object() {
        // Set document on window
        window_obj.define_property_or_throw(
            js_string!("document"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(document.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.document: {}", e)))?;

        // Set history on window
        window_obj.define_property_or_throw(
            js_string!("history"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(history.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.history: {}", e)))?;

        // Set window as self-reference
        window_obj.define_property_or_throw(
            js_string!("window"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.window: {}", e)))?;

        // Set self as self-reference
        window_obj.define_property_or_throw(
            js_string!("self"),
            boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.self: {}", e)))?;
    }

    // Initialize document state
    if let Some(document_obj) = document.as_object() {
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

    // Setup native Selection API and getSelection functions
    let selection_setup = Source::from_bytes(r#"
        // Create a global selection instance
        if (typeof window !== 'undefined') {
            window._globalSelection = new Selection();

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

    println!("🔧 setup_native_dom_globals() - Native DOM globals setup complete");
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