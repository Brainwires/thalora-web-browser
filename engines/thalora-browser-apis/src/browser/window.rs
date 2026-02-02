//! Window Web API implementation for Boa
//!
//! Native implementation of Window standard
//! https://html.spec.whatwg.org/#the-window-object

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder}
};
use crate::storage::storage::Storage;
use crate::file::file_system::{show_open_file_picker, show_save_file_picker, show_directory_picker};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JavaScript `Window` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Window;

impl IntrinsicObject for Window {
    fn init(realm: &Realm) {
        let location_func = BuiltInBuilder::callable(realm, get_location)
            .name(js_string!("get location"))
            .build();

        let history_func = BuiltInBuilder::callable(realm, get_history)
            .name(js_string!("get history"))
            .build();

        let document_func = BuiltInBuilder::callable(realm, get_document)
            .name(js_string!("get document"))
            .build();

        let navigator_func = BuiltInBuilder::callable(realm, get_navigator)
            .name(js_string!("get navigator"))
            .build();

        let performance_func = BuiltInBuilder::callable(realm, get_performance)
            .name(js_string!("get performance"))
            .build();

        let screen_func = BuiltInBuilder::callable(realm, get_screen)
            .name(js_string!("get screen"))
            .build();

        let chrome_func = BuiltInBuilder::callable(realm, get_chrome)
            .name(js_string!("get chrome"))
            .build();

        let local_storage_func = BuiltInBuilder::callable(realm, get_local_storage)
            .name(js_string!("get localStorage"))
            .build();

        let session_storage_func = BuiltInBuilder::callable(realm, get_session_storage)
            .name(js_string!("get sessionStorage"))
            .build();

        // indexedDB is now set directly in initialize_browser_apis() rather than as a getter
        // let indexed_db_func = BuiltInBuilder::callable(realm, get_indexed_db)
        //     .name(js_string!("get indexedDB"))
        //     .build();

        let get_selection_func = BuiltInBuilder::callable(realm, get_selection)
            .name(js_string!("getSelection"))
            .build();

        // File System API functions
        let show_open_file_picker_func = BuiltInBuilder::callable(realm, show_open_file_picker)
            .name(js_string!("showOpenFilePicker"))
            .build();

        let show_save_file_picker_func = BuiltInBuilder::callable(realm, show_save_file_picker)
            .name(js_string!("showSaveFilePicker"))
            .build();

        let show_directory_picker_func = BuiltInBuilder::callable(realm, show_directory_picker)
            .name(js_string!("showDirectoryPicker"))
            .build();

        // Frame hierarchy accessors
        let parent_func = BuiltInBuilder::callable(realm, get_window_self)
            .name(js_string!("get parent"))
            .build();

        let top_func = BuiltInBuilder::callable(realm, get_window_self)
            .name(js_string!("get top"))
            .build();

        let self_func = BuiltInBuilder::callable(realm, get_window_self)
            .name(js_string!("get self"))
            .build();

        let frames_func = BuiltInBuilder::callable(realm, get_window_self)
            .name(js_string!("get frames"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("location"),
                Some(location_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("history"),
                Some(history_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("document"),
                Some(document_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("navigator"),
                Some(navigator_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("performance"),
                Some(performance_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("screen"),
                Some(screen_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("chrome"),
                Some(chrome_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("localStorage"),
                Some(local_storage_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("sessionStorage"),
                Some(session_storage_func),
                None,
                Attribute::CONFIGURABLE,
            )
            // indexedDB is now set directly in initialize_browser_apis()
            // .accessor(
            //     js_string!("indexedDB"),
            //     Some(indexed_db_func),
            //     None,
            //     Attribute::CONFIGURABLE,
            // )
            .property(
                js_string!("innerWidth"),
                1366, // Standard desktop width
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("innerHeight"),
                768, // Standard desktop height
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            // Frame hierarchy properties - return self for top-level window
            .accessor(
                js_string!("parent"),
                Some(parent_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("top"),
                Some(top_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("self"),
                Some(self_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("frames"),
                Some(frames_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("length"),
                0, // Number of frames
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("frameElement"),
                JsValue::null(), // null if not in an iframe
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(match_media, js_string!("matchMedia"), 1)
            .method(get_selection, js_string!("getSelection"), 0)
            .method(get_computed_style, js_string!("getComputedStyle"), 1)
            .method(post_message, js_string!("postMessage"), 2)
            .method(show_open_file_picker, js_string!("showOpenFilePicker"), 0)
            .method(show_save_file_picker, js_string!("showSaveFilePicker"), 0)
            .method(show_directory_picker, js_string!("showDirectoryPicker"), 0)
            // Scroll methods
            .method(scroll_to, js_string!("scrollTo"), 2)
            .method(scroll_to, js_string!("scroll"), 2)
            .method(scroll_by, js_string!("scrollBy"), 2)
            // Scroll position properties
            .property(
                js_string!("scrollX"),
                0,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("scrollY"),
                0,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("pageXOffset"),
                0,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("pageYOffset"),
                0,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Window {
    const NAME: JsString = StaticJsStrings::WINDOW;
}

impl BuiltInConstructor for Window {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::window;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::window,
            context,
        )?;

        let window_data = WindowData::new(context);

        let window = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            window_data,
        );

        Ok(window.into())
    }
}

/// Internal data for Window objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WindowData {
    #[unsafe_ignore_trace]
    location: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    history: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    document: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    navigator: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    #[unsafe_ignore_trace]
    current_url: Arc<Mutex<String>>,
}

impl WindowData {
    fn new(context: &mut Context) -> Self {
        Self {
            location: Arc::new(Mutex::new(JsObject::default(context.intrinsics()))),
            history: Arc::new(Mutex::new(JsObject::default(context.intrinsics()))),
            document: Arc::new(Mutex::new(JsObject::default(context.intrinsics()))),
            navigator: Arc::new(Mutex::new(JsObject::default(context.intrinsics()))),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            current_url: Arc::new(Mutex::new("about:blank".to_string())),
        }
    }

    pub fn set_location(&self, location: JsObject) {
        *self.location.lock().unwrap() = location;
    }

    pub fn set_history(&self, history: JsObject) {
        *self.history.lock().unwrap() = history;
    }

    pub fn set_document(&self, document: JsObject) {
        *self.document.lock().unwrap() = document;
    }

    pub fn set_navigator(&self, navigator: JsObject) {
        *self.navigator.lock().unwrap() = navigator;
    }

    pub fn get_location(&self) -> JsObject {
        self.location.lock().unwrap().clone()
    }

    pub fn get_history(&self) -> JsObject {
        self.history.lock().unwrap().clone()
    }

    pub fn get_document(&self) -> JsObject {
        self.document.lock().unwrap().clone()
    }

    pub fn get_navigator(&self) -> JsObject {
        self.navigator.lock().unwrap().clone()
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

    pub fn set_current_url(&self, url: String) {
        *self.current_url.lock().unwrap() = url;
    }

    pub fn get_current_url(&self) -> String {
        self.current_url.lock().unwrap().clone()
    }
}

/// `Window.prototype.location` getter
fn get_location(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.location called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.location called on non-Window object")
    })?;

    let location = window.get_location();

    // Initialize location object if empty
    if !location.has_property(js_string!("href"), context)? {
            let current_url = window.get_current_url();

            // Add href property
            location.define_property_or_throw(
                js_string!("href"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(current_url.as_str()))
                    .build(),
                context,
            )?;

            // Add protocol property
            let protocol = if current_url.starts_with("https:") {
                "https:"
            } else if current_url.starts_with("http:") {
                "http:"
            } else {
                "about:"
            };

            location.define_property_or_throw(
                js_string!("protocol"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(protocol))
                    .build(),
                context,
            )?;

            // Add hostname property
            let hostname = if let Some(url_start) = current_url.find("://") {
                let after_protocol = &current_url[url_start + 3..];
                if let Some(slash_pos) = after_protocol.find('/') {
                    &after_protocol[..slash_pos]
                } else {
                    after_protocol
                }
            } else {
                ""
            };

            location.define_property_or_throw(
                js_string!("hostname"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(hostname))
                    .build(),
                context,
            )?;
        }

    Ok(location.into())
}

/// `Window.prototype.history` getter
fn get_history(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.history called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.history called on non-Window object")
    })?;

    let history = window.get_history();

    // Initialize history object if empty
    if !history.has_property(js_string!("length"), context)? {
            // Add length property
            history.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(1)
                    .build(),
                context,
            )?;

            // Add state property
            history.define_property_or_throw(
                js_string!("state"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsValue::null())
                    .build(),
                context,
            )?;

            // Add back method
            let back_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                // Implementation would trigger pageswap event and navigate back
                Ok(JsValue::undefined())
            })
            .name(js_string!("back"))
            .build();

            history.define_property_or_throw(
                js_string!("back"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(back_function)
                    .build(),
                context,
            )?;

            // Add forward method
            let forward_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                // Implementation would trigger pageswap event and navigate forward
                Ok(JsValue::undefined())
            })
            .name(js_string!("forward"))
            .build();

            history.define_property_or_throw(
                js_string!("forward"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(forward_function)
                    .build(),
                context,
            )?;

            // Add go method
            let go_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _delta = _args.get_or_undefined(0);
                // Implementation would trigger pageswap event and navigate by delta
                Ok(JsValue::undefined())
            })
            .name(js_string!("go"))
            .build();

            history.define_property_or_throw(
                js_string!("go"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(go_function)
                    .build(),
                context,
            )?;

            // Add pushState method
            let push_state_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _state = _args.get_or_undefined(0);
                let _title = _args.get_or_undefined(1);
                let _url = _args.get_or_undefined(2);
                // Implementation would trigger pageswap event and push new state
                Ok(JsValue::undefined())
            })
            .name(js_string!("pushState"))
            .build();

            history.define_property_or_throw(
                js_string!("pushState"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(push_state_function)
                    .build(),
                context,
            )?;

            // Add replaceState method
            let replace_state_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _state = _args.get_or_undefined(0);
                let _title = _args.get_or_undefined(1);
                let _url = _args.get_or_undefined(2);
                // Implementation would trigger pageswap event and replace current state
                Ok(JsValue::undefined())
            })
            .name(js_string!("replaceState"))
            .build();

            history.define_property_or_throw(
                js_string!("replaceState"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(replace_state_function)
                    .build(),
                context,
            )?;
        }

    Ok(history.into())
}

/// `Window.prototype.document` getter
fn get_document(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.document called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.document called on non-Window object")
    })?;

    let document = window.get_document();

    // Initialize document if needed
    if !document.has_property(js_string!("readyState"), context)? {
        // Create a new Document instance
        use crate::dom::document::Document;

        let document_constructor_args: &[JsValue] = &[];
        let new_document = Document::constructor(&JsValue::undefined(), document_constructor_args, context)?;

        if let Some(doc_obj) = new_document.as_object() {
            window.set_document(doc_obj.clone());
            return Ok(new_document);
        }
    }

    Ok(document.into())
}

/// `Window.prototype.navigator` getter
fn get_navigator(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.navigator called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.navigator called on non-Window object")
    })?;

    let navigator = window.get_navigator();

        // Initialize navigator object - check if we have a Navigator constructor registered externally
        if !navigator.has_property(js_string!("userAgent"), context)? {
            eprintln!("🔍 DEBUG: navigator doesn't have userAgent, trying to create proper Navigator instance");
            // Try to create a proper Navigator instance from external constructor
            if let Some(nav_constructor) = context.realm().get_external_constructor("Navigator") {
                eprintln!("🔍 DEBUG: Found Navigator constructor");
                use boa_engine::builtins::BuiltInConstructor;
                use crate::browser::navigator::Navigator;

                let nav_args: &[JsValue] = &[];
                let nav_fn_value: JsValue = nav_constructor.constructor().clone().into();
                eprintln!("🔍 DEBUG: About to call Navigator::constructor");
                match Navigator::constructor(&nav_fn_value, nav_args, context) {
                    Ok(nav_instance) => {
                        eprintln!("🔍 DEBUG: Navigator::constructor succeeded: {:?}", nav_instance);
                        if let Some(nav_obj) = nav_instance.as_object() {
                            eprintln!("🔍 DEBUG: Got navigator object, setting it on window");
                            window.set_navigator(nav_obj.clone());
                            return Ok(nav_instance);
                        } else {
                            eprintln!("❌ DEBUG: nav_instance is not an object!");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ DEBUG: Navigator::constructor failed: {:?}", e);
                    }
                }
            } else {
                eprintln!("❌ DEBUG: No Navigator constructor found in context.realm()");
            }
            eprintln!("🔍 DEBUG: Falling back to manual navigator property creation");

            // Fallback: manually add properties if Navigator constructor not available
            // Add userAgent property - use shared USER_AGENT constant
            navigator.define_property_or_throw(
                js_string!("userAgent"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from(thalora_constants::USER_AGENT))
                    .build(),
                context,
            )?;

            // Add platform property - Windows to match userAgent
            navigator.define_property_or_throw(
                js_string!("platform"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("MacIntel"))
                    .build(),
                context,
            )?;

            // Add language property
            navigator.define_property_or_throw(
                js_string!("language"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("en-US"))
                    .build(),
                context,
            )?;

            // Add languages array property
            use boa_engine::builtins::array::Array;
            let languages_array = Array::create_array_from_list([
                JsString::from("en-US").into(),
                JsString::from("en").into(),
            ], context);
            navigator.define_property_or_throw(
                js_string!("languages"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(languages_array)
                    .build(),
                context,
            )?;

            // Add onLine property
            navigator.define_property_or_throw(
                js_string!("onLine"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(true)
                    .build(),
                context,
            )?;

            // Add webdriver property (should be false for legitimate browsers)
            navigator.define_property_or_throw(
                js_string!("webdriver"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(false)
                    .build(),
                context,
            )?;

            // Add plugins array (fake some common plugins)
            let plugins_array = Array::create_array_from_list([
                // Create fake PDF Viewer plugin
                create_fake_plugin(context, "PDF Viewer", "Portable Document Format", "pdf")?,
                // Create fake Chrome PDF Plugin
                create_fake_plugin(context, "Chrome PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake Chromium PDF Plugin
                create_fake_plugin(context, "Chromium PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake Microsoft Edge PDF Plugin
                create_fake_plugin(context, "Microsoft Edge PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake WebKit built-in PDF
                create_fake_plugin(context, "WebKit built-in PDF", "Portable Document Format", "pdf")?,
            ], context);

            // Add length property to plugins array
            plugins_array.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(false)
                    .writable(false)
                    .value(5)
                    .build(),
                context,
            )?;

            navigator.define_property_or_throw(
                js_string!("plugins"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(plugins_array)
                    .build(),
                context,
            )?;

            // Add mimeTypes array (related to plugins)
            let mime_types_array = Array::create_array_from_list([
                create_fake_mime_type(context, "application/pdf", "pdf")?,
                create_fake_mime_type(context, "text/pdf", "pdf")?,
            ], context);

            mime_types_array.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(false)
                    .writable(false)
                    .value(2)
                    .build(),
                context,
            )?;

            navigator.define_property_or_throw(
                js_string!("mimeTypes"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(mime_types_array)
                    .build(),
                context,
            )?;

            // Add cookieEnabled property
            navigator.define_property_or_throw(
                js_string!("cookieEnabled"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(true)
                    .build(),
                context,
            )?;

            // Add doNotTrack property
            navigator.define_property_or_throw(
                js_string!("doNotTrack"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsValue::null())
                    .build(),
                context,
            )?;

            // Add Web Locks API (navigator.locks)
            use crate::locks::LockManager;
            let lock_manager = LockManager::new();
            let lock_manager_proto = context.intrinsics().constructors().lock_manager().prototype();
            let lock_manager_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                lock_manager_proto,
                lock_manager,
            );
            navigator.define_property_or_throw(
                js_string!("locks"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(lock_manager_obj)
                    .build(),
                context,
            )?;

            // Add hardwareConcurrency property (fake CPU core count)
            navigator.define_property_or_throw(
                js_string!("hardwareConcurrency"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(8) // Fake 8 CPU cores
                    .build(),
                context,
            )?;

            // Add maxTouchPoints property
            navigator.define_property_or_throw(
                js_string!("maxTouchPoints"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(0) // Desktop - no touch
                    .build(),
                context,
            )?;

            // Add Storage API (navigator.storage)
            use crate::storage::storage_manager::StorageManager;
            let storage_manager = StorageManager::create_storage_manager();
            let storage_manager_prototype = context.intrinsics().constructors().storage_manager().prototype();
            storage_manager.set_prototype(Some(storage_manager_prototype));
            navigator.define_property_or_throw(
                js_string!("storage"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(storage_manager)
                    .build(),
                context,
            )?;
        } else {
            eprintln!("🔍 DEBUG: navigator already has userAgent, using existing object");
        }

    Ok(navigator.into())
}

/// `Window.prototype.addEventListener(type, listener)`
fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.addEventListener called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.addEventListener called on non-Window object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1).clone();

    window.add_event_listener(event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Window.prototype.removeEventListener(type, listener)`
fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.removeEventListener called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.removeEventListener called on non-Window object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    window.remove_event_listener(&event_type.to_std_string_escaped(), listener);
    Ok(JsValue::undefined())
}

/// `Window.prototype.dispatchEvent(event)`
fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.dispatchEvent called on non-object")
    })?;

    let window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.dispatchEvent called on non-Window object")
    })?;

    let event = args.get_or_undefined(0);

    // Get event type from event object
    if event.is_object() {
        if let Some(event_obj) = event.as_object() {
            if let Ok(type_val) = event_obj.get(js_string!("type"), context) {
                let event_type = type_val.to_string(context)?;
                let listeners = window.get_event_listeners(&event_type.to_std_string_escaped());

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

/// `Window.prototype.matchMedia(mediaQuery)`
fn match_media(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.matchMedia called on non-object")
    })?;

    let _window = this_obj.downcast_ref::<WindowData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Window.prototype.matchMedia called on non-Window object")
    })?;

    let media_query = args.get_or_undefined(0).to_string(context)?;
    let query_str = media_query.to_std_string_escaped();

    // Parse and evaluate the media query
    let matches = evaluate_media_query(&query_str);

    // Create MediaQueryListData with listener storage
    let mql_data = MediaQueryListData::new(query_str.clone(), matches);

    // Create MediaQueryList object with internal data
    let media_query_list_typed = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().object().prototype(),
        mql_data,
    );

    // Upcast to untyped JsObject for property definition methods
    let media_query_list = media_query_list_typed.upcast();

    // Add properties to MediaQueryList
    media_query_list.define_property_or_throw(
        js_string!("media"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(media_query)
            .build(),
        context,
    )?;

    media_query_list.define_property_or_throw(
        js_string!("matches"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(matches)
            .build(),
        context,
    )?;

    // Add addListener method
    let add_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_listener)
        .name(js_string!("addListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("addListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(add_listener_func)
            .build(),
        context,
    )?;

    // Add removeListener method
    let remove_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_listener)
        .name(js_string!("removeListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("removeListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(remove_listener_func)
            .build(),
        context,
    )?;

    // Add addEventListener method (newer API)
    let add_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_event_listener)
        .name(js_string!("addEventListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("addEventListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(add_event_listener_func)
            .build(),
        context,
    )?;

    // Add removeEventListener method
    let remove_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_event_listener)
        .name(js_string!("removeEventListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("removeEventListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(remove_event_listener_func)
            .build(),
        context,
    )?;

    Ok(media_query_list.into())
}

/// Create the matchMedia global function for use outside Window context
pub fn create_match_media_function(context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::FunctionObjectBuilder;
    use boa_engine::NativeFunction;

    let func = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(global_match_media),
    )
    .name(js_string!("matchMedia"))
    .length(1)
    .build();

    Ok(func.into())
}

/// Global matchMedia function - creates MediaQueryList without requiring Window context
fn global_match_media(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let media_query = args.get_or_undefined(0).to_string(context)?;
    let query_str = media_query.to_std_string_escaped();

    // Parse and evaluate the media query
    let matches = evaluate_media_query(&query_str);

    // Create MediaQueryListData with listener storage
    let mql_data = MediaQueryListData::new(query_str.clone(), matches);

    // Create MediaQueryList object with internal data
    let media_query_list_typed = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().object().prototype(),
        mql_data,
    );

    let media_query_list = media_query_list_typed.upcast();

    // Add properties to MediaQueryList
    media_query_list.define_property_or_throw(
        js_string!("media"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(media_query)
            .build(),
        context,
    )?;

    media_query_list.define_property_or_throw(
        js_string!("matches"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(matches)
            .build(),
        context,
    )?;

    // Add addEventListener method
    let add_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_event_listener)
        .name(js_string!("addEventListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("addEventListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(add_event_listener_func)
            .build(),
        context,
    )?;

    // Add removeEventListener method
    let remove_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_event_listener)
        .name(js_string!("removeEventListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("removeEventListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(remove_event_listener_func)
            .build(),
        context,
    )?;

    // Add legacy addListener method
    let add_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_listener)
        .name(js_string!("addListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("addListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(add_listener_func)
            .build(),
        context,
    )?;

    // Add legacy removeListener method
    let remove_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_listener)
        .name(js_string!("removeListener"))
        .build();

    media_query_list.define_property_or_throw(
        js_string!("removeListener"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(remove_listener_func)
            .build(),
        context,
    )?;

    Ok(media_query_list.into())
}

/// Enhanced media query evaluator with better parsing
fn evaluate_media_query(query: &str) -> bool {
    let query = query.trim();

    // Default viewport dimensions (can be made configurable later)
    let viewport_width = 1366.0; // Common desktop width
    let viewport_height = 768.0;  // Common desktop height
    let pixel_density = 1.0;

    // Handle empty/all queries
    if query.is_empty() || query == "all" {
        return true;
    }

    // Handle media types
    if query == "screen" {
        return true;
    }
    if query == "print" || query == "speech" || query == "braille" {
        return false;
    }

    // Parse complex queries with logical operators
    if query.contains(" and ") {
        return query.split(" and ")
            .all(|part| evaluate_single_media_feature(part.trim(), viewport_width, viewport_height, pixel_density));
    }

    if query.contains(" or ") || query.contains(", ") {
        // Handle both comma-separated and " or " separated queries
        let parts = if query.contains(", ") {
            query.split(", ").collect::<Vec<_>>()
        } else {
            query.split(" or ").collect::<Vec<_>>()
        };

        return parts.iter()
            .map(|part| part.trim())
            .any(|part| {
                if part.contains(" and ") {
                    part.split(" and ")
                        .all(|subpart| evaluate_single_media_feature(subpart.trim(), viewport_width, viewport_height, pixel_density))
                } else {
                    evaluate_single_media_feature(part, viewport_width, viewport_height, pixel_density)
                }
            });
    }

    // Single media feature
    evaluate_single_media_feature(query, viewport_width, viewport_height, pixel_density)
}

fn evaluate_single_media_feature(feature: &str, width: f64, height: f64, density: f64) -> bool {
    let feature = feature.trim();

    // Remove parentheses if present
    let feature = if feature.starts_with('(') && feature.ends_with(')') {
        &feature[1..feature.len()-1]
    } else {
        feature
    };

    // Width queries
    if let Some(value) = extract_pixel_value(feature, "max-width") {
        return width <= value;
    }
    if let Some(value) = extract_pixel_value(feature, "min-width") {
        return width >= value;
    }
    if let Some(value) = extract_pixel_value(feature, "width") {
        return width == value;
    }

    // Height queries
    if let Some(value) = extract_pixel_value(feature, "max-height") {
        return height <= value;
    }
    if let Some(value) = extract_pixel_value(feature, "min-height") {
        return height >= value;
    }
    if let Some(value) = extract_pixel_value(feature, "height") {
        return height == value;
    }

    // Device pixel ratio
    if let Some(value) = extract_float_value(feature, "max-device-pixel-ratio") {
        return density <= value;
    }
    if let Some(value) = extract_float_value(feature, "min-device-pixel-ratio") {
        return density >= value;
    }
    if let Some(value) = extract_float_value(feature, "-webkit-max-device-pixel-ratio") {
        return density <= value;
    }
    if let Some(value) = extract_float_value(feature, "-webkit-min-device-pixel-ratio") {
        return density >= value;
    }

    // Orientation
    if feature.contains("orientation: landscape") || feature.contains("orientation:landscape") {
        return width > height;
    }
    if feature.contains("orientation: portrait") || feature.contains("orientation:portrait") {
        return height > width;
    }

    // Media types
    if feature == "screen" {
        return true;
    }
    if feature == "print" || feature == "speech" || feature == "braille" {
        return false;
    }

    // Color capabilities
    if feature.contains("color") && !feature.contains(":") {
        return true; // Assume color display
    }
    if let Some(value) = extract_numeric_value(feature, "min-color") {
        return value <= 8; // 8-bit color depth
    }

    // Default to true for unrecognized features to be permissive
    true
}

fn extract_pixel_value(feature: &str, property: &str) -> Option<f64> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        let value_part = value_part.trim();

        // Handle px values
        if value_part.ends_with("px") {
            if let Ok(value) = value_part[..value_part.len()-2].trim().parse::<f64>() {
                return Some(value);
            }
        }

        // Handle em values (assume 16px = 1em)
        if value_part.ends_with("em") {
            if let Ok(value) = value_part[..value_part.len()-2].trim().parse::<f64>() {
                return Some(value * 16.0);
            }
        }

        // Handle rem values (assume 16px = 1rem)
        if value_part.ends_with("rem") {
            if let Ok(value) = value_part[..value_part.len()-3].trim().parse::<f64>() {
                return Some(value * 16.0);
            }
        }

        // Handle unitless values (assume px)
        if let Ok(value) = value_part.parse::<f64>() {
            return Some(value);
        }
    }
    None
}

fn extract_float_value(feature: &str, property: &str) -> Option<f64> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        if let Ok(value) = value_part.trim().parse::<f64>() {
            return Some(value);
        }
    }
    None
}

fn extract_numeric_value(feature: &str, property: &str) -> Option<u32> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        if let Ok(value) = value_part.trim().parse::<u32>() {
            return Some(value);
        }
    }
    None
}

/// Internal data for MediaQueryList objects
/// Stores the media query string, match state, and event listeners
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MediaQueryListData {
    #[unsafe_ignore_trace]
    media: String,
    #[unsafe_ignore_trace]
    matches: Arc<Mutex<bool>>,
    /// Legacy addListener/removeListener callbacks (deprecated but still used)
    #[unsafe_ignore_trace]
    legacy_listeners: Arc<Mutex<Vec<JsValue>>>,
    /// Modern addEventListener callbacks, keyed by event type ("change")
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
}

impl MediaQueryListData {
    fn new(media: String, matches: bool) -> Self {
        Self {
            media,
            matches: Arc::new(Mutex::new(matches)),
            legacy_listeners: Arc::new(Mutex::new(Vec::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_media(&self) -> &str {
        &self.media
    }

    fn get_matches(&self) -> bool {
        *self.matches.lock().unwrap()
    }

    fn add_legacy_listener(&self, listener: JsValue) {
        let mut listeners = self.legacy_listeners.lock().unwrap();
        // Don't add duplicate listeners
        if !listeners.iter().any(|l| JsValue::same_value(l, &listener)) {
            listeners.push(listener);
        }
    }

    fn remove_legacy_listener(&self, listener: &JsValue) {
        let mut listeners = self.legacy_listeners.lock().unwrap();
        listeners.retain(|l| !JsValue::same_value(l, listener));
    }

    fn add_event_listener(&self, event_type: String, listener: JsValue) {
        let mut listeners = self.event_listeners.lock().unwrap();
        let type_listeners = listeners.entry(event_type).or_insert_with(Vec::new);
        // Don't add duplicate listeners
        if !type_listeners.iter().any(|l| JsValue::same_value(l, &listener)) {
            type_listeners.push(listener);
        }
    }

    fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        let mut listeners = self.event_listeners.lock().unwrap();
        if let Some(type_listeners) = listeners.get_mut(event_type) {
            type_listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }
}

// MediaQueryList method implementations
fn media_query_list_add_listener(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let listener = args.get_or_undefined(0);
    if listener.is_undefined() || listener.is_null() {
        return Ok(JsValue::undefined());
    }

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MediaQueryList.addListener called on non-object")
    })?;

    if let Some(mql_data) = this_obj.downcast_ref::<MediaQueryListData>() {
        mql_data.add_legacy_listener(listener.clone());
    }
    Ok(JsValue::undefined())
}

fn media_query_list_remove_listener(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let listener = args.get_or_undefined(0);
    if listener.is_undefined() || listener.is_null() {
        return Ok(JsValue::undefined());
    }

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MediaQueryList.removeListener called on non-object")
    })?;

    if let Some(mql_data) = this_obj.downcast_ref::<MediaQueryListData>() {
        mql_data.remove_legacy_listener(listener);
    }
    Ok(JsValue::undefined())
}

fn media_query_list_add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);
    if listener.is_undefined() || listener.is_null() {
        return Ok(JsValue::undefined());
    }

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MediaQueryList.addEventListener called on non-object")
    })?;

    if let Some(mql_data) = this_obj.downcast_ref::<MediaQueryListData>() {
        mql_data.add_event_listener(event_type.to_std_string_escaped(), listener.clone());
    }
    Ok(JsValue::undefined())
}

fn media_query_list_remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);
    if listener.is_undefined() || listener.is_null() {
        return Ok(JsValue::undefined());
    }

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MediaQueryList.removeEventListener called on non-object")
    })?;

    if let Some(mql_data) = this_obj.downcast_ref::<MediaQueryListData>() {
        mql_data.remove_event_listener(&event_type.to_std_string_escaped(), listener);
    }
    Ok(JsValue::undefined())
}

/// `Window.prototype.screen` getter
fn get_screen(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    eprintln!("✅ Creating Screen object for window.screen");

    // Check if we already have a screen object in global scope to avoid creating duplicate
    if let Ok(existing_screen) = context.global_object().get(js_string!("screen"), context) {
        if !existing_screen.is_undefined() {
            return Ok(existing_screen);
        }
    }

    // Create Screen object
    let screen = JsObject::default(context.intrinsics());

    // Default desktop screen dimensions (1920x1080)
    let width = 1920;
    let height = 1080;
    let avail_width = 1920; // Available width (excluding taskbar, etc.)
    let avail_height = 1040; // Available height (excluding taskbar)
    let color_depth = 24; // 24-bit color
    let pixel_depth = 24; // Same as color depth on modern displays

    // Add width property
    screen.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(width)
            .build(),
        context,
    )?;

    // Add height property
    screen.define_property_or_throw(
        js_string!("height"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(height)
            .build(),
        context,
    )?;

    // Add availWidth property
    screen.define_property_or_throw(
        js_string!("availWidth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(avail_width)
            .build(),
        context,
    )?;

    // Add availHeight property
    screen.define_property_or_throw(
        js_string!("availHeight"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(avail_height)
            .build(),
        context,
    )?;

    // Add colorDepth property
    screen.define_property_or_throw(
        js_string!("colorDepth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(color_depth)
            .build(),
        context,
    )?;

    // Add pixelDepth property
    screen.define_property_or_throw(
        js_string!("pixelDepth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(pixel_depth)
            .build(),
        context,
    )?;

    // Create orientation object
    let orientation = JsObject::default(context.intrinsics());

    // Add orientation properties
    orientation.define_property_or_throw(
        js_string!("angle"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(0) // 0 degrees (landscape)
            .build(),
        context,
    )?;

    orientation.define_property_or_throw(
        js_string!("type"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(js_string!("landscape-primary"))
            .build(),
        context,
    )?;

    // Add lock method to orientation
    let lock_func = BuiltInBuilder::callable(context.realm(), screen_orientation_lock)
        .name(js_string!("lock"))
        .build();

    orientation.define_property_or_throw(
        js_string!("lock"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(lock_func)
            .build(),
        context,
    )?;

    // Add unlock method to orientation
    let unlock_func = BuiltInBuilder::callable(context.realm(), screen_orientation_unlock)
        .name(js_string!("unlock"))
        .build();

    orientation.define_property_or_throw(
        js_string!("unlock"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(unlock_func)
            .build(),
        context,
    )?;

    // Add orientation property to screen
    screen.define_property_or_throw(
        js_string!("orientation"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(orientation)
            .build(),
        context,
    )?;

    // Also register screen as a global variable (not just window.screen)
    // This ensures both window.screen and global screen work correctly
    context.global_object().set(
        js_string!("screen"),
        screen.clone(),
        false,
        context,
    )?;

    eprintln!("✅ Screen object registered both as window.screen and global screen");

    Ok(screen.into())
}

/// Global storage for screen orientation lock state
/// In a headless browser, we simulate orientation locking by tracking the state
static SCREEN_ORIENTATION_LOCK: std::sync::OnceLock<Mutex<Option<String>>> = std::sync::OnceLock::new();

fn get_orientation_lock() -> &'static Mutex<Option<String>> {
    SCREEN_ORIENTATION_LOCK.get_or_init(|| Mutex::new(None))
}

/// Valid orientation values per the Screen Orientation API spec
const VALID_ORIENTATIONS: &[&str] = &[
    "any",
    "natural",
    "landscape",
    "portrait",
    "portrait-primary",
    "portrait-secondary",
    "landscape-primary",
    "landscape-secondary",
];

// Screen orientation method implementations
fn screen_orientation_lock(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let orientation = args.get_or_undefined(0).to_string(context)?;
    let orientation_str = orientation.to_std_string_escaped();

    // Validate orientation value
    if !VALID_ORIENTATIONS.contains(&orientation_str.as_str()) {
        // Per spec: TypeError for invalid orientation
        return Err(JsNativeError::typ()
            .with_message(format!("Invalid orientation: {}", orientation_str))
            .into());
    }

    // Store the locked orientation
    if let Ok(mut lock) = get_orientation_lock().lock() {
        *lock = Some(orientation_str.clone());
    }

    // Return a resolved Promise (in headless, orientation is always "successful")
    let promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    Ok(promise)
}

fn screen_orientation_unlock(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Clear the locked orientation state
    if let Ok(mut lock) = get_orientation_lock().lock() {
        *lock = None;
    }

    // Return undefined as per spec
    Ok(JsValue::undefined())
}

/// Helper function to create fake plugin objects
fn create_fake_plugin(context: &mut Context, name: &str, description: &str, suffix: &str) -> JsResult<JsValue> {
    let plugin = JsObject::default(context.intrinsics());

    // Add name property
    plugin.define_property_or_throw(
        js_string!("name"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(name))
            .build(),
        context,
    )?;

    // Add description property
    plugin.define_property_or_throw(
        js_string!("description"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(description))
            .build(),
        context,
    )?;

    // Add filename property
    plugin.define_property_or_throw(
        js_string!("filename"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(format!("internal-{}-viewer", suffix)))
            .build(),
        context,
    )?;

    // Add length property
    plugin.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(false)
            .writable(false)
            .value(1)
            .build(),
        context,
    )?;

    Ok(plugin.into())
}

/// Helper function to create fake MIME type objects
fn create_fake_mime_type(context: &mut Context, type_name: &str, suffix: &str) -> JsResult<JsValue> {
    let mime_type = JsObject::default(context.intrinsics());

    // Add type property
    mime_type.define_property_or_throw(
        js_string!("type"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(type_name))
            .build(),
        context,
    )?;

    // Add suffixes property
    mime_type.define_property_or_throw(
        js_string!("suffixes"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(suffix))
            .build(),
        context,
    )?;

    // Add description property
    mime_type.define_property_or_throw(
        js_string!("description"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from("Portable Document Format"))
            .build(),
        context,
    )?;

    // Add enabledPlugin property (reference back to plugin)
    let fake_plugin = create_fake_plugin(context, "PDF Viewer", "Portable Document Format", suffix)?;
    mime_type.define_property_or_throw(
        js_string!("enabledPlugin"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(fake_plugin)
            .build(),
        context,
    )?;

    Ok(mime_type.into())
}

/// `Window.prototype.chrome` getter - Chrome-specific APIs
fn get_chrome(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create Chrome object with common Chrome-specific APIs
    let chrome = JsObject::default(context.intrinsics());

    // Add runtime object (Chrome extension API)
    let runtime = JsObject::default(context.intrinsics());

    // Add onConnect property to runtime (for Chrome extensions)
    runtime.define_property_or_throw(
        js_string!("onConnect"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsValue::undefined())
            .build(),
        context,
    )?;

    // Add getManifest method to runtime
    let get_manifest_func = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        // Return undefined since we're not a Chrome extension
        Ok(JsValue::undefined())
    })
    .name(js_string!("getManifest"))
    .build();

    runtime.define_property_or_throw(
        js_string!("getManifest"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(get_manifest_func)
            .build(),
        context,
    )?;

    chrome.define_property_or_throw(
        js_string!("runtime"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(runtime)
            .build(),
        context,
    )?;

    // Add app object (Chrome Apps API)
    let app = JsObject::default(context.intrinsics());

    // Add isInstalled property
    app.define_property_or_throw(
        js_string!("isInstalled"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(false)
            .build(),
        context,
    )?;

    chrome.define_property_or_throw(
        js_string!("app"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(app)
            .build(),
        context,
    )?;

    // Add csi method (Chrome Speed Index)
    let csi_func = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        // Return empty object for CSI data
        Ok(JsValue::undefined())
    })
    .name(js_string!("csi"))
    .build();

    chrome.define_property_or_throw(
        js_string!("csi"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(csi_func)
            .build(),
        context,
    )?;

    // Add loadTimes method (deprecated but still checked)
    let load_times_func = BuiltInBuilder::callable(context.realm(), |_this, _args, context| {
        // Return realistic Chrome loadTimes object
        let load_times = JsObject::default(context.intrinsics());

        load_times.define_property_or_throw(
            js_string!("requestTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.0) // Unix timestamp
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("startLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.1)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("commitLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.2)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("finishDocumentLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.3)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("finishLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.4)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("firstPaintTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.5)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("firstPaintAfterLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.6)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("navigationType"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(JsString::from("navigate"))
                .build(),
            context,
        )?;

        Ok(load_times.into())
    })
    .name(js_string!("loadTimes"))
    .build();

    chrome.define_property_or_throw(
        js_string!("loadTimes"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(load_times_func)
            .build(),
        context,
    )?;

    Ok(chrome.into())
}

/// `window.localStorage` getter
fn get_local_storage(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create localStorage instance with Storage prototype
    let local_storage = Storage::create_local_storage();

    // Set the prototype to the Storage prototype
    let storage_prototype = context.intrinsics().constructors().storage().prototype();
    local_storage.set_prototype(Some(storage_prototype));

    Ok(local_storage.into())
}

/// `window.sessionStorage` getter
fn get_session_storage(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create sessionStorage instance with Storage prototype
    let session_storage = Storage::create_session_storage();

    // Set the prototype to the Storage prototype
    let storage_prototype = context.intrinsics().constructors().storage().prototype();
    session_storage.set_prototype(Some(storage_prototype));

    Ok(session_storage.into())
}

/// `window.indexedDB` getter
#[cfg(feature = "native")]
fn get_indexed_db(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::storage::indexed_db::factory::IDBFactory;

    // Create a new IDBFactory instance
    let factory = IDBFactory::new().map_err(|e| {
        JsNativeError::error().with_message(format!("Failed to create IDBFactory: {}", e))
    })?;

    // Create JsObject with IDBFactory data
    let factory_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().idb_factory().prototype(),
        factory,
    );

    Ok(factory_obj.into())
}

/// `window.indexedDB` getter (WASM version - stub)
#[cfg(not(feature = "native"))]
fn get_indexed_db(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In WASM, we use the browser's native IndexedDB
    Ok(JsValue::undefined())
}

fn get_selection(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a new Selection instance using the Selection constructor
    use crate::browser::selection::Selection;
    use boa_engine::builtins::IntrinsicObject;

    let selection_constructor = Selection::get(context.intrinsics());
    let selection_args = [];
    let selection_instance = Selection::constructor(
        &selection_constructor.clone().into(),
        &selection_args,
        context,
    )?;

    Ok(selection_instance)
}

/// `window.performance` getter
fn get_performance(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a new Performance instance using the Performance constructor
    use crate::browser::performance::Performance;
    use boa_engine::builtins::IntrinsicObject;

    let performance_constructor = Performance::get(context.intrinsics());
    let performance_args = [];
    let performance_instance = Performance::constructor(
        &performance_constructor.clone().into(),
        &performance_args,
        context,
    )?;

    Ok(performance_instance)
}

/// `window.getComputedStyle(element, pseudoElement)` implementation
/// Returns a CSSStyleDeclaration object containing the computed styles for an element
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/getComputedStyle
fn get_computed_style(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::dom::element::ElementData;

    let element = args.get_or_undefined(0);
    let _pseudo_element = args.get_or_undefined(1); // Optional pseudo-element selector (::before, ::after)

    // Get the element object
    let element_obj = element.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("getComputedStyle: argument is not an element")
    })?;

    // Try to get ElementData from the object
    let element_data = element_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("getComputedStyle: argument is not an element")
    })?;

    // Get the element's computed style
    let css_style = element_data.get_style();

    // Create a CSSStyleDeclaration-like object with the computed styles
    let style_obj = JsObject::default(context.intrinsics());

    // Define default computed styles (browser defaults)
    let defaults = [
        ("display", "block"),
        ("position", "static"),
        ("visibility", "visible"),
        ("opacity", "1"),
        ("overflow", "visible"),
        ("boxSizing", "content-box"),
        ("margin", "0px"),
        ("marginTop", "0px"),
        ("marginRight", "0px"),
        ("marginBottom", "0px"),
        ("marginLeft", "0px"),
        ("padding", "0px"),
        ("paddingTop", "0px"),
        ("paddingRight", "0px"),
        ("paddingBottom", "0px"),
        ("paddingLeft", "0px"),
        ("border", "0px none rgb(0, 0, 0)"),
        ("borderWidth", "0px"),
        ("borderStyle", "none"),
        ("borderColor", "rgb(0, 0, 0)"),
        ("width", "auto"),
        ("height", "auto"),
        ("minWidth", "0px"),
        ("minHeight", "0px"),
        ("maxWidth", "none"),
        ("maxHeight", "none"),
        ("color", "rgb(0, 0, 0)"),
        ("backgroundColor", "rgba(0, 0, 0, 0)"),
        ("fontFamily", "serif"),
        ("fontSize", "16px"),
        ("fontWeight", "400"),
        ("fontStyle", "normal"),
        ("lineHeight", "normal"),
        ("textAlign", "start"),
        ("textDecoration", "none"),
        ("textTransform", "none"),
        ("whiteSpace", "normal"),
        ("wordSpacing", "0px"),
        ("letterSpacing", "normal"),
        ("cursor", "auto"),
        ("zIndex", "auto"),
        ("float", "none"),
        ("clear", "none"),
        ("transform", "none"),
        ("transition", "all 0s ease 0s"),
        ("animation", "none 0s ease 0s 1 normal none running"),
        ("flexDirection", "row"),
        ("flexWrap", "nowrap"),
        ("justifyContent", "normal"),
        ("alignItems", "normal"),
        ("alignContent", "normal"),
        ("order", "0"),
        ("flexGrow", "0"),
        ("flexShrink", "1"),
        ("flexBasis", "auto"),
        ("gridTemplateColumns", "none"),
        ("gridTemplateRows", "none"),
        ("gap", "normal"),
    ];

    // Set default values
    for (property, default_value) in defaults {
        style_obj.set(
            js_string!(property),
            JsValue::from(js_string!(default_value)),
            false,
            context,
        )?;
    }

    // Override with actual computed values from the element's style
    for (property, value) in css_style.iter_properties() {
        // Convert kebab-case to camelCase for JavaScript
        let camel_property = property
            .split('-')
            .enumerate()
            .map(|(i, part)| {
                if i == 0 {
                    part.to_string()
                } else {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                }
            })
            .collect::<String>();

        style_obj.set(
            JsString::from(camel_property.as_str()),
            JsValue::from(JsString::from(value.as_str())),
            false,
            context,
        )?;
    }

    // Add getPropertyValue method
    let get_property_value_fn = BuiltInBuilder::callable(context.realm(), computed_style_get_property_value)
        .name(js_string!("getPropertyValue"))
        .length(1)
        .build();

    style_obj.set(
        js_string!("getPropertyValue"),
        get_property_value_fn,
        false,
        context,
    )?;

    // Add length property (number of properties)
    style_obj.set(
        js_string!("length"),
        JsValue::from(defaults.len() as i32),
        false,
        context,
    )?;

    Ok(style_obj.into())
}

/// `CSSStyleDeclaration.getPropertyValue(property)` for computed styles
fn computed_style_get_property_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("getPropertyValue called on non-object")
    })?;

    let property = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Convert kebab-case to camelCase for lookup
    let camel_property = property
        .split('-')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            }
        })
        .collect::<String>();

    // Try to get the property value
    let value = this_obj.get(JsString::from(camel_property.as_str()), context)?;

    if value.is_undefined() {
        Ok(JsValue::from(js_string!("")))
    } else {
        Ok(value)
    }
}

/// `window.postMessage(message, targetOrigin, transfer)` implementation
/// Sends a cross-origin message to another window (or the same window)
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/postMessage
fn post_message(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = args.get_or_undefined(0).clone();
    let target_origin = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    let _transfer = args.get(2); // Optional transferable objects (not fully implemented)

    eprintln!("DEBUG: postMessage called with targetOrigin: {}", target_origin);

    // Create a MessageEvent with the posted message
    let message_event_constructor = context.intrinsics().constructors().message_event().constructor();

    // Create event init object
    let event_init = JsObject::default(context.intrinsics());
    event_init.set(js_string!("data"), message.clone(), false, context)?;
    event_init.set(js_string!("origin"), js_string!(target_origin.clone()), false, context)?;
    event_init.set(js_string!("source"), JsValue::null(), false, context)?;
    event_init.set(js_string!("bubbles"), JsValue::from(false), false, context)?;
    event_init.set(js_string!("cancelable"), JsValue::from(false), false, context)?;

    // Create the MessageEvent
    let event_args = [
        js_string!("message").into(),
        event_init.into(),
    ];

    let message_event = crate::events::message_event::MessageEvent::constructor(
        &message_event_constructor.clone().into(),
        &event_args,
        context,
    )?;

    // In a real browser, this would be dispatched asynchronously to the target window
    // For now, we dispatch it to the current window's event listeners immediately
    // This is a simplified implementation that handles same-origin messages

    // Get the window object to dispatch the event
    if let Ok(window_obj) = context.global_object().get(js_string!("window"), context) {
        if let Some(window) = window_obj.as_object() {
            if let Some(window_data) = window.downcast_ref::<WindowData>() {
                // Get message event listeners
                let listeners = window_data.event_listeners.lock().unwrap();
                if let Some(message_listeners) = listeners.get("message") {
                    for listener in message_listeners {
                        // Call each listener with the message event using the callable interface
                        if let Some(callable) = listener.as_callable() {
                            let _ = callable.call(&window_obj, &[message_event.clone()], context);
                        }
                    }
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// Getter for `window.self`, `window.parent`, `window.top`, `window.frames`
/// Returns the window object itself (for top-level window context)
fn get_window_self(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return the global window object
    context.global_object().get(js_string!("window"), context)
}

/// Scrolls the window to a particular position
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/scrollTo
fn scroll_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, scrolling is a no-op but we still need to accept the call
    // This supports both scrollTo(x, y) and scrollTo(options) signatures

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

    // Update scrollX and scrollY on the window object
    if let Ok(window_val) = context.global_object().get(js_string!("window"), context) {
        if let Some(window) = window_val.as_object() {
            let _ = window.set(js_string!("scrollX"), x, false, context);
            let _ = window.set(js_string!("scrollY"), y, false, context);
            let _ = window.set(js_string!("pageXOffset"), x, false, context);
            let _ = window.set(js_string!("pageYOffset"), y, false, context);
        }
    }

    Ok(JsValue::undefined())
}

/// Scrolls the window by a given amount
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/scrollBy
fn scroll_by(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get current scroll position
    let (current_x, current_y) = if let Ok(window_val) = context.global_object().get(js_string!("window"), context) {
        if let Some(window) = window_val.as_object() {
            let x = window.get(js_string!("scrollX"), context)
                .ok()
                .and_then(|v| v.to_number(context).ok())
                .unwrap_or(0.0);
            let y = window.get(js_string!("scrollY"), context)
                .ok()
                .and_then(|v| v.to_number(context).ok())
                .unwrap_or(0.0);
            (x, y)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };

    // Get delta values
    let (dx, dy) = if args.len() >= 2 {
        let dx = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);
        let dy = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
        (dx, dy)
    } else if let Some(options) = args.get(0).and_then(|v| v.as_object()) {
        let dx = options.get(js_string!("left"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        let dy = options.get(js_string!("top"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        (dx, dy)
    } else {
        (0.0, 0.0)
    };

    // Update scroll position
    if let Ok(window_val) = context.global_object().get(js_string!("window"), context) {
        if let Some(window) = window_val.as_object() {
            let new_x = current_x + dx;
            let new_y = current_y + dy;
            let _ = window.set(js_string!("scrollX"), new_x, false, context);
            let _ = window.set(js_string!("scrollY"), new_y, false, context);
            let _ = window.set(js_string!("pageXOffset"), new_x, false, context);
            let _ = window.set(js_string!("pageYOffset"), new_y, false, context);
        }
    }

    Ok(JsValue::undefined())
}