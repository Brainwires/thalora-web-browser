//! HTMLIFrameElement implementation for Boa
//!
//! Implements the HTMLIFrameElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// JavaScript `HTMLIFrameElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLIFrameElement;

impl IntrinsicObject for HTMLIFrameElement {
    fn init(realm: &Realm) {
        // src getter/setter
        let src_getter = BuiltInBuilder::callable(realm, get_src)
            .name(js_string!("get src"))
            .build();
        let src_setter = BuiltInBuilder::callable(realm, set_src)
            .name(js_string!("set src"))
            .build();

        // srcdoc getter/setter
        let srcdoc_getter = BuiltInBuilder::callable(realm, get_srcdoc)
            .name(js_string!("get srcdoc"))
            .build();
        let srcdoc_setter = BuiltInBuilder::callable(realm, set_srcdoc)
            .name(js_string!("set srcdoc"))
            .build();

        // name getter/setter
        let name_getter = BuiltInBuilder::callable(realm, get_name)
            .name(js_string!("get name"))
            .build();
        let name_setter = BuiltInBuilder::callable(realm, set_name)
            .name(js_string!("set name"))
            .build();

        // sandbox getter/setter
        let sandbox_getter = BuiltInBuilder::callable(realm, get_sandbox)
            .name(js_string!("get sandbox"))
            .build();
        let sandbox_setter = BuiltInBuilder::callable(realm, set_sandbox)
            .name(js_string!("set sandbox"))
            .build();

        // allow getter/setter
        let allow_getter = BuiltInBuilder::callable(realm, get_allow)
            .name(js_string!("get allow"))
            .build();
        let allow_setter = BuiltInBuilder::callable(realm, set_allow)
            .name(js_string!("set allow"))
            .build();

        // width getter/setter
        let width_getter = BuiltInBuilder::callable(realm, get_width)
            .name(js_string!("get width"))
            .build();
        let width_setter = BuiltInBuilder::callable(realm, set_width)
            .name(js_string!("set width"))
            .build();

        // height getter/setter
        let height_getter = BuiltInBuilder::callable(realm, get_height)
            .name(js_string!("get height"))
            .build();
        let height_setter = BuiltInBuilder::callable(realm, set_height)
            .name(js_string!("set height"))
            .build();

        // loading getter/setter (lazy loading attribute)
        let loading_getter = BuiltInBuilder::callable(realm, get_loading)
            .name(js_string!("get loading"))
            .build();
        let loading_setter = BuiltInBuilder::callable(realm, set_loading)
            .name(js_string!("set loading"))
            .build();

        // contentDocument getter (read-only)
        let content_document_getter = BuiltInBuilder::callable(realm, get_content_document)
            .name(js_string!("get contentDocument"))
            .build();

        // contentWindow getter (read-only)
        let content_window_getter = BuiltInBuilder::callable(realm, get_content_window)
            .name(js_string!("get contentWindow"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Inherit from HTMLElement prototype
            .inherits(Some(realm.intrinsics().constructors().html_element().prototype()))
            .accessor(
                js_string!("src"),
                Some(src_getter),
                Some(src_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("srcdoc"),
                Some(srcdoc_getter),
                Some(srcdoc_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("name"),
                Some(name_getter),
                Some(name_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("sandbox"),
                Some(sandbox_getter),
                Some(sandbox_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("allow"),
                Some(allow_getter),
                Some(allow_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("width"),
                Some(width_getter),
                Some(width_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("height"),
                Some(height_getter),
                Some(height_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("loading"),
                Some(loading_getter),
                Some(loading_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("contentDocument"),
                Some(content_document_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("contentWindow"),
                Some(content_window_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLIFrameElement {
    const NAME: JsString = StaticJsStrings::HTML_IFRAME_ELEMENT;
}

impl BuiltInConstructor for HTMLIFrameElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_iframe_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLIFrameElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_iframe_element,
            context,
        )?;

        let iframe_data = HTMLIFrameElementData::new();
        let iframe_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            iframe_data,
        );

        let iframe_generic = iframe_obj.upcast();

        // Set Element interface properties
        iframe_generic.set(js_string!("tagName"), js_string!("IFRAME"), false, context)?;
        iframe_generic.set(js_string!("nodeName"), js_string!("IFRAME"), false, context)?;
        iframe_generic.set(js_string!("nodeType"), 1, false, context)?;

        Ok(iframe_generic.into())
    }
}

/// Internal data for HTMLIFrameElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLIFrameElementData {
    /// URL to load in the iframe
    #[unsafe_ignore_trace]
    src: Arc<Mutex<String>>,
    /// Inline HTML content for the iframe
    #[unsafe_ignore_trace]
    srcdoc: Arc<Mutex<String>>,
    /// Name for targeting
    #[unsafe_ignore_trace]
    name: Arc<Mutex<String>>,
    /// Sandbox restrictions
    #[unsafe_ignore_trace]
    sandbox: Arc<Mutex<String>>,
    /// Feature policy
    #[unsafe_ignore_trace]
    allow: Arc<Mutex<String>>,
    /// Width attribute
    #[unsafe_ignore_trace]
    width: Arc<Mutex<String>>,
    /// Height attribute
    #[unsafe_ignore_trace]
    height: Arc<Mutex<String>>,
    /// Loading strategy (eager/lazy)
    #[unsafe_ignore_trace]
    loading: Arc<Mutex<String>>,
    /// Isolated document for the iframe's content
    #[unsafe_ignore_trace]
    content_document: Arc<Mutex<Option<JsObject>>>,
    /// Isolated window for the iframe's browsing context
    #[unsafe_ignore_trace]
    content_window: Arc<Mutex<Option<JsObject>>>,
}

impl HTMLIFrameElementData {
    pub fn new() -> Self {
        Self {
            src: Arc::new(Mutex::new(String::new())),
            srcdoc: Arc::new(Mutex::new(String::new())),
            name: Arc::new(Mutex::new(String::new())),
            sandbox: Arc::new(Mutex::new(String::new())),
            allow: Arc::new(Mutex::new(String::new())),
            width: Arc::new(Mutex::new(String::new())),
            height: Arc::new(Mutex::new(String::new())),
            loading: Arc::new(Mutex::new("eager".to_string())),
            content_document: Arc::new(Mutex::new(None)),
            content_window: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the content document (isolated Document for this iframe)
    pub fn set_content_document(&self, doc: JsObject) {
        *self.content_document.lock().unwrap() = Some(doc);
    }

    /// Get the content document
    pub fn get_content_document(&self) -> Option<JsObject> {
        self.content_document.lock().unwrap().clone()
    }

    /// Set the content window (isolated Window for this iframe)
    pub fn set_content_window(&self, window: JsObject) {
        *self.content_window.lock().unwrap() = Some(window);
    }

    /// Get the content window
    pub fn get_content_window(&self) -> Option<JsObject> {
        self.content_window.lock().unwrap().clone()
    }
}

// ============================================================================
// Property getters and setters
// ============================================================================

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.src called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.src.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.src called on non-object")
    })?;

    let src = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.src.lock().unwrap() = src;
    }

    Ok(JsValue::undefined())
}

fn get_srcdoc(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.srcdoc called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.srcdoc.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_srcdoc(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.srcdoc called on non-object")
    })?;

    let srcdoc = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.srcdoc.lock().unwrap() = srcdoc;
    }

    Ok(JsValue::undefined())
}

fn get_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.name called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.name.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.name called on non-object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.name.lock().unwrap() = name;
    }

    Ok(JsValue::undefined())
}

fn get_sandbox(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.sandbox called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.sandbox.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_sandbox(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.sandbox called on non-object")
    })?;

    let sandbox = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.sandbox.lock().unwrap() = sandbox;
    }

    Ok(JsValue::undefined())
}

fn get_allow(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.allow called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.allow.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_allow(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.allow called on non-object")
    })?;

    let allow = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.allow.lock().unwrap() = allow;
    }

    Ok(JsValue::undefined())
}

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.width called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.width.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_width(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.width called on non-object")
    })?;

    let width = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.width.lock().unwrap() = width;
    }

    Ok(JsValue::undefined())
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.height called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.height.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_height(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.height called on non-object")
    })?;

    let height = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.height.lock().unwrap() = height;
    }

    Ok(JsValue::undefined())
}

fn get_loading(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.loading called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(js_string!(data.loading.lock().unwrap().clone()).into())
    } else {
        Ok(js_string!("eager").into())
    }
}

fn set_loading(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.loading called on non-object")
    })?;

    let loading = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Validate loading attribute (only "eager" or "lazy" are valid)
    let valid_loading = match loading.as_str() {
        "lazy" => "lazy",
        _ => "eager", // Default to eager for invalid values
    };

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        *data.loading.lock().unwrap() = valid_loading.to_string();
    }

    Ok(JsValue::undefined())
}

// ============================================================================
// Content document and window getters
// ============================================================================

fn get_content_document(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.contentDocument called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        if let Some(doc) = data.get_content_document() {
            Ok(doc.into())
        } else {
            // Return null if no document is available (cross-origin or not yet loaded)
            Ok(JsValue::null())
        }
    } else {
        Ok(JsValue::null())
    }
}

fn get_content_window(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLIFrameElement.prototype.contentWindow called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        if let Some(window) = data.get_content_window() {
            Ok(window.into())
        } else {
            // Return null if no window is available
            Ok(JsValue::null())
        }
    } else {
        Ok(JsValue::null())
    }
}

/// Initialize the isolated browsing context for an iframe
/// This creates a fresh Document and Window for the iframe's content
pub fn initialize_iframe_context(
    iframe_obj: &JsObject,
    context: &mut Context,
) -> JsResult<()> {
    eprintln!("🔲 IFRAME: Initializing iframe context...");

    let iframe_data = iframe_obj.downcast_ref::<HTMLIFrameElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Not an HTMLIFrameElement")
    })?;

    // Create isolated Document for iframe
    let document_constructor = context.intrinsics().constructors().document().constructor();
    let iframe_document = crate::dom::document::Document::constructor(
        &document_constructor.clone().into(),
        &[],
        context,
    )?;
    let iframe_document_obj = iframe_document.as_object().unwrap().clone();
    eprintln!("🔲 IFRAME: Created iframe document");

    // Check if createElement method exists on the prototype
    if let Ok(create_element) = iframe_document_obj.get(js_string!("createElement"), context) {
        eprintln!("🔲 IFRAME: iframe document.createElement = {:?}", create_element.get_type());
    } else {
        eprintln!("🔲 IFRAME: iframe document.createElement NOT FOUND!");
    }

    // Create isolated Window for iframe
    let window_constructor = context.intrinsics().constructors().window().constructor();
    let iframe_window = crate::browser::window::Window::constructor(
        &window_constructor.clone().into(),
        &[],
        context,
    )?;
    let iframe_window_obj = iframe_window.as_object().unwrap().clone();
    eprintln!("🔲 IFRAME: Created iframe window");

    // Link window.document to the iframe's document
    iframe_window_obj.set(js_string!("document"), iframe_document_obj.clone(), false, context)?;
    eprintln!("🔲 IFRAME: Linked window.document to iframe document");

    // Store the isolated context in the iframe data
    iframe_data.set_content_document(iframe_document_obj);
    iframe_data.set_content_window(iframe_window_obj);
    eprintln!("🔲 IFRAME: Context stored in iframe data");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_html_iframe_element_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof HTMLIFrameElement === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_iframe_element_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const iframe = new HTMLIFrameElement();
            iframe.tagName === 'IFRAME';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_iframe_element_src() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const iframe = new HTMLIFrameElement();
            iframe.src = 'https://example.com';
            iframe.src === 'https://example.com';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_iframe_element_loading() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const iframe = new HTMLIFrameElement();
            iframe.loading = 'lazy';
            iframe.loading === 'lazy';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
