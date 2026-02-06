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
use std::collections::HashMap;
use crate::browser::window_registry;
use crate::browser::window::WindowData;
use crate::dom::element::ElementData;

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

        // Set nodeType as own property (tagName/nodeName come from Element.prototype via dispatch)
        iframe_generic.set(js_string!("nodeType"), 1, false, context)?;

        Ok(iframe_generic.into())
    }
}

/// Internal data for HTMLIFrameElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLIFrameElementData {
    /// Base element data — provides tagName, id, className, attributes, children, etc.
    #[unsafe_ignore_trace]
    pub(crate) element: ElementData,
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
            element: ElementData::with_tag_name("IFRAME".to_string()),
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

    /// Access the embedded ElementData
    pub fn element_data(&self) -> &ElementData {
        &self.element
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

    /// Set an attribute on the iframe element
    pub fn set_attribute(&self, name: &str, value: String) {
        // Handle known iframe-specific attributes with dedicated fields
        match name.to_lowercase().as_str() {
            "src" => *self.src.lock().unwrap() = value.clone(),
            "srcdoc" => *self.srcdoc.lock().unwrap() = value.clone(),
            "name" => *self.name.lock().unwrap() = value.clone(),
            "sandbox" => *self.sandbox.lock().unwrap() = value.clone(),
            "allow" => *self.allow.lock().unwrap() = value.clone(),
            "width" => *self.width.lock().unwrap() = value.clone(),
            "height" => *self.height.lock().unwrap() = value.clone(),
            "loading" => {
                let valid = match value.as_str() {
                    "lazy" => "lazy",
                    _ => "eager",
                };
                *self.loading.lock().unwrap() = valid.to_string();
            }
            _ => {}
        }
        // Delegate all attribute storage to the base ElementData
        self.element.set_attribute(name.to_string(), value);
    }

    /// Get an attribute from the iframe element
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        // For known iframe-specific attrs, return from dedicated fields
        match name.to_lowercase().as_str() {
            "src" => Some(self.src.lock().unwrap().clone()),
            "srcdoc" => Some(self.srcdoc.lock().unwrap().clone()),
            "name" => Some(self.name.lock().unwrap().clone()),
            "sandbox" => Some(self.sandbox.lock().unwrap().clone()),
            "allow" => Some(self.allow.lock().unwrap().clone()),
            "width" => Some(self.width.lock().unwrap().clone()),
            "height" => Some(self.height.lock().unwrap().clone()),
            "loading" => Some(self.loading.lock().unwrap().clone()),
            // For everything else, delegate to ElementData
            _ => self.element.get_attribute(name)
        }
    }

    /// Get the element ID
    pub fn get_id(&self) -> String {
        self.element.get_id()
    }

    /// Set the element ID
    pub fn set_id(&self, id: String) {
        self.element.set_id(id.clone());
        self.element.set_attribute("id".to_string(), id);
    }

    /// Get the element class name
    pub fn get_class_name(&self) -> String {
        self.element.get_class_name()
    }

    /// Set the element class name
    pub fn set_class_name(&self, class_name: String) {
        self.element.set_class_name(class_name.clone());
        self.element.set_attribute("class".to_string(), class_name);
    }

    /// Append a child element
    pub fn append_child(&self, child: JsObject) {
        self.element.append_child(child);
    }

    /// Get children
    pub fn get_children(&self) -> Vec<JsObject> {
        self.element.get_children()
    }

    // =========================================================================
    // Mutex getters for direct field access (used by innerHTML parsing)
    // =========================================================================

    /// Get reference to src mutex for direct access
    pub fn get_src_mutex(&self) -> &Arc<Mutex<String>> {
        &self.src
    }

    /// Get reference to name mutex for direct access
    pub fn get_name_mutex(&self) -> &Arc<Mutex<String>> {
        &self.name
    }

    /// Get reference to width mutex for direct access
    pub fn get_width_mutex(&self) -> &Arc<Mutex<String>> {
        &self.width
    }

    /// Get reference to height mutex for direct access
    pub fn get_height_mutex(&self) -> &Arc<Mutex<String>> {
        &self.height
    }

    /// Get reference to sandbox mutex for direct access
    pub fn get_sandbox_mutex(&self) -> &Arc<Mutex<String>> {
        &self.sandbox
    }

    /// Get reference to allow mutex for direct access
    pub fn get_allow_mutex(&self) -> &Arc<Mutex<String>> {
        &self.allow
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

    // Store the src value
    if let Some(data) = this_obj.downcast_ref::<HTMLIFrameElementData>() {
        let old_src = data.src.lock().unwrap().clone();
        *data.src.lock().unwrap() = src.clone();

        // Only trigger load if src actually changed and is a real URL
        if src != old_src && !src.is_empty() && src != "about:blank" {
            eprintln!("🔲 IFRAME: src changed from '{}' to '{}', triggering load", old_src, src);
            // Trigger content load
            // Note: We need to drop the borrow before calling load_iframe_content
            drop(data);

            // Load the content - errors are logged but don't fail the setter
            if let Err(e) = load_iframe_content(&this_obj.clone(), &src, context) {
                eprintln!("🔲 IFRAME: Load failed: {:?}", e);
            }
        }
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

    // Get the window registry
    let registry = window_registry::get_registry();

    // Get parent window from global context
    let parent_window = context.global_object().get(js_string!("window"), context)?;
    let parent_window_obj = parent_window.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Could not get parent window")
    })?;

    // Get parent's window ID from registry
    // Try WindowData first, then fall back to top-level ID
    let parent_id = if let Some(window_data) = parent_window_obj.downcast_ref::<WindowData>() {
        window_data.get_window_id()
    } else {
        // Not a WindowData - use top-level ID
        registry.get_top_level_id()
    };
    eprintln!("🔲 IFRAME: Parent window ID: {:?}", parent_id);

    // Determine iframe origin from src attribute
    let iframe_origin = iframe_data.src.lock().unwrap().clone();
    let origin = if iframe_origin.is_empty() {
        // If no src, use parent's origin (about:srcdoc)
        if let Some(pid) = parent_id {
            registry.get_origin(pid).unwrap_or_else(|| "about:blank".to_string())
        } else {
            "about:blank".to_string()
        }
    } else {
        extract_origin_from_url(&iframe_origin)
    };
    eprintln!("🔲 IFRAME: Iframe origin: {}", origin);

    // Create isolated Document for iframe
    let document_constructor = context.intrinsics().constructors().document().constructor();
    let iframe_document = crate::dom::document::Document::constructor(
        &document_constructor.clone().into(),
        &[],
        context,
    )?;
    let iframe_document_obj = iframe_document.as_object().unwrap().clone();
    eprintln!("🔲 IFRAME: Created iframe document");

    // Initialize the iframe document with basic HTML structure
    // This ensures getElementsByTagName('head')[0] works
    if let Some(doc_data) = iframe_document_obj.downcast_ref::<crate::dom::document::DocumentData>() {
        doc_data.set_html_content("<!DOCTYPE html><html><head></head><body></body></html>");
        eprintln!("🔲 IFRAME: Initialized iframe document with basic HTML structure");
    }

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

    // Register iframe window in the window registry and store hierarchy info in WindowData
    if let Some(pid) = parent_id {
        let window_id = registry.register_iframe_window(pid, origin.clone());
        eprintln!("🔲 IFRAME: Registered iframe in window registry with ID {}", window_id);

        // Store the window ID, parent window, and frame element in WindowData
        if let Some(window_data) = iframe_window_obj.downcast_ref::<WindowData>() {
            window_data.set_window_id(window_id);
            window_data.set_current_url(if iframe_origin.is_empty() { "about:blank".to_string() } else { iframe_origin });
            // Store parent window reference for window.parent
            window_data.set_parent_window(parent_window_obj.clone());
            // Store frame element reference for window.frameElement
            window_data.set_frame_element(iframe_obj.clone());
            eprintln!("🔲 IFRAME: Set window_id {}, parent_window, and frame_element in WindowData", window_id);
        }
    } else {
        eprintln!("⚠️ IFRAME: Parent window not registered in registry, iframe hierarchy may not work correctly");
        // Still set parent and frame element even without registry entry
        if let Some(window_data) = iframe_window_obj.downcast_ref::<WindowData>() {
            window_data.set_parent_window(parent_window_obj.clone());
            window_data.set_frame_element(iframe_obj.clone());
            eprintln!("🔲 IFRAME: Set parent_window and frame_element in WindowData (no registry entry)");
        }
    }

    // Link window.document to the iframe's document
    iframe_window_obj.set(js_string!("document"), iframe_document_obj.clone(), false, context)?;
    eprintln!("🔲 IFRAME: Linked window.document to iframe document");

    // Store the isolated context in the iframe data
    iframe_data.set_content_document(iframe_document_obj);
    iframe_data.set_content_window(iframe_window_obj);
    eprintln!("🔲 IFRAME: Context stored in iframe data");

    Ok(())
}

/// Extract the origin from a URL string
fn extract_origin_from_url(url: &str) -> String {
    if url.is_empty() || url == "about:blank" {
        return "null".to_string();
    }

    if let Some(scheme_end) = url.find("://") {
        let scheme = &url[..scheme_end];
        let after_scheme = &url[scheme_end + 3..];
        let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
        let host_with_port = &after_scheme[..host_end];
        format!("{}://{}", scheme, host_with_port)
    } else {
        "null".to_string()
    }
}

// ============================================================================
// Iframe Content Loading
// ============================================================================

/// Load content into an iframe from its src URL
/// This fetches the URL, parses the HTML, and sets up the iframe's document
pub fn load_iframe_content(
    iframe_obj: &JsObject,
    url: &str,
    context: &mut Context,
) -> JsResult<()> {
    // Skip loading for about:blank or empty URLs
    if url.is_empty() || url == "about:blank" || url.starts_with("about:") {
        eprintln!("🔲 IFRAME LOAD: Skipping load for special URL: {}", url);
        return Ok(());
    }

    // Skip loading for javascript: URLs (security)
    if url.starts_with("javascript:") {
        eprintln!("🔲 IFRAME LOAD: Skipping javascript: URL for security");
        return Ok(());
    }

    eprintln!("🔲 IFRAME LOAD: Loading content from {}", url);

    // Fetch the content using the blocking HTTP client
    let html_content = match fetch_iframe_url(url) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("🔲 IFRAME LOAD: Failed to fetch {}: {}", url, e);
            // Don't fail - just leave the iframe with empty content
            return Ok(());
        }
    };

    eprintln!("🔲 IFRAME LOAD: Fetched {} bytes from {}", html_content.len(), url);

    // Get the iframe's contentDocument
    let iframe_data = iframe_obj.downcast_ref::<HTMLIFrameElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Not an HTMLIFrameElement")
    })?;

    let content_document = iframe_data.get_content_document().ok_or_else(|| {
        JsNativeError::typ().with_message("Iframe has no contentDocument")
    })?;

    let content_window = iframe_data.get_content_window().ok_or_else(|| {
        JsNativeError::typ().with_message("Iframe has no contentWindow")
    })?;

    // Set the HTML content on the iframe's document
    if let Some(doc_data) = content_document.downcast_ref::<crate::dom::document::DocumentData>() {
        doc_data.set_html_content(&html_content);
        eprintln!("🔲 IFRAME LOAD: Set HTML content on iframe document");
    }

    // Extract and execute scripts from the loaded HTML
    execute_iframe_scripts(&html_content, &content_window, &content_document, context)?;

    // Fire load event on the iframe
    fire_iframe_load_event(iframe_obj, context)?;

    eprintln!("🔲 IFRAME LOAD: Completed loading {}", url);

    Ok(())
}

/// Fetch URL content using blocking HTTP client
#[cfg(feature = "native")]
fn fetch_iframe_url(url: &str) -> Result<String, String> {
    use crate::http_blocking::BlockingClient;
    use std::time::Duration;

    let client = BlockingClient::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    response.text()
        .map_err(|e| format!("Failed to read response body: {}", e))
}

#[cfg(not(feature = "native"))]
fn fetch_iframe_url(_url: &str) -> Result<String, String> {
    // In WASM, we can't make blocking HTTP requests
    Err("Iframe src loading not available in WASM mode".to_string())
}

/// Execute scripts from iframe HTML in the iframe's context
fn execute_iframe_scripts(
    html: &str,
    content_window: &JsObject,
    content_document: &JsObject,
    context: &mut Context,
) -> JsResult<()> {
    // Extract script tags from HTML
    let scripts = extract_scripts_from_html(html);

    if scripts.is_empty() {
        eprintln!("🔲 IFRAME SCRIPTS: No scripts found in iframe content");
        return Ok(());
    }

    eprintln!("🔲 IFRAME SCRIPTS: Found {} scripts to execute", scripts.len());

    for (i, script) in scripts.iter().enumerate() {
        if script.trim().is_empty() {
            continue;
        }

        eprintln!("🔲 IFRAME SCRIPTS: Executing script {} ({} chars)", i + 1, script.len());

        // Wrap the script to execute in the iframe's context
        // This creates a closure where window/document/self refer to the iframe's objects
        let wrapped_script = format!(
            r#"(function(window, document, self, parent, top) {{
                "use strict";
                try {{
                    {}
                }} catch (e) {{
                    console.error("Iframe script error:", e);
                }}
            }}).call(arguments[0], arguments[0], arguments[1], arguments[0], arguments[0].parent, arguments[0].top)"#,
            script
        );

        // Create the function and call it with iframe's window and document
        let func_result = context.eval(boa_engine::Source::from_bytes(&wrapped_script));

        match func_result {
            Ok(func_val) => {
                // The wrapped script is an IIFE, so it executes immediately
                // We need to call it with the iframe's window and document
                if let Some(func) = func_val.as_callable() {
                    let args = vec![
                        content_window.clone().into(),
                        content_document.clone().into(),
                    ];
                    match func.call(&content_window.clone().into(), &args, context) {
                        Ok(_) => eprintln!("🔲 IFRAME SCRIPTS: Script {} executed successfully", i + 1),
                        Err(e) => eprintln!("🔲 IFRAME SCRIPTS: Script {} execution error: {:?}", i + 1, e),
                    }
                }
            }
            Err(e) => {
                eprintln!("🔲 IFRAME SCRIPTS: Script {} parse error: {:?}", i + 1, e);
            }
        }
    }

    Ok(())
}

/// Extract inline script content from HTML
fn extract_scripts_from_html(html: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    let mut search_start = 0;

    while let Some(start) = html[search_start..].find("<script") {
        let abs_start = search_start + start;

        // Find the end of the opening tag
        if let Some(tag_end) = html[abs_start..].find('>') {
            let tag_content = &html[abs_start..abs_start + tag_end];

            // Skip external scripts (they have src attribute)
            if tag_content.contains("src=") {
                // For external scripts, we'd need to fetch them too
                // For now, skip them
                search_start = abs_start + tag_end + 1;
                continue;
            }

            // Find the closing </script> tag
            let content_start = abs_start + tag_end + 1;
            if let Some(close_tag) = html[content_start..].find("</script>") {
                let script_content = &html[content_start..content_start + close_tag];
                if !script_content.trim().is_empty() {
                    scripts.push(script_content.to_string());
                }
                search_start = content_start + close_tag + 9; // 9 = len("</script>")
            } else {
                break;
            }
        } else {
            break;
        }
    }

    scripts
}

/// Fire the load event on an iframe element
fn fire_iframe_load_event(iframe_obj: &JsObject, context: &mut Context) -> JsResult<()> {
    // Create a load event
    let event_constructor = context.intrinsics().constructors().event().constructor();
    let load_event = event_constructor.construct(
        &[js_string!("load").into()],
        None,
        context,
    )?;

    // Dispatch the event on the iframe
    // This would trigger any onload handlers
    if let Some(dispatch_event) = iframe_obj.get(js_string!("dispatchEvent"), context)?.as_callable() {
        let _ = dispatch_event.call(&iframe_obj.clone().into(), &[load_event.into()], context);
    }

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
