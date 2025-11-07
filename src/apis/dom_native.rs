use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, js_string, JsValue, JsObject, Source, JsResult, JsArgs};
use thalora_browser_apis::boa_engine::builtins::BuiltInBuilder;

/// Setup native DOM globals using Boa's built-in implementations
/// This replaces the polyfill-based DOM with real implementations
pub fn setup_native_dom_globals(context: &mut Context) -> Result<()> {
    eprintln!("🔍 DEBUG: setup_native_dom_globals called!");
    eprintln!("🔍 DEBUG: Context has intrinsics available");

    // Initialize Console API first (needed for Google's JavaScript)
    eprintln!("🔍 DEBUG: Initializing Console API");
    thalora_browser_apis::console::console::Console::init(context);
    eprintln!("🔍 DEBUG: Console API initialized successfully");

    // Initialize Timers API (setTimeout, setInterval - needed for Google's JavaScript)
    eprintln!("🔍 DEBUG: Initializing Timers API");
    thalora_browser_apis::timers::timers::Timers::init(context);
    eprintln!("🔍 DEBUG: Timers API initialized successfully");

    // Create instances using constructor functions directly instead of evaluating JavaScript

    // Get the constructor functions
    let document_constructor = context.intrinsics().constructors().document().constructor();
    let window_constructor = context.intrinsics().constructors().window().constructor();
    let history_constructor = context.intrinsics().constructors().history().constructor();

    // Also expose other constructors (Element, Range, Selection) so JS tests and
    // polyfills can reference the constructor names (e.g. `typeof Element`) even
    // before instances are created.
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let range_constructor = context.intrinsics().constructors().range().constructor();
    let selection_constructor = context.intrinsics().constructors().selection().constructor();

    // Call the constructors to create instances using construct
    let document_obj = document_constructor.construct(&[], None, context).map_err(|e| anyhow::Error::msg(format!("Failed to create Document instance: {}", e)))?;
    let window_obj = window_constructor.construct(&[], None, context).map_err(|e| anyhow::Error::msg(format!("Failed to create Window instance: {}", e)))?;
    let history_obj = history_constructor.construct(&[], None, context).map_err(|e| anyhow::Error::msg(format!("Failed to create History instance: {}", e)))?;

    // Convert to JsValue for setting as globals
    let document_value = JsValue::from(document_obj.clone());
    let window_value = JsValue::from(window_obj.clone());
    let history_value = JsValue::from(history_obj.clone());

    // Set up the global object relationships
    let global = context.global_object();

    // Helper function to set or update global property with better error reporting
    fn set_global_property_helper(global: &JsObject, name: &str, value: JsValue, context: &mut Context) -> Result<()> {
        eprintln!("🔍 DEBUG: Setting global property '{}' with value type: {:?}", name, value.get_type());

        let result = if global.has_property(js_string!(name), context).unwrap_or(false) {
            // Property exists, just update its value
            eprintln!("🔍 DEBUG: Property '{}' exists, updating...", name);
            global.set(js_string!(name), value, true, context)
                .map_err(|e| anyhow::Error::msg(format!("Failed to update {} global: {}", name, e)))
        } else {
            // Property doesn't exist, define it
            eprintln!("🔍 DEBUG: Property '{}' doesn't exist, defining...", name);
            global.define_property_or_throw(
                js_string!(name),
                thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(value)
                    .build(),
                context,
            ).map_err(|e| anyhow::Error::msg(format!("Failed to set {} global: {}", name, e)))
        };

        match result {
            Ok(_) => {
                eprintln!("🔍 DEBUG: Successfully set global property '{}'", name);
                Ok(())
            }
            Err(e) => {
                eprintln!("🔍 DEBUG: Failed to set global property '{}': {}", name, e);
                Err(e)
            }
        }
    }

    // Set window as global
    set_global_property_helper(&global, "window", window_value.clone(), context)?;

    // Set self as alias for window
    set_global_property_helper(&global, "self", window_value.clone(), context)?;

    // Set globalThis as alias for window
    set_global_property_helper(&global, "globalThis", window_value.clone(), context)?;

    // Set document as global
    set_global_property_helper(&global, "document", document_value.clone(), context)?;

    // Set history as global
    set_global_property_helper(&global, "history", history_value.clone(), context)?;

    // Expose constructor functions on the global object (Document, Window, History,
    // Element, Range, Selection) so scripts can access constructors and prototypes
    // directly (e.g. `Document.parseHTMLUnsafe`, `Element.prototype`).
    set_global_property_helper(&global, "Document", JsValue::from(document_constructor.clone()), context)?;
    set_global_property_helper(&global, "Window", JsValue::from(window_constructor.clone()), context)?;
    set_global_property_helper(&global, "History", JsValue::from(history_constructor.clone()), context)?;
    set_global_property_helper(&global, "Element", JsValue::from(element_constructor.clone()), context)?;
    set_global_property_helper(&global, "Range", JsValue::from(range_constructor.clone()), context)?;
    set_global_property_helper(&global, "Selection", JsValue::from(selection_constructor.clone()), context)?;

    // Setup native PageSwapEvent global constructor
    let pageswap_event_constructor = context.intrinsics().constructors().pageswap_event().constructor();
    set_global_property_helper(&global, "PageSwapEvent", JsValue::from(pageswap_event_constructor.clone()), context)?;

    // Set up the relationships between window, document, and history
    {
        // Set document on window
        window_obj.define_property_or_throw(
            js_string!("document"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(document_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.document: {}", e)))?;

        // Set history on window
        window_obj.define_property_or_throw(
            js_string!("history"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(history_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.history: {}", e)))?;

        // Set window as self-reference
        window_obj.define_property_or_throw(
            js_string!("window"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.window: {}", e)))?;

        // Set self as self-reference
        window_obj.define_property_or_throw(
            js_string!("self"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(window_value.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.self: {}", e)))?;

        // Add window.chrome object for Google detection bypass
        // Chrome/Chromium browsers have this object, headless browsers often don't
        let chrome_obj = thalora_browser_apis::boa_engine::object::ObjectInitializer::new(context)
            .property(js_string!("runtime"), JsValue::undefined(), thalora_browser_apis::boa_engine::property::Attribute::all())
            .build();

        window_obj.define_property_or_throw(
            js_string!("chrome"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(chrome_obj)
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.chrome: {}", e)))?;

        // Add window.outerWidth and outerHeight - viewport dimensions for legitimate browser
        window_obj.define_property_or_throw(
            js_string!("outerWidth"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(1920) // Standard desktop width
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.outerWidth: {}", e)))?;

        window_obj.define_property_or_throw(
            js_string!("outerHeight"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(1080) // Standard desktop height
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.outerHeight: {}", e)))?;

        // Add window.innerWidth and innerHeight - content area dimensions
        window_obj.define_property_or_throw(
            js_string!("innerWidth"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(1920)
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.innerWidth: {}", e)))?;

        window_obj.define_property_or_throw(
            js_string!("innerHeight"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(1080)
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.innerHeight: {}", e)))?;

        // Add screen object - display information for legitimate browser
        let screen_obj = thalora_browser_apis::boa_engine::object::ObjectInitializer::new(context)
            .property(js_string!("width"), 1920, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .property(js_string!("height"), 1080, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .property(js_string!("availWidth"), 1920, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .property(js_string!("availHeight"), 1040, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .property(js_string!("colorDepth"), 24, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .property(js_string!("pixelDepth"), 24, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
            .build();

        window_obj.define_property_or_throw(
            js_string!("screen"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(screen_obj.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.screen: {}", e)))?;

        // Also expose screen globally
        set_global_property_helper(&global, "screen", JsValue::from(screen_obj), context)?;

        // Add Image constructor - HTMLImageElement constructor for legitimate browser
        let image_constructor = BuiltInBuilder::callable(context.realm(), |_this: &JsValue, args: &[JsValue], context: &mut Context| -> JsResult<JsValue> {
            eprintln!("🖼️ Image constructor called with {} args", args.len());

            // Create a basic object that represents an image
            let img_obj = thalora_browser_apis::boa_engine::object::ObjectInitializer::new(context)
                .property(js_string!("tagName"), js_string!("IMG"), thalora_browser_apis::boa_engine::property::Attribute::READONLY)
                .property(js_string!("src"), js_string!(""), thalora_browser_apis::boa_engine::property::Attribute::all())
                .property(js_string!("alt"), js_string!(""), thalora_browser_apis::boa_engine::property::Attribute::all())
                .property(js_string!("width"), 0, thalora_browser_apis::boa_engine::property::Attribute::all())
                .property(js_string!("height"), 0, thalora_browser_apis::boa_engine::property::Attribute::all())
                .property(js_string!("complete"), false, thalora_browser_apis::boa_engine::property::Attribute::all())
                .property(js_string!("naturalWidth"), 0, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
                .property(js_string!("naturalHeight"), 0, thalora_browser_apis::boa_engine::property::Attribute::READONLY)
                .build();

            // If width/height arguments provided, set them
            if let Some(width_arg) = args.get(0) {
                if let Some(width) = width_arg.as_number() {
                    img_obj.set(js_string!("width"), width as i32, false, context)?;
                }
            }
            if let Some(height_arg) = args.get(1) {
                if let Some(height) = height_arg.as_number() {
                    img_obj.set(js_string!("height"), height as i32, false, context)?;
                }
            }

            Ok(JsValue::from(img_obj))
        })
        .name(js_string!("Image"))
        .build();

        window_obj.define_property_or_throw(
            js_string!("Image"),
            thalora_browser_apis::boa_engine::property::PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(false)
                .writable(true)
                .value(image_constructor.clone())
                .build(),
            context,
        ).map_err(|e| anyhow::Error::msg(format!("Failed to set window.Image: {}", e)))?;

        // Also expose Image globally
        set_global_property_helper(&global, "Image", JsValue::from(image_constructor), context)?;

        // Expose window's EventTarget methods globally (ensuring same function references)
        // This ensures that global addEventListener === window.addEventListener
        if let Ok(add_event_listener_method) = window_obj.get(js_string!("addEventListener"), context) {
            set_global_property_helper(&global, "addEventListener", add_event_listener_method, context)?;
        }
        if let Ok(remove_event_listener_method) = window_obj.get(js_string!("removeEventListener"), context) {
            set_global_property_helper(&global, "removeEventListener", remove_event_listener_method, context)?;
        }
        if let Ok(dispatch_event_method) = window_obj.get(js_string!("dispatchEvent"), context) {
            set_global_property_helper(&global, "dispatchEvent", dispatch_event_method, context)?;
        }
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

    // Also attach parseHTMLUnsafe to Document constructor if available
    let document_parse_setup = Source::from_bytes(r#"
        try {
            if (typeof Document !== 'undefined' && typeof Document.parseHTMLUnsafe === 'undefined') {
                Document.parseHTMLUnsafe = function(input, options) {
                    options = options || {};
                    // Simple parse that returns a fragment-like object
                    var frag = { nodeName: '#document-fragment', nodeType: 11, childNodes: [] };
                    frag.appendChild = function(node) { this.childNodes.push(node); };
                    return frag;
                };
            }

            // Ensure Element.prototype.setHTMLUnsafe exists and sets innerHTML when possible
            if (typeof Element !== 'undefined' && typeof Element.prototype.setHTMLUnsafe === 'undefined') {
                Element.prototype.setHTMLUnsafe = function(html) {
                    try {
                        if (typeof this.innerHTML !== 'undefined') {
                            this.innerHTML = html;
                        } else {
                            // Fallback: set a property for native Element implementations
                            this._setHTMLUnsafeValue = html;
                        }
                        return true;
                    } catch (e) {
                        return false;
                    }
                };
            }
        } catch (e) {
            // ignore
        }
    "#);
    context.eval(document_parse_setup).map_err(|e| anyhow::Error::msg(format!("Failed to setup Document.parseHTMLUnsafe/Element.setHTMLUnsafe: {}", e)))?;

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