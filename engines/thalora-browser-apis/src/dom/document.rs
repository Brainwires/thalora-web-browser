//! Document Web API implementation for Boa
//!
//! Native implementation of Document standard
//! https://dom.spec.whatwg.org/#interface-document

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    NativeFunction,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder}
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JavaScript `Document` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Document;

impl IntrinsicObject for Document {
    fn init(realm: &Realm) {
        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get URL"))
            .build();

        let title_func = BuiltInBuilder::callable(realm, get_title)
            .name(js_string!("get title"))
            .build();

        let title_setter_func = BuiltInBuilder::callable(realm, set_title)
            .name(js_string!("set title"))
            .build();

        let body_func = BuiltInBuilder::callable(realm, get_body)
            .name(js_string!("get body"))
            .build();

        let head_func = BuiltInBuilder::callable(realm, get_head)
            .name(js_string!("get head"))
            .build();

        let document_element_func = BuiltInBuilder::callable(realm, get_document_element)
            .name(js_string!("get documentElement"))
            .build();

        let forms_func = BuiltInBuilder::callable(realm, get_forms)
            .name(js_string!("get forms"))
            .build();

        let images_func = BuiltInBuilder::callable(realm, get_images)
            .name(js_string!("get images"))
            .build();

        let links_func = BuiltInBuilder::callable(realm, get_links)
            .name(js_string!("get links"))
            .build();

        let scripts_func = BuiltInBuilder::callable(realm, get_scripts)
            .name(js_string!("get scripts"))
            .build();

        let cookie_func = BuiltInBuilder::callable(realm, get_cookie)
            .name(js_string!("get cookie"))
            .build();

        let cookie_setter_func = BuiltInBuilder::callable(realm, set_cookie)
            .name(js_string!("set cookie"))
            .build();

        let referrer_func = BuiltInBuilder::callable(realm, get_referrer)
            .name(js_string!("get referrer"))
            .build();

        let domain_func = BuiltInBuilder::callable(realm, get_domain)
            .name(js_string!("get domain"))
            .build();

        let character_set_func = BuiltInBuilder::callable(realm, get_character_set)
            .name(js_string!("get characterSet"))
            .build();

        let content_type_func = BuiltInBuilder::callable(realm, get_content_type)
            .name(js_string!("get contentType"))
            .build();

        let visibility_state_func = BuiltInBuilder::callable(realm, get_visibility_state)
            .name(js_string!("get visibilityState"))
            .build();

        let hidden_func = BuiltInBuilder::callable(realm, get_hidden)
            .name(js_string!("get hidden"))
            .build();

        let active_element_func = BuiltInBuilder::callable(realm, get_active_element)
            .name(js_string!("get activeElement"))
            .build();

        let current_script_func = BuiltInBuilder::callable(realm, get_current_script)
            .name(js_string!("get currentScript"))
            .build();

        let scrolling_element_func = BuiltInBuilder::callable(realm, get_scrolling_element)
            .name(js_string!("get scrollingElement"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Set up prototype chain: Document -> Node -> EventTarget
            .inherits(Some(realm.intrinsics().constructors().node().prototype()))
            .accessor(
                js_string!("readyState"),
                Some(ready_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("URL"),
                Some(url_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("title"),
                Some(title_func),
                Some(title_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("body"),
                Some(body_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("head"),
                Some(head_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("documentElement"),
                Some(document_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("forms"),
                Some(forms_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("images"),
                Some(images_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("links"),
                Some(links_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scripts"),
                Some(scripts_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cookie"),
                Some(cookie_func),
                Some(cookie_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("referrer"),
                Some(referrer_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("domain"),
                Some(domain_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("characterSet"),
                Some(character_set_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("contentType"),
                Some(content_type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("visibilityState"),
                Some(visibility_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hidden"),
                Some(hidden_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("activeElement"),
                Some(active_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("currentScript"),
                Some(current_script_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollingElement"),
                Some(scrolling_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(create_element, js_string!("createElement"), 1)
            .method(create_element_ns, js_string!("createElementNS"), 2)
            .method(create_text_node, js_string!("createTextNode"), 1)
            .method(create_document_fragment, js_string!("createDocumentFragment"), 0)
            .method(create_range, js_string!("createRange"), 0)
            .method(get_element_by_id, js_string!("getElementById"), 1)
            .method(query_selector, js_string!("querySelector"), 1)
            .method(query_selector_all, js_string!("querySelectorAll"), 1)
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(start_view_transition, js_string!("startViewTransition"), 0)
            // New DOM query methods
            .method(get_elements_by_class_name, js_string!("getElementsByClassName"), 1)
            .method(get_elements_by_tag_name, js_string!("getElementsByTagName"), 1)
            .method(get_elements_by_name, js_string!("getElementsByName"), 1)
            .method(create_comment, js_string!("createComment"), 1)
            .method(create_attribute, js_string!("createAttribute"), 1)
            .method(has_focus, js_string!("hasFocus"), 0)
            .method(exec_command, js_string!("execCommand"), 3)
            // DOM Traversal methods
            .method(create_tree_walker, js_string!("createTreeWalker"), 1)
            .method(create_node_iterator, js_string!("createNodeIterator"), 1)
            // CSSOM View methods (used by Cloudflare Turnstile for bot detection)
            .method(element_from_point, js_string!("elementFromPoint"), 2)
            .method(elements_from_point, js_string!("elementsFromPoint"), 2)
            // Scroll methods (Document delegates to Window)
            .method(scroll_to_document, js_string!("scrollTo"), 2)
            .method(scroll_to_document, js_string!("scroll"), 2)  // alias for scrollTo
            // Internal trusted event dispatch (for Cloudflare etc.)
            .method(dispatch_trusted_mouse_event_document, js_string!("__dispatchTrustedMouseEvent"), 3)
            // Static method: Document.parseHTMLUnsafe(html) - Chrome 124+
            .static_method(super::document_parse::parse_html_unsafe, js_string!("parseHTMLUnsafe"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Document {
    const NAME: JsString = StaticJsStrings::DOCUMENT;
}

impl BuiltInConstructor for Document {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 68; // Accessors and methods on prototype (added scrollTo, scroll, __dispatchTrustedMouseEvent)
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::document;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::document,
            context,
        )?;

        let document_data = DocumentData::new();

        let document = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            document_data,
        );

        Ok(document.into())
    }
}

/// Represents a loaded script element with all its attributes
/// This is used to track dynamically loaded scripts that need to be
/// visible in document.scripts and getElementsByTagName("script")
#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub src: Option<String>,
    pub script_type: Option<String>,
    pub async_: bool,
    pub defer: bool,
    pub text: String,
    pub attributes: HashMap<String, String>,
}

impl ScriptEntry {
    pub fn new() -> Self {
        Self {
            src: None,
            script_type: None,
            async_: false,
            defer: false,
            text: String::new(),
            attributes: HashMap::new(),
        }
    }

    /// Create a ScriptEntry with a source URL
    pub fn with_src(src: String) -> Self {
        Self {
            src: Some(src),
            script_type: None,
            async_: false,
            defer: false,
            text: String::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set all attributes from an iterator (typically from HTML parsing)
    pub fn with_attributes<I, K, V>(mut self, attrs: I) -> Self
    where
        I: Iterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in attrs {
            let key_str = key.as_ref().to_string();
            let value_str = value.as_ref().to_string();

            // Also set known fields from attributes
            match key_str.as_str() {
                "src" => self.src = Some(value_str.clone()),
                "type" => self.script_type = Some(value_str.clone()),
                "async" => self.async_ = true,
                "defer" => self.defer = true,
                _ => {}
            }

            self.attributes.insert(key_str, value_str);
        }
        self
    }
}

/// Internal data for Document objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DocumentData {
    #[unsafe_ignore_trace]
    ready_state: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    url: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    title: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    cookie: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    referrer: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    domain: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    character_set: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    content_type: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    elements: Arc<Mutex<HashMap<String, JsObject>>>,
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    #[unsafe_ignore_trace]
    html_content: Arc<Mutex<String>>,
    /// Registry of loaded scripts (both from HTML and dynamically created)
    /// This allows document.scripts and getElementsByTagName("script") to find
    /// scripts that were executed but not statically present in the current HTML
    #[unsafe_ignore_trace]
    loaded_scripts: Arc<Mutex<Vec<ScriptEntry>>>,
}

impl DocumentData {
    fn new() -> Self {
        let doc_data = Self {
            ready_state: Arc::new(Mutex::new("loading".to_string())),
            url: Arc::new(Mutex::new("about:blank".to_string())),
            title: Arc::new(Mutex::new("".to_string())),
            cookie: Arc::new(Mutex::new("".to_string())),
            referrer: Arc::new(Mutex::new("".to_string())),
            domain: Arc::new(Mutex::new("".to_string())),
            character_set: Arc::new(Mutex::new("UTF-8".to_string())),
            content_type: Arc::new(Mutex::new("text/html".to_string())),
            elements: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            html_content: Arc::new(Mutex::new("".to_string())),
            loaded_scripts: Arc::new(Mutex::new(Vec::new())),
        };

        // Set up DOM sync bridge - connect Element changes to Document updates
        use crate::dom::element::GLOBAL_DOM_SYNC;
        let html_content_ref = doc_data.html_content.clone();
        GLOBAL_DOM_SYNC.get_or_init(|| crate::dom::element::DomSync::new())
            .set_updater(Box::new(move |html| {
                *html_content_ref.lock().unwrap() = html.to_string();
            }));

        doc_data
    }

    pub fn set_ready_state(&self, state: &str) {
        *self.ready_state.lock().unwrap() = state.to_string();
    }

    pub fn set_url(&self, url: &str) {
        *self.url.lock().unwrap() = url.to_string();
    }

    pub fn set_title(&self, title: &str) {
        *self.title.lock().unwrap() = title.to_string();
    }

    pub fn set_html_content(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();

        // Process all forms in the HTML and prepare them for DOM access
        self.process_forms_in_html(html);
    }

    pub fn update_html_from_dom(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();
    }

    pub fn get_html_content(&self) -> String {
        self.html_content.lock().unwrap().clone()
    }

    pub fn get_ready_state(&self) -> String {
        self.ready_state.lock().unwrap().clone()
    }

    pub fn get_url(&self) -> String {
        self.url.lock().unwrap().clone()
    }

    pub fn get_title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn add_element(&self, id: String, element: JsObject) {
        self.elements.lock().unwrap().insert(id, element);
    }

    pub fn get_element(&self, id: &str) -> Option<JsObject> {
        self.elements.lock().unwrap().get(id).cloned()
    }

    /// Process all forms in HTML content and prepare elements collections
    /// This ensures that forms accessed via DOM events have proper elements collections
    fn process_forms_in_html(&self, html_content: &str) {
        eprintln!("🔍 DEBUG: process_forms_in_html called with {} characters of HTML", html_content.len());

        // Parse the HTML content to find all forms
        let document = scraper::Html::parse_document(html_content);

        // Find all form elements
        if let Ok(form_selector) = scraper::Selector::parse("form") {
            let form_count = document.select(&form_selector).count();
            eprintln!("🔍 DEBUG: Found {} forms in HTML", form_count);

            for (form_index, form_element) in document.select(&form_selector).enumerate() {
                // Create a unique ID for this form if it doesn't have one
                let form_id = if let Some(id) = form_element.value().attr("id") {
                    id.to_string()
                } else {
                    format!("auto_form_{}", form_index)
                };

                // Store form metadata for later DOM access
                let mut form_inputs = Vec::new();

                // Parse form's inner HTML to find input elements
                let form_inner_html = form_element.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                if let Ok(input_selector) = scraper::Selector::parse("input") {
                    for input_element in form_doc.select(&input_selector) {
                        if let Some(input_name) = input_element.value().attr("name") {
                            let input_value = input_element.value().attr("value").unwrap_or("").to_string();
                            let input_type = input_element.value().attr("type").unwrap_or("text").to_string();

                            form_inputs.push((input_name.to_string(), input_value, input_type));
                        }
                    }
                }

                // Store the form metadata for later JavaScript access
                // We'll use this when DOM queries ask for this form
                self.add_form_metadata(form_id, form_inputs);
            }
        }
    }

    /// Add form metadata that can be used when creating form elements in JavaScript
    fn add_form_metadata(&self, form_id: String, inputs: Vec<(String, String, String)>) {
        // Create an HTMLFormElement with proper elements collection
        use crate::misc::form::{HTMLFormElement, HTMLInputElement, HTMLFormControlsCollection};
        use boa_engine::{Context, object::ObjectInitializer, js_string};

        // For now, store the metadata - we'll need a context to create the actual objects
        // This processing happens at document level so all forms are known before JavaScript queries them
        // TODO: This needs to be enhanced to create actual JavaScript objects when we have a context
        eprintln!("🔍 DEBUG: Found form '{}' with {} inputs", form_id, inputs.len());
        for (name, value, input_type) in &inputs {
            eprintln!("🔍 DEBUG: - Input '{}' = '{}' (type: {})", name, value, input_type);
        }
    }

    pub fn add_event_listener(&self, event_type: String, listener: JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }

    pub fn get_event_listeners(&self, event_type: &str) -> Vec<JsValue> {
        self.event_listeners.lock().unwrap()
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Register a script that has been loaded/executed
    /// This makes the script visible in document.scripts and getElementsByTagName("script")
    pub fn register_script(&self, entry: ScriptEntry) {
        self.loaded_scripts.lock().unwrap().push(entry);
    }

    /// Get all registered loaded scripts
    pub fn get_loaded_scripts(&self) -> Vec<ScriptEntry> {
        self.loaded_scripts.lock().unwrap().clone()
    }

    /// Clear all registered scripts (typically when navigating to a new page)
    pub fn clear_scripts(&self) {
        self.loaded_scripts.lock().unwrap().clear();
    }

    /// Get a reference to the loaded_scripts Arc for sharing with other components
    pub fn get_loaded_scripts_ref(&self) -> Arc<Mutex<Vec<ScriptEntry>>> {
        self.loaded_scripts.clone()
    }
}

/// `Document.prototype.readyState` getter
fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.readyState called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.readyState called on non-Document object")
            })?;
            document.get_ready_state()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.URL` getter
fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.URL called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.URL called on non-Document object")
            })?;
            document.get_url()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.title` getter
fn get_title(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.title called on non-Document object")
            })?;
            document.get_title()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.title` setter
fn set_title(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title setter called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.title setter called on non-Document object")
    })?;

    let title = args.get_or_undefined(0).to_string(context)?;
    document.set_title(&title.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Document.prototype.body` getter
fn get_body(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.body called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.body called on non-Document object")
    })?;

    // Create body element if it doesn't exist
    if let Some(body) = document.get_element("body") {
        Ok(body.into())
    } else {
        // Create a new body element using the Element constructor
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let body_element = element_constructor.construct(&[], None, context)?;

        document.add_element("body".to_string(), body_element.clone());
        Ok(body_element.into())
    }
}

/// `Document.prototype.head` getter
fn get_head(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.head called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.head called on non-Document object")
    })?;

    // Create head element if it doesn't exist
    if let Some(head) = document.get_element("head") {
        Ok(head.into())
    } else {
        // Create a new head element
        let head_element = JsObject::default(context.intrinsics());

        // Add tagName property
        head_element.define_property_or_throw(
            js_string!("tagName"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from("HEAD"))
                .build(),
            context,
        )?;

        document.add_element("head".to_string(), head_element.clone());
        Ok(head_element.into())
    }
}

/// `Document.prototype.createElement(tagName)`
fn create_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.createElement called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.createElement called on non-Document object")
    })?;

    let tag_name = args.get_or_undefined(0).to_string(context)?;
    let tag_name_upper = tag_name.to_std_string_escaped().to_uppercase();

    // Create a proper Element object using Element constructor pattern
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = crate::dom::element::Element::constructor(
        &element_constructor.clone().into(),
        &[],
        context,
    )?;

    // Get the Element object from the JsValue
    let element_obj = element.as_object().unwrap();

    // Add tagName property (this should be done by ElementData, but make it explicit)
    element_obj.define_property_or_throw(
        js_string!("tagName"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(tag_name_upper.as_str()))
            .build(),
        context,
    )?;

    // Set the tag name in the element data
    if let Some(element_data) = element_obj.downcast_ref::<crate::dom::element::ElementData>() {
        element_data.set_tag_name(tag_name_upper.clone());
    }

    // Add style property as empty object
    let style_obj = JsObject::default(context.intrinsics());
    element_obj.define_property_or_throw(
        js_string!("style"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(style_obj)
            .build(),
        context,
    )?;

    // Add Form-specific functionality for <form> elements
    if tag_name_upper == "FORM" {
        // Create elements collection that Google's code expects
        let elements_collection = JsObject::default(context.intrinsics());

        // Add common form controls as properties of elements collection
        // Google often checks for elements like 'q' (search query)
        let q_element = JsObject::default(context.intrinsics());
        q_element.define_property_or_throw(
            js_string!("value"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!(""))
                .build(),
            context,
        )?;

        elements_collection.define_property_or_throw(
            js_string!("q"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(q_element)
                .build(),
            context,
        )?;

        // Add elements collection to form
        element_obj.define_property_or_throw(
            js_string!("elements"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(elements_collection)
                .build(),
            context,
        )?;

        // Add getAttribute method that Google's code uses
        let get_attribute_func = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
            let attr_name = args.get_or_undefined(0).to_string(ctx)?;
            let attr_name_str = attr_name.to_std_string_escaped();

            // Return common attributes that Google checks
            match attr_name_str.as_str() {
                "data-submitfalse" => Ok(JsValue::null()), // Google checks this
                _ => Ok(JsValue::null())
            }
        })
        .name(js_string!("getAttribute"))
        .build();

        element_obj.define_property_or_throw(
            js_string!("getAttribute"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_attribute_func)
                .build(),
            context,
        )?;
    }

    // Add Button-specific functionality for <button> elements
    if tag_name_upper == "BUTTON" {
        // Add button-specific properties
        element_obj.define_property_or_throw(
            js_string!("type"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!("button"))
                .build(),
            context,
        )?;

        element_obj.define_property_or_throw(
            js_string!("value"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(js_string!(""))
                .build(),
            context,
        )?;
    }

    // Add Canvas-specific functionality for <canvas> elements
    if tag_name_upper == "CANVAS" {
        // Add width and height properties with default values
        element_obj.define_property_or_throw(
            js_string!("width"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(300) // Default canvas width
                .build(),
            context,
        )?;

        element_obj.define_property_or_throw(
            js_string!("height"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(150) // Default canvas height
                .build(),
            context,
        )?;

        // Add getContext method
        let get_context_func = BuiltInBuilder::callable(context.realm(), canvas_get_context)
            .name(js_string!("getContext"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("getContext"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_context_func)
                .build(),
            context,
        )?;

        // Add toDataURL method
        let to_data_url_func = BuiltInBuilder::callable(context.realm(), canvas_to_data_url)
            .name(js_string!("toDataURL"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("toDataURL"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(to_data_url_func)
                .build(),
            context,
        )?;
    }

    // Add IFrame-specific functionality for <iframe> elements
    if tag_name_upper == "IFRAME" {
        // Create an HTMLIFrameElement instead of generic Element
        let iframe_constructor = context.intrinsics().constructors().html_iframe_element().constructor();
        let iframe = crate::dom::html_iframe_element::HTMLIFrameElement::constructor(
            &iframe_constructor.clone().into(),
            &[],
            context,
        )?;

        let iframe_obj = iframe.as_object().unwrap();

        // Initialize the iframe's isolated browsing context (contentDocument/contentWindow)
        crate::dom::html_iframe_element::initialize_iframe_context(&iframe_obj, context)?;

        return Ok(iframe);
    }

    // Add Script-specific functionality for <script> elements
    if tag_name_upper == "SCRIPT" {
        // Create an HTMLScriptElement instead of generic Element
        let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
        let script = crate::dom::html_script_element::HTMLScriptElement::constructor(
            &script_constructor.clone().into(),
            &[],
            context,
        )?;

        return Ok(script);
    }

    Ok(element)
}

/// `Document.prototype.createElementNS(namespaceURI, qualifiedName)`
/// Creates an element with the specified namespace URI and qualified name.
/// Used for creating SVG, MathML, and other namespaced elements.
fn create_element_ns(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.createElementNS called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.createElementNS called on non-Document object")
    })?;

    // Get namespace URI (can be null)
    let namespace_uri = args.get_or_undefined(0);
    let namespace_str = if namespace_uri.is_null() || namespace_uri.is_undefined() {
        None
    } else {
        Some(namespace_uri.to_string(context)?.to_std_string_escaped())
    };

    // Get qualified name (required)
    let qualified_name = args.get_or_undefined(1).to_string(context)?;
    let qualified_name_str = qualified_name.to_std_string_escaped();

    // Extract local name (after the colon if there's a prefix)
    let local_name = if let Some(colon_pos) = qualified_name_str.find(':') {
        &qualified_name_str[colon_pos + 1..]
    } else {
        &qualified_name_str
    };

    // Create a proper Element object using Element constructor
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = crate::dom::element::Element::constructor(
        &element_constructor.clone().into(),
        &[],
        context,
    )?;

    let element_obj = element.as_object().unwrap();

    // For SVG namespace, use lowercase tag name; otherwise uppercase
    let is_svg = namespace_str.as_deref() == Some("http://www.w3.org/2000/svg");
    let tag_name = if is_svg {
        local_name.to_string()
    } else {
        local_name.to_uppercase()
    };

    // Set tagName property
    element_obj.define_property_or_throw(
        js_string!("tagName"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(tag_name.as_str()))
            .build(),
        context,
    )?;

    // Set namespace URI if provided
    if let Some(ref ns) = namespace_str {
        element_obj.define_property_or_throw(
            js_string!("namespaceURI"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from(ns.as_str()))
                .build(),
            context,
        )?;
    }

    // Set the element data
    if let Some(element_data) = element_obj.downcast_ref::<crate::dom::element::ElementData>() {
        element_data.set_tag_name(tag_name.clone());
        element_data.set_namespace_uri(namespace_str.clone());
    }

    // Add style property
    let style_obj = JsObject::default(context.intrinsics());
    element_obj.define_property_or_throw(
        js_string!("style"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(style_obj)
            .build(),
        context,
    )?;

    // For SVG elements, add SVG-specific properties
    if is_svg {
        // Add SVGAnimatedLength-like properties for common SVG attributes
        // These return objects with baseVal and animVal properties
        let svg_animated_props = ["width", "height", "x", "y", "cx", "cy", "r", "rx", "ry"];
        for prop in svg_animated_props {
            let animated_length = JsObject::default(context.intrinsics());
            animated_length.define_property_or_throw(
                js_string!("baseVal"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsValue::from(0))
                    .build(),
                context,
            )?;
            animated_length.define_property_or_throw(
                js_string!("animVal"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsValue::from(0))
                    .build(),
                context,
            )?;

            element_obj.define_property_or_throw(
                js_string!(prop),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(animated_length)
                    .build(),
                context,
            )?;
        }

        // Add getBBox method for SVG elements
        let get_bbox_func = BuiltInBuilder::callable(context.realm(), svg_get_bbox)
            .name(js_string!("getBBox"))
            .build();

        element_obj.define_property_or_throw(
            js_string!("getBBox"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(get_bbox_func)
                .build(),
            context,
        )?;
    }

    Ok(element)
}

/// SVG getBBox() implementation - returns bounding box for SVG elements
fn svg_get_bbox(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a DOMRect-like object with x, y, width, height
    let bbox = JsObject::default(context.intrinsics());
    bbox.define_property_or_throw(
        js_string!("x"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("y"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    bbox.define_property_or_throw(
        js_string!("height"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(JsValue::from(0))
            .build(),
        context,
    )?;
    Ok(bbox.into())
}

/// `Document.prototype.createTextNode(data)`
fn create_text_node(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0).to_string(context)?;

    // Create a Text node using the Text constructor
    let text_constructor = context.intrinsics().constructors().text().constructor();
    let text = crate::dom::text::Text::constructor(
        &text_constructor.clone().into(),
        &[data.into()],
        context,
    )?;

    Ok(text)
}

/// `Document.prototype.createDocumentFragment()`
fn create_document_fragment(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a DocumentFragment using the DocumentFragment constructor
    let fragment_constructor = context.intrinsics().constructors().document_fragment().constructor();
    let fragment = crate::dom::document_fragment::DocumentFragment::constructor(
        &fragment_constructor.clone().into(),
        &[],
        context,
    )?;

    Ok(fragment)
}

/// `Document.prototype.createRange()`
fn create_range(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a Range using the Range constructor
    let range_constructor = context.intrinsics().constructors().range().constructor();
    let range = crate::dom::range::Range::constructor(
        &range_constructor.clone().into(),
        &[],
        context,
    )?;

    Ok(range)
}

/// `Document.prototype.getElementById(id)`
fn get_element_by_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementById called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementById called on non-Document object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?;

    if let Some(element) = document.get_element(&id.to_std_string_escaped()) {
        Ok(element.into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Document.prototype.querySelector(selector)`
fn query_selector(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    eprintln!("DEBUG: query_selector called!");

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelector called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.querySelector called on non-Document object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();
    eprintln!("DEBUG: query_selector selector: {}", selector_str);

    // Get the HTML content from the document
    let html_content = document.get_html_content();
    eprintln!("DEBUG: query_selector HTML content length: {}", html_content.len());

    // Use real DOM implementation with scraper library
    if let Some(element) = create_real_element_from_html(context, &selector_str, &html_content)? {
        return Ok(element.into());
    }

    eprintln!("DEBUG: query_selector returning null - no element found");
    Ok(JsValue::null())
}

/// Real DOM element creation using scraper library and actual HTML content
fn create_real_element_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Option<JsObject>> {
    // Use the scraper crate to parse real HTML and find elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        if let Some(element_ref) = document.select(&css_selector).next() {
            // Get real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();
            eprintln!("DEBUG: querySelector creating element with tagName: {}", tag_name);

            // Create a proper Element using ElementData with correct prototype
            // This ensures getters (tagName, className, etc.) work correctly
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_data.set_text_content(text_content);

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_data.set_inner_html(inner_html);

            // Create JsObject with proper prototype chain
            // Use HTMLElement prototype for HTML elements (ensures instanceof HTMLElement works)
            // This ensures methods like dispatchEvent, getBoundingClientRect work correctly
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let typed_element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            // Convert to generic JsObject to use .set() method for form handling
            let element_obj = typed_element_obj.upcast();

            eprintln!("DEBUG: Element created with proper ElementData, tagName: {}", tag_name);

            // Add value property for input elements (needs special handling)
            if tag_name == "INPUT" {
                if let Some(value) = element_ref.value().attr("value") {
                    element_obj.set(js_string!("value"), js_string!(value), false, context)?;
                } else {
                    element_obj.set(js_string!("value"), js_string!(""), false, context)?;
                }

                // Add name property for input elements (needed for form.elements access)
                if let Some(name) = element_ref.value().attr("name") {
                    element_obj.set(js_string!("name"), js_string!(name), false, context)?;
                }
            }

            // Add form-specific functionality for FORM elements from HTML
            if tag_name == "FORM" {
                // Create elements collection
                let elements_collection = context.intrinsics().constructors().object().constructor();

                // Find all input elements within this form using the HTML content
                let form_selector = scraper::Selector::parse("input").unwrap();

                // Parse the inner HTML of this form to find inputs
                let form_inner_html = element_ref.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                for input_element in form_doc.select(&form_selector) {
                    if let Some(input_name) = input_element.value().attr("name") {
                        // Create input element object
                        let input_obj = context.intrinsics().constructors().object().constructor();

                        // Add value property
                        if let Some(input_value) = input_element.value().attr("value") {
                            input_obj.set(js_string!("value"), js_string!(input_value), false, context)?;
                        } else {
                            input_obj.set(js_string!("value"), js_string!(""), false, context)?;
                        }

                        // Add name property
                        input_obj.set(js_string!("name"), js_string!(input_name), false, context)?;

                        // Add input type
                        if let Some(input_type) = input_element.value().attr("type") {
                            input_obj.set(js_string!("type"), js_string!(input_type), false, context)?;
                        } else {
                            input_obj.set(js_string!("type"), js_string!("text"), false, context)?;
                        }

                        // Add this input to the elements collection by name
                        elements_collection.set(js_string!(input_name), input_obj, false, context)?;
                    }
                }

                // Add elements collection to the form
                element_obj.set(js_string!("elements"), elements_collection, false, context)?;

                // Add getAttribute method that Google's code needs
                let get_attribute_func = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
                    let attr_name = args.get_or_undefined(0).to_string(ctx)?;
                    let attr_name_str = attr_name.to_std_string_escaped();

                    // Return common attributes that Google checks
                    match attr_name_str.as_str() {
                        "data-submitfalse" => Ok(JsValue::null()), // Google checks this
                        _ => Ok(JsValue::null())
                    }
                })
                .name(js_string!("getAttribute"))
                .build();

                element_obj.set(js_string!("getAttribute"), get_attribute_func, false, context)?;
            }

            return Ok(Some(element_obj));
        }
    }

    Ok(None)
}

/// Real DOM elements creation using scraper library to find all matching elements
fn create_all_real_elements_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Vec<JsValue>> {
    let mut elements = Vec::new();

    // Use the scraper crate to parse real HTML and find all elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        for element_ref in document.select(&css_selector) {
            // Get real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();

            // Create a proper Element using ElementData with correct prototype
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_data.set_text_content(text_content);

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_data.set_inner_html(inner_html);

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            elements.push(element_obj.upcast().into());
        }
    }

    Ok(elements)
}

/// `Document.prototype.querySelectorAll(selector)`
fn query_selector_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelectorAll called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.querySelectorAll called on non-Document object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Get the HTML content from the document
    let html_content = document.get_html_content();

    // Use real DOM implementation with scraper library to find all matching elements
    let elements = create_all_real_elements_from_html(context, &selector_str, &html_content)?;

    use boa_engine::builtins::array::Array;
    let array = Array::create_array_from_list(elements, context);
    Ok(array.into())
}

/// `Document.prototype.addEventListener(type, listener)`
fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.addEventListener called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.addEventListener called on non-Document object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1).clone();

    document.add_event_listener(event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Document.prototype.removeEventListener(type, listener)`
fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.removeEventListener called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.removeEventListener called on non-Document object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    document.remove_event_listener(&event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Document.prototype.dispatchEvent(event)`
fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.dispatchEvent called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.dispatchEvent called on non-Document object")
    })?;

    let event = args.get_or_undefined(0);

    // Get event type from event object
    if event.is_object() {
        if let Some(event_obj) = event.as_object() {
            if let Ok(type_val) = event_obj.get(js_string!("type"), context) {
                let event_type = type_val.to_string(context)?;
                let listeners = document.get_event_listeners(&event_type.to_std_string_escaped());

                // Call each listener
                for listener in listeners {
                    if listener.is_callable() {
                        let _ = listener.as_callable().unwrap().call(
                            &this_obj.clone().into(),
                            &[event.clone()],
                            context,
                        );
                    }
                }
            }
        }
    }

    Ok(true.into())
}

/// `Document.prototype.startViewTransition(callback)`
fn start_view_transition(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.startViewTransition called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.startViewTransition called on non-Document object")
    })?;

    let callback = args.get_or_undefined(0);

    // Create transition object
    let transition = JsObject::default(context.intrinsics());

    // Add finished property as resolved Promise
    let finished_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    transition.define_property_or_throw(
        js_string!("finished"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(finished_promise)
            .build(),
        context,
    )?;

    // Add ready property as resolved Promise
    let ready_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    transition.define_property_or_throw(
        js_string!("ready"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(ready_promise)
            .build(),
        context,
    )?;

    // Handle callback if provided
    let mut callback_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    if !callback.is_undefined() && callback.is_callable() {
        // Call the callback function
        if let Ok(result) = callback.as_callable()
            .unwrap()
            .call(&JsValue::undefined(), &[], context) {

            // Check if result is a promise
            if result.is_object() {
                if let Some(obj) = result.as_object() {
                    if obj.has_property(js_string!("then"), context)? {
                        callback_promise = result;
                    }
                }
            }
        }
    }

    // Add updateCallbackDone property
    transition.define_property_or_throw(
        js_string!("updateCallbackDone"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(callback_promise)
            .build(),
        context,
    )?;

    // Add skipTransition method
    let skip_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        Ok(JsValue::undefined())
    })
    .name(js_string!("skipTransition"))
    .build();

    transition.define_property_or_throw(
        js_string!("skipTransition"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(skip_function)
            .build(),
        context,
    )?;

    Ok(transition.into())
}

// ============================================================================
// New DOM Query Methods (Phase 6.1)
// ============================================================================

/// `Document.prototype.getElementsByClassName(classNames)`
fn get_elements_by_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByClassName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByClassName called on non-Document object")
    })?;

    let class_names = args.get_or_undefined(0).to_string(context)?;
    let class_names_str = class_names.to_std_string_escaped();

    // Parse class names (space-separated)
    let classes: Vec<&str> = class_names_str.split_whitespace().collect();

    // Get HTML content and parse
    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_fragment(&html_content);

    // Build CSS selector for matching all classes
    let selector_str = classes.iter()
        .map(|c| format!(".{}", c))
        .collect::<Vec<_>>()
        .join("");

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML (including class)
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set innerHTML
            element_data.set_inner_html(element.inner_html());

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    // Set length property
    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.getElementsByTagName(tagName)`
fn get_elements_by_tag_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByTagName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByTagName called on non-Document object")
    })?;

    let tag_name = args.get_or_undefined(0).to_string(context)?;
    let tag_name_str = tag_name.to_std_string_escaped().to_lowercase();

    // Special handling for "script" tag - use proper HTMLScriptElement with full attributes
    if tag_name_str == "script" {
        return get_scripts(this, args, context);
    }

    // Get HTML content and parse
    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_fragment(&html_content);

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    // Handle "*" to get all elements
    let selector_str = if tag_name_str == "*" {
        "*".to_string()
    } else {
        tag_name_str.clone()
    };

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Set innerHTML
            element_data.set_inner_html(element.inner_html());

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.getElementsByName(name)`
fn get_elements_by_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementsByName called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.getElementsByName called on non-Document object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    let html_content = document.html_content.lock().unwrap().clone();
    let fragment = scraper::Html::parse_fragment(&html_content);

    let result = JsObject::default(context.intrinsics());
    let mut index = 0u32;

    let selector_str = format!("[name=\"{}\"]", name_str);

    if let Ok(selector) = scraper::Selector::parse(&selector_str) {
        for element in fragment.select(&selector) {
            // Create proper ElementData with correct prototype
            let tag_name = element.value().name().to_uppercase();
            let element_data = crate::dom::element::ElementData::with_tag_name(tag_name.clone());

            // Set attributes from the parsed HTML
            for (attr_name, attr_value) in element.value().attrs() {
                element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
            }

            // Create JsObject with HTMLElement prototype chain (ensures instanceof HTMLElement works)
            let prototype = context.intrinsics().constructors().html_element().prototype();
            let element_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                element_data,
            );

            result.define_property_or_throw(
                index,
                PropertyDescriptorBuilder::new()
                    .value(element_obj.upcast())
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            index += 1;
        }
    }

    result.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(index))
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(result.into())
}

/// `Document.prototype.createComment(data)`
fn create_comment(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0).to_string(context)?;
    let data_str = data.to_std_string_escaped();

    // Create a Comment node object
    let comment = JsObject::default(context.intrinsics());

    // nodeType = 8 for Comment
    comment.define_property_or_throw(
        js_string!("nodeType"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(8))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    comment.define_property_or_throw(
        js_string!("nodeName"),
        PropertyDescriptorBuilder::new()
            .value(js_string!("#comment"))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    comment.define_property_or_throw(
        js_string!("data"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(data_str.clone()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    comment.define_property_or_throw(
        js_string!("textContent"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(data_str.clone()))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    comment.define_property_or_throw(
        js_string!("nodeValue"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(data_str))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(comment.into())
}

/// `Document.prototype.createAttribute(name)`
fn create_attribute(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    // Create an Attr node object
    let attr = JsObject::default(context.intrinsics());

    // nodeType = 2 for Attr
    attr.define_property_or_throw(
        js_string!("nodeType"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(2))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    attr.define_property_or_throw(
        js_string!("nodeName"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(name_str.clone()))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    attr.define_property_or_throw(
        js_string!("name"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(name_str))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    attr.define_property_or_throw(
        js_string!("value"),
        PropertyDescriptorBuilder::new()
            .value(js_string!(""))
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    attr.define_property_or_throw(
        js_string!("specified"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::from(true))
            .writable(false)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    attr.define_property_or_throw(
        js_string!("ownerElement"),
        PropertyDescriptorBuilder::new()
            .value(JsValue::null())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build(),
        context,
    )?;

    Ok(attr.into())
}

/// `Document.prototype.hasFocus()`
fn has_focus(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser context, the document always has focus
    Ok(JsValue::from(true))
}

/// `Document.prototype.execCommand(commandId, showUI, value)`
fn exec_command(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let command_id = args.get_or_undefined(0).to_string(context)?;
    let _command_str = command_id.to_std_string_escaped();

    // execCommand is deprecated but still used by some sites
    // Return false to indicate the command was not executed
    // In a headless browser, we don't have editing capabilities
    Ok(JsValue::from(false))
}

/// `Document.prototype.createTreeWalker(root, whatToShow, filter)`
fn create_tree_walker(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use super::treewalker::{TreeWalker, node_filter};

    // Get the root node (required)
    let root = args.get_or_undefined(0);
    let root_obj = root.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("createTreeWalker: root must be a Node")
    })?.clone();

    // Get whatToShow (optional, defaults to SHOW_ALL)
    let what_to_show = if args.len() > 1 && !args.get_or_undefined(1).is_undefined() {
        args.get_or_undefined(1).to_u32(context)?
    } else {
        node_filter::SHOW_ALL
    };

    // Get filter (optional)
    let filter = if args.len() > 2 && !args.get_or_undefined(2).is_null_or_undefined() {
        args.get_or_undefined(2).as_object().map(|o| o.clone())
    } else {
        None
    };

    let tree_walker = TreeWalker::create(root_obj, what_to_show, filter, context)?;
    Ok(tree_walker.into())
}

/// `Document.prototype.createNodeIterator(root, whatToShow, filter)`
fn create_node_iterator(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use super::nodeiterator::NodeIterator;
    use super::treewalker::node_filter;

    // Get the root node (required)
    let root = args.get_or_undefined(0);
    let root_obj = root.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("createNodeIterator: root must be a Node")
    })?.clone();

    // Get whatToShow (optional, defaults to SHOW_ALL)
    let what_to_show = if args.len() > 1 && !args.get_or_undefined(1).is_undefined() {
        args.get_or_undefined(1).to_u32(context)?
    } else {
        node_filter::SHOW_ALL
    };

    // Get filter (optional)
    let filter = if args.len() > 2 && !args.get_or_undefined(2).is_null_or_undefined() {
        args.get_or_undefined(2).as_object().map(|o| o.clone())
    } else {
        None
    };

    let node_iterator = NodeIterator::create(root_obj, what_to_show, filter, context)?;
    Ok(node_iterator.into())
}

/// Canvas `getContext(contextType)` method implementation
fn canvas_get_context(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let context_type = args.get_or_undefined(0).to_string(context)?;
    let context_type_str = context_type.to_std_string_escaped();

    match context_type_str.as_str() {
        "2d" => {
            // Create a Canvas 2D rendering context object
            let context_2d = JsObject::default(context.intrinsics());

            // Add Canvas 2D methods
            // Drawing rectangles
            let fill_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_rect)
                .name(js_string!("fillRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_rect_func)
                    .build(),
                context,
            )?;

            let stroke_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_rect)
                .name(js_string!("strokeRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_rect_func)
                    .build(),
                context,
            )?;

            let clear_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_clear_rect)
                .name(js_string!("clearRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("clearRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(clear_rect_func)
                    .build(),
                context,
            )?;

            // Text rendering
            let fill_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_text)
                .name(js_string!("fillText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_text_func)
                    .build(),
                context,
            )?;

            let stroke_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_text)
                .name(js_string!("strokeText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_text_func)
                    .build(),
                context,
            )?;

            let measure_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_measure_text)
                .name(js_string!("measureText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("measureText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(measure_text_func)
                    .build(),
                context,
            )?;

            // Path methods
            let begin_path_func = BuiltInBuilder::callable(context.realm(), canvas_2d_begin_path)
                .name(js_string!("beginPath"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("beginPath"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(begin_path_func)
                    .build(),
                context,
            )?;

            let move_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_move_to)
                .name(js_string!("moveTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("moveTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(move_to_func)
                    .build(),
                context,
            )?;

            let line_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_line_to)
                .name(js_string!("lineTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("lineTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(line_to_func)
                    .build(),
                context,
            )?;

            let stroke_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke)
                .name(js_string!("stroke"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("stroke"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_func)
                    .build(),
                context,
            )?;

            let fill_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill)
                .name(js_string!("fill"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fill"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_func)
                    .build(),
                context,
            )?;

            // Style properties
            context_2d.define_property_or_throw(
                js_string!("fillStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("strokeStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("lineWidth"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(1.0)
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("font"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("10px sans-serif"))
                    .build(),
                context,
            )?;

            Ok(context_2d.into())
        }
        "webgl" | "experimental-webgl" => {
            create_webgl_context(context, false)
        }
        "webgl2" | "experimental-webgl2" => {
            create_webgl_context(context, true)
        }
        _ => Ok(JsValue::null())
    }
}

/// Canvas `toDataURL(type, quality)` method implementation
fn canvas_to_data_url(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _mime_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let _quality = args.get_or_undefined(1).to_number(context)?;

    // For now, return a minimal empty PNG data URL
    // TODO: Implement actual image generation
    Ok(js_string!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==").into())
}

// Canvas 2D context method implementations
fn canvas_2d_fill_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle drawing
    eprintln!("Canvas fillRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle outlining
    eprintln!("Canvas strokeRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_clear_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle clearing
    eprintln!("Canvas clearRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_fill_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text rendering
    eprintln!("Canvas fillText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text stroking
    eprintln!("Canvas strokeText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_measure_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;

    // Create TextMetrics object
    let metrics = JsObject::default(context.intrinsics());

    // Calculate approximate width (very basic implementation)
    let text_width = text.to_std_string_escaped().len() as f64 * 6.0; // Rough estimate

    metrics.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(text_width)
            .build(),
        context,
    )?;

    // TODO: Add other TextMetrics properties (actualBoundingBoxLeft, etc.)

    Ok(metrics.into())
}

fn canvas_2d_begin_path(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path state management
    eprintln!("Canvas beginPath()");
    Ok(JsValue::undefined())
}

fn canvas_2d_move_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement path cursor movement
    eprintln!("Canvas moveTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_line_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement line drawing to path
    eprintln!("Canvas lineTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path stroking
    eprintln!("Canvas stroke()");
    Ok(JsValue::undefined())
}

fn canvas_2d_fill(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path filling
    eprintln!("Canvas fill()");
    Ok(JsValue::undefined())
}

/// Create WebGL context with comprehensive method support
fn create_webgl_context(context: &mut Context, is_webgl2: bool) -> JsResult<JsValue> {
    let gl_context = JsObject::default(context.intrinsics());

    // WebGL constants (subset of most commonly used)
    gl_context.set(js_string!("VERTEX_SHADER"), JsValue::from(35633), false, context)?;
    gl_context.set(js_string!("FRAGMENT_SHADER"), JsValue::from(35632), false, context)?;
    gl_context.set(js_string!("ARRAY_BUFFER"), JsValue::from(34962), false, context)?;
    gl_context.set(js_string!("STATIC_DRAW"), JsValue::from(35044), false, context)?;
    gl_context.set(js_string!("COLOR_BUFFER_BIT"), JsValue::from(16384), false, context)?;
    gl_context.set(js_string!("TRIANGLES"), JsValue::from(4), false, context)?;
    gl_context.set(js_string!("FLOAT"), JsValue::from(5126), false, context)?;

    // Core WebGL methods
    let create_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let shader_obj = JsObject::default(_context.intrinsics());
        Ok(JsValue::from(shader_obj))
    }) };
    gl_context.set(js_string!("createShader"), JsValue::from(create_shader_fn.to_js_function(context.realm())), false, context)?;

    let create_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let program_obj = JsObject::default(_context.intrinsics());
        Ok(JsValue::from(program_obj))
    }) };
    gl_context.set(js_string!("createProgram"), JsValue::from(create_program_fn.to_js_function(context.realm())), false, context)?;

    let create_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let buffer_obj = JsObject::default(_context.intrinsics());
        Ok(JsValue::from(buffer_obj))
    }) };
    gl_context.set(js_string!("createBuffer"), JsValue::from(create_buffer_fn.to_js_function(context.realm())), false, context)?;

    // Shader operations
    let shader_source_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("shaderSource"), JsValue::from(shader_source_fn.to_js_function(context.realm())), false, context)?;

    let compile_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("compileShader"), JsValue::from(compile_shader_fn.to_js_function(context.realm())), false, context)?;

    // Program operations
    let attach_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("attachShader"), JsValue::from(attach_shader_fn.to_js_function(context.realm())), false, context)?;

    let link_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("linkProgram"), JsValue::from(link_program_fn.to_js_function(context.realm())), false, context)?;

    let use_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("useProgram"), JsValue::from(use_program_fn.to_js_function(context.realm())), false, context)?;

    // Buffer operations
    let bind_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("bindBuffer"), JsValue::from(bind_buffer_fn.to_js_function(context.realm())), false, context)?;

    let buffer_data_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("bufferData"), JsValue::from(buffer_data_fn.to_js_function(context.realm())), false, context)?;

    // Rendering operations
    let viewport_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("viewport"), JsValue::from(viewport_fn.to_js_function(context.realm())), false, context)?;

    let clear_color_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("clearColor"), JsValue::from(clear_color_fn.to_js_function(context.realm())), false, context)?;

    let clear_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("clear"), JsValue::from(clear_fn.to_js_function(context.realm())), false, context)?;

    let draw_arrays_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("drawArrays"), JsValue::from(draw_arrays_fn.to_js_function(context.realm())), false, context)?;

    // Critical fingerprinting methods
    let get_parameter_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
        if args.is_empty() {
            return Ok(JsValue::null());
        }

        let param = args[0].to_i32(context)?;
        match param {
            7936 => Ok(JsValue::from(js_string!("WebKit"))), // GL_VENDOR
            7937 => Ok(JsValue::from(js_string!("WebKit WebGL"))), // GL_RENDERER
            7938 => Ok(JsValue::from(js_string!("WebGL 1.0 (OpenGL ES 2.0 Chromium)"))), // GL_VERSION
            34921 => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"))), // GL_SHADING_LANGUAGE_VERSION
            34930 => Ok(JsValue::from(16)), // GL_MAX_TEXTURE_SIZE
            3379 => Ok(JsValue::from(16384)), // GL_MAX_VIEWPORT_DIMS
            _ => Ok(JsValue::from(0))
        }
    }) };
    gl_context.set(js_string!("getParameter"), JsValue::from(get_parameter_fn.to_js_function(context.realm())), false, context)?;

    // Extensions support
    let get_extension_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
        if args.is_empty() {
            return Ok(JsValue::null());
        }

        let ext_name = args[0].to_string(context)?.to_std_string_escaped();
        match ext_name.as_str() {
            "WEBKIT_EXT_texture_filter_anisotropic" |
            "EXT_texture_filter_anisotropic" |
            "OES_element_index_uint" |
            "OES_standard_derivatives" => {
                let ext_obj = JsObject::default(context.intrinsics());
                Ok(JsValue::from(ext_obj))
            },
            _ => Ok(JsValue::null())
        }
    }) };
    gl_context.set(js_string!("getExtension"), JsValue::from(get_extension_fn.to_js_function(context.realm())), false, context)?;

    let get_supported_extensions_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
        let extensions = vec![
            "WEBKIT_EXT_texture_filter_anisotropic",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "WEBGL_debug_renderer_info"
        ];

        let js_array = boa_engine::object::builtins::JsArray::new(context);
        for (i, ext) in extensions.iter().enumerate() {
            js_array.set(i, js_string!(*ext), true, context).ok();
        }
        Ok(JsValue::from(js_array))
    }) };
    gl_context.set(js_string!("getSupportedExtensions"), JsValue::from(get_supported_extensions_fn.to_js_function(context.realm())), false, context)?;

    // WebGL2 specific methods
    if is_webgl2 {
        let create_vertex_array_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            let vao_obj = JsObject::default(_context.intrinsics());
            Ok(JsValue::from(vao_obj))
        }) };
        gl_context.set(js_string!("createVertexArray"), JsValue::from(create_vertex_array_fn.to_js_function(context.realm())), false, context)?;
    }

    Ok(JsValue::from(gl_context))
}

// ============================================================================
// Document collection and property getters
// ============================================================================

/// `Document.prototype.documentElement` getter
fn get_document_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.documentElement called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.documentElement called on non-Document object")
    })?;

    // Return existing html element or create one
    if let Some(html) = document.get_element("html") {
        Ok(html.into())
    } else {
        // Create a new html element
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let html_element = element_constructor.construct(&[], None, context)?;
        if let Some(elem_data) = html_element.downcast_ref::<crate::dom::element::ElementData>() {
            elem_data.set_tag_name("HTML".to_string());
        }
        document.add_element("html".to_string(), html_element.clone());
        Ok(html_element.into())
    }
}

/// `Document.prototype.forms` getter - returns HTMLCollection of all form elements
fn get_forms(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.forms called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.forms called on non-Document object")
    })?;

    // Parse HTML content to find all form elements
    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut forms = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("form") {
        for form_element in doc.select(&selector) {
            // Create a form element object
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(form_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = form_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name("FORM".to_string());
                    // Set form attributes
                    if let Some(id) = form_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                    if let Some(name) = form_element.value().attr("name") {
                        elem_data.set_attribute("name".to_string(), name.to_string());
                    }
                    if let Some(action) = form_element.value().attr("action") {
                        elem_data.set_attribute("action".to_string(), action.to_string());
                    }
                    if let Some(method) = form_element.value().attr("method") {
                        elem_data.set_attribute("method".to_string(), method.to_string());
                    }
                }
                forms.push(JsValue::from(form_obj));
            }
        }
    }

    // Create HTMLCollection-like array
    let array = boa_engine::builtins::array::Array::create_array_from_list(forms, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.images` getter - returns HTMLCollection of all img elements
fn get_images(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.images called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.images called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut images = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("img") {
        for img_element in doc.select(&selector) {
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(img_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = img_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name("IMG".to_string());
                    if let Some(src) = img_element.value().attr("src") {
                        elem_data.set_attribute("src".to_string(), src.to_string());
                    }
                    if let Some(alt) = img_element.value().attr("alt") {
                        elem_data.set_attribute("alt".to_string(), alt.to_string());
                    }
                    if let Some(id) = img_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                }
                images.push(JsValue::from(img_obj));
            }
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(images, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.links` getter - returns HTMLCollection of all a and area elements with href
fn get_links(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.links called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.links called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut links = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("a[href], area[href]") {
        for link_element in doc.select(&selector) {
            let element_constructor = context.intrinsics().constructors().element().constructor();
            if let Ok(link_obj) = element_constructor.construct(&[], None, context) {
                if let Some(elem_data) = link_obj.downcast_ref::<crate::dom::element::ElementData>() {
                    elem_data.set_tag_name(link_element.value().name().to_uppercase());
                    if let Some(href) = link_element.value().attr("href") {
                        elem_data.set_attribute("href".to_string(), href.to_string());
                    }
                    if let Some(id) = link_element.value().attr("id") {
                        elem_data.set_id(id.to_string());
                    }
                }
                links.push(JsValue::from(link_obj));
            }
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(links, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// `Document.prototype.scripts` getter - returns HTMLCollection of all script elements
/// This includes both scripts from the static HTML and dynamically loaded scripts
fn get_scripts(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.scripts called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.scripts called on non-Document object")
    })?;

    let html_content = document.get_html_content();
    let doc = scraper::Html::parse_document(&html_content);

    let mut scripts = Vec::new();

    // Track script sources to avoid duplicates (static HTML vs registered)
    let mut seen_srcs: std::collections::HashSet<String> = std::collections::HashSet::new();

    // First, get scripts from static HTML parsing
    if let Ok(selector) = scraper::Selector::parse("script") {
        for script_element in doc.select(&selector) {
            // Create proper HTMLScriptElement instead of generic Element
            let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
            if let Ok(script_obj) = script_constructor.construct(&[], None, context) {
                // Collect all attributes from the HTML element
                let mut attrs: HashMap<String, String> = HashMap::new();
                for (key, value) in script_element.value().attrs() {
                    attrs.insert(key.to_string(), value.to_string());
                }

                // Track the src to avoid duplicates
                if let Some(src) = script_element.value().attr("src") {
                    seen_srcs.insert(src.to_string());
                }

                // Set attributes on the HTMLScriptElement
                if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                    if let Some(src) = attrs.get("src") {
                        script_data.set_src(src.clone());
                    }
                    if let Some(type_) = attrs.get("type") {
                        script_data.set_type(type_.clone());
                    }
                    if attrs.contains_key("async") {
                        script_data.set_async(true);
                    }
                    if attrs.contains_key("defer") {
                        script_data.set_defer(true);
                    }
                    if let Some(id) = attrs.get("id") {
                        script_data.set_id(id.clone());
                    }
                    // Set all custom attributes (including data-* attributes)
                    for (key, value) in &attrs {
                        script_data.set_attribute(key, value.clone());
                    }
                }

                // Also set the text content (inline script)
                let text_content: String = script_element.text().collect();
                if !text_content.is_empty() {
                    if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                        script_data.set_text(text_content);
                    }
                }

                scripts.push(JsValue::from(script_obj));
            }
        }
    }

    // Then, add scripts from the loaded_scripts registry that aren't already in the HTML
    let loaded_scripts = document.get_loaded_scripts();
    for entry in loaded_scripts {
        // Skip if we already have this script from HTML parsing
        if let Some(ref src) = entry.src {
            if seen_srcs.contains(src) {
                continue;
            }
        }

        // Create HTMLScriptElement for the registered script
        let script_constructor = context.intrinsics().constructors().html_script_element().constructor();
        if let Ok(script_obj) = script_constructor.construct(&[], None, context) {
            if let Some(script_data) = script_obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
                if let Some(ref src) = entry.src {
                    script_data.set_src(src.clone());
                }
                if let Some(ref type_) = entry.script_type {
                    script_data.set_type(type_.clone());
                }
                script_data.set_async(entry.async_);
                script_data.set_defer(entry.defer);
                script_data.set_text(entry.text.clone());

                // Set all custom attributes
                for (key, value) in &entry.attributes {
                    script_data.set_attribute(key, value.clone());
                }
            }
            scripts.push(JsValue::from(script_obj));
        }
    }

    let array = boa_engine::builtins::array::Array::create_array_from_list(scripts, context);
    add_html_collection_methods(&array, context)?;
    Ok(array.into())
}

/// Helper function to add HTMLCollection methods to an array
fn add_html_collection_methods(array: &JsObject, context: &mut Context) -> JsResult<()> {
    // Add item() method
    let item_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let index = args.get_or_undefined(0).to_u32(ctx)?;
        if let Some(arr) = this.as_object() {
            if let Ok(val) = arr.get(index, ctx) {
                if !val.is_undefined() {
                    return Ok(val);
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("item"))
    .build();
    array.set(js_string!("item"), item_fn, false, context)?;

    // Add namedItem() method
    let named_item_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
        if let Some(arr) = this.as_object() {
            if let Ok(length) = arr.get(js_string!("length"), ctx) {
                let len = length.to_u32(ctx)?;
                for i in 0..len {
                    if let Ok(item) = arr.get(i, ctx) {
                        if let Some(item_obj) = item.as_object() {
                            // Check id attribute
                            if let Ok(id) = item_obj.get(js_string!("id"), ctx) {
                                if id.to_string(ctx)?.to_std_string_escaped() == name {
                                    return Ok(item);
                                }
                            }
                            // Check name attribute
                            if let Ok(elem_name) = item_obj.get(js_string!("name"), ctx) {
                                if elem_name.to_string(ctx)?.to_std_string_escaped() == name {
                                    return Ok(item);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("namedItem"))
    .build();
    array.set(js_string!("namedItem"), named_item_fn, false, context)?;

    Ok(())
}

/// Helper function to add getAttribute method to an element with captured attributes
fn add_get_attribute_method(element: &JsObject, attrs: HashMap<String, String>, context: &mut Context) -> JsResult<()> {
    // Store attributes on the element object itself as a hidden property
    // This allows getAttribute to access them
    let attrs_obj = JsObject::default(context.intrinsics());
    for (key, value) in &attrs {
        attrs_obj.set(js_string!(key.clone()), js_string!(value.clone()), false, context)?;
    }
    element.set(js_string!("__attributes__"), attrs_obj, false, context)?;

    // Create getAttribute method that reads from __attributes__
    let get_attr_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let name = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();

        if let Some(this_obj) = this.as_object() {
            if let Ok(attrs_val) = this_obj.get(js_string!("__attributes__"), ctx) {
                if let Some(attrs_obj) = attrs_val.as_object() {
                    if let Ok(value) = attrs_obj.get(js_string!(name.clone()), ctx) {
                        if !value.is_undefined() {
                            return Ok(value);
                        }
                    }
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("getAttribute"))
    .build();

    element.set(js_string!("getAttribute"), get_attr_fn, false, context)?;

    Ok(())
}

/// `Document.prototype.cookie` getter
fn get_cookie(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.cookie called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.cookie called on non-Document object")
    })?;

    let cookie = document.cookie.lock().unwrap().clone();
    Ok(JsString::from(cookie).into())
}

/// `Document.prototype.cookie` setter
fn set_cookie(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.cookie setter called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.cookie setter called on non-Document object")
    })?;

    let new_cookie = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Cookie setting appends to existing cookies (doesn't replace)
    // Parse the new cookie and add/update it
    let mut cookies = document.cookie.lock().unwrap();
    if cookies.is_empty() {
        *cookies = new_cookie;
    } else {
        // Parse the cookie name from the new cookie
        if let Some(eq_pos) = new_cookie.find('=') {
            let new_name = &new_cookie[..eq_pos];
            // Check if this cookie already exists
            let mut found = false;
            let existing: Vec<&str> = cookies.split("; ").collect();
            let mut updated = Vec::new();
            for cookie in existing {
                if cookie.starts_with(&format!("{}=", new_name)) {
                    updated.push(new_cookie.as_str());
                    found = true;
                } else {
                    updated.push(cookie);
                }
            }
            if !found {
                updated.push(&new_cookie);
            }
            *cookies = updated.join("; ");
        }
    }

    Ok(JsValue::undefined())
}

/// `Document.prototype.referrer` getter
fn get_referrer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.referrer called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.referrer called on non-Document object")
    })?;

    let referrer = document.referrer.lock().unwrap().clone();
    Ok(JsString::from(referrer).into())
}

/// `Document.prototype.domain` getter
fn get_domain(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.domain called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.domain called on non-Document object")
    })?;

    // Extract domain from URL
    let url = document.get_url();
    let domain = if let Ok(parsed) = url::Url::parse(&url) {
        parsed.host_str().unwrap_or("").to_string()
    } else {
        document.domain.lock().unwrap().clone()
    };
    Ok(JsString::from(domain).into())
}

/// `Document.prototype.characterSet` getter
fn get_character_set(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.characterSet called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.characterSet called on non-Document object")
    })?;

    let charset = document.character_set.lock().unwrap().clone();
    Ok(JsString::from(charset).into())
}

/// `Document.prototype.contentType` getter
fn get_content_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.contentType called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.contentType called on non-Document object")
    })?;

    let content_type = document.content_type.lock().unwrap().clone();
    Ok(JsString::from(content_type).into())
}

/// `Document.prototype.visibilityState` getter
fn get_visibility_state(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, document is always "visible"
    Ok(JsString::from("visible").into())
}

/// `Document.prototype.hidden` getter
fn get_hidden(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, document is never hidden
    Ok(false.into())
}

/// `Document.prototype.activeElement` getter
/// Returns the currently focused element, or body if nothing is focused
fn get_active_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.activeElement called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.activeElement called on non-Document object")
    })?;

    // Check the focus manager for the currently active element
    if let Some(active) = crate::browser::focus_manager::FocusManager::get_active_element() {
        return Ok(active.into());
    }

    // Per spec: return body as default active element when no element has focus
    if let Some(body) = document.get_element("body") {
        Ok(body.into())
    } else {
        // Create body if it doesn't exist
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let body_element = element_constructor.construct(&[], None, context)?;
        if let Some(elem_data) = body_element.downcast_ref::<crate::dom::element::ElementData>() {
            elem_data.set_tag_name("BODY".to_string());
        }
        document.add_element("body".to_string(), body_element.clone());
        Ok(body_element.into())
    }
}

/// `Document.prototype.currentScript` getter
/// Returns the script element that is currently being executed, or null if no script is executing.
fn get_current_script(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Try to get the currently executing script from global state
    // The script execution code should set this when executing a script element
    if let Ok(current) = context.global_object().get(js_string!("__currentScript__"), context) {
        if !current.is_undefined() && !current.is_null() {
            return Ok(current);
        }
    }
    // Per spec, returns null when not inside a script element's execution
    Ok(JsValue::null())
}

/// `Document.prototype.scrollingElement` getter
/// Returns the Element that scrolls the document, typically document.documentElement or document.body
/// https://drafts.csswg.org/cssom-view/#dom-document-scrollingelement
fn get_scrolling_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // In standards mode, return documentElement (html element)
    // In quirks mode, return body
    // For simplicity, we always return documentElement (standards mode behavior)
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.scrollingElement called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.scrollingElement called on non-Document object")
    })?;

    // Return documentElement (html element)
    get_document_element(this, _args, context)
}

/// `Document.prototype.elementFromPoint(x, y)`
/// Returns the topmost Element at the specified coordinates (relative to viewport).
/// Used by Cloudflare Turnstile for bot detection during mouse interactions.
/// https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
fn element_from_point(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.elementFromPoint called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.elementFromPoint called on non-Document object")
    })?;

    // Get x and y coordinates
    let x = args.get_or_undefined(0).to_number(context)?;
    let y = args.get_or_undefined(1).to_number(context)?;

    // If coordinates are negative, return null per spec
    if x < 0.0 || y < 0.0 {
        return Ok(JsValue::null());
    }

    // Get viewport dimensions
    let viewport_width = crate::layout_registry::get_viewport_width();
    let viewport_height = crate::layout_registry::get_viewport_height();

    // If outside viewport, return null
    if x > viewport_width || y > viewport_height {
        return Ok(JsValue::null());
    }

    // Get HTML content to find elements
    let html_content = document.get_html_content();

    // In a real browser, this would do hit-testing based on rendered layout.
    // For our headless implementation, we return the body element as a reasonable
    // fallback for bot detection purposes. This allows Turnstile's mouse event
    // validation to find a target element.
    if !html_content.is_empty() {
        // Parse HTML and return body element
        let parsed_doc = scraper::Html::parse_document(&html_content);
        if let Ok(body_selector) = scraper::Selector::parse("body") {
            if let Some(body_ref) = parsed_doc.select(&body_selector).next() {
                let element_constructor = context.intrinsics().constructors().element().constructor();
                let element_obj = element_constructor.construct(&[], Some(&element_constructor), context)?;

                element_obj.set(js_string!("tagName"), js_string!("BODY"), false, context)?;
                element_obj.set(js_string!("nodeType"), 1, false, context)?;

                // Copy attributes
                for (attr_name, attr_value) in body_ref.value().attrs() {
                    element_obj.set(js_string!(attr_name), js_string!(attr_value), false, context)?;
                }

                return Ok(element_obj.into());
            }
        }
    }

    // If no body found, return null
    Ok(JsValue::null())
}

/// `Document.prototype.elementsFromPoint(x, y)`
/// Returns an array of all Elements at the specified coordinates.
/// https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
fn elements_from_point(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get the topmost element
    let top_element = element_from_point(this, args, context)?;

    // Create an array with the element (simplified - real impl would return all stacked elements)
    if !top_element.is_null() {
        let array = boa_engine::builtins::array::Array::array_create(1, None, context)?;
        array.set(0, top_element, true, context)?;
        Ok(array.into())
    } else {
        let array = boa_engine::builtins::array::Array::array_create(0, None, context)?;
        Ok(array.into())
    }
}

/// `Document.prototype.scrollTo(x, y)` or `Document.prototype.scrollTo(options)`
/// In browsers, this scrolls the viewport (delegates to window.scrollTo)
fn scroll_to_document(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Parse arguments - supports both scrollTo(x, y) and scrollTo(options) forms
    let (x, y) = if args.len() >= 2 {
        // scrollTo(x, y) form
        let x = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);
        let y = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
        (x, y)
    } else if let Some(options) = args.get(0).and_then(|v| v.as_object()) {
        // scrollTo(options) form
        let x = options.get(js_string!("left"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        let y = options.get(js_string!("top"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        (x, y)
    } else {
        (0.0, 0.0)
    };

    // Delegate to window.scrollTo via global object
    if let Ok(window_val) = context.global_object().get(js_string!("window"), context) {
        if let Some(window_obj) = window_val.as_object() {
            let _ = window_obj.set(js_string!("scrollX"), x, false, context);
            let _ = window_obj.set(js_string!("scrollY"), y, false, context);
        }
    }

    Ok(JsValue::undefined())
}

/// `Document.prototype.__dispatchTrustedMouseEvent(eventType, clientX, clientY, options?)`
/// Dispatches a trusted mouse event. Used for Cloudflare Turnstile and similar bot detection.
fn dispatch_trusted_mouse_event_document(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::events::ui_events::MouseEventData;

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("__dispatchTrustedMouseEvent called on non-object")
    })?;

    // Verify this is a Document
    if this_obj.downcast_ref::<DocumentData>().is_none() {
        return Err(JsNativeError::typ()
            .with_message("__dispatchTrustedMouseEvent called on non-Document object")
            .into());
    }

    // Get event type
    let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Get coordinates
    let client_x = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
    let client_y = args.get_or_undefined(2).to_number(context).unwrap_or(0.0);

    // Get optional parameters
    let options = args.get_or_undefined(3);
    let (button, buttons, ctrl_key, shift_key, alt_key, meta_key) = if options.is_object() {
        let opts = options.as_object().unwrap();
        let button = opts.get(js_string!("button"), context)
            .map(|v| v.to_i32(context).unwrap_or(0) as i16)
            .unwrap_or(0);
        let buttons = opts.get(js_string!("buttons"), context)
            .map(|v| v.to_u32(context).unwrap_or(0) as u16)
            .unwrap_or(if event_type.contains("down") || event_type == "click" { 1 } else { 0 });
        let ctrl_key = opts.get(js_string!("ctrlKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let shift_key = opts.get(js_string!("shiftKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let alt_key = opts.get(js_string!("altKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let meta_key = opts.get(js_string!("metaKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        (button, buttons, ctrl_key, shift_key, alt_key, meta_key)
    } else {
        let buttons = if event_type.contains("down") || event_type == "click" { 1 } else { 0 };
        (0, buttons, false, false, false, false)
    };

    // Determine event properties
    let (bubbles, cancelable) = match event_type.as_str() {
        "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove"
        | "mouseover" | "mouseout" | "mouseenter" | "mouseleave" => (true, true),
        _ => (true, false),
    };

    // Create trusted mouse event data
    let mut mouse_event = MouseEventData::new_trusted_with_coords(
        event_type.clone(),
        bubbles,
        cancelable,
        client_x,
        client_y,
        client_x,  // screen_x (same as clientX for simplicity)
        client_y,  // screen_y
        client_x,  // page_x
        client_y,  // page_y
        0.0,       // movement_x
        0.0,       // movement_y
        button,
        buttons,
    );

    // Set modifier keys directly (fields are public)
    mouse_event.ctrl_key = ctrl_key;
    mouse_event.shift_key = shift_key;
    mouse_event.alt_key = alt_key;
    mouse_event.meta_key = meta_key;

    // Create the event object
    let event_prototype = context.intrinsics().constructors().mouse_event().prototype();
    let event_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        event_prototype,
        mouse_event,
    );

    // Dispatch to the document using dispatchEvent
    dispatch_event(this, &[event_obj.upcast().into()], context)?;

    Ok(true.into())
}

