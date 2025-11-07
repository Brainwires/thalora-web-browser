//! Implementation of the Navigator interface.
//!
//! The Navigator interface represents the state and the identity of the user agent.
//! It allows scripts to query it and to register themselves to carry on some activities.
//!
//! More information:
//! - [WHATWG HTML Specification](https://html.spec.whatwg.org/multipage/system-state.html#the-navigator-object)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Navigator)

use boa_gc::{Finalize, Trace};
use boa_engine::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
};
use boa_engine::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use boa_engine::context::intrinsics::StandardConstructor;
// TODO: Implement web_locks module
// use crate::browser::web_locks::LockManagerObject;
use crate::worker::service_worker_container::ServiceWorkerContainer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Protocol handler storage for registerProtocolHandler
type ProtocolHandlers = Arc<Mutex<HashMap<String, String>>>;

/// `Navigator` object that provides information about the user agent and platform.
#[derive(Debug, Clone, Finalize)]
pub struct Navigator {
    // NavigatorID properties
    user_agent: String,
    app_code_name: String,
    app_name: String,
    app_version: String,
    platform: String,
    product: String,
    product_sub: String,
    vendor: String,
    vendor_sub: String,

    // NavigatorLanguage properties
    language: String,
    languages: Vec<String>,

    // NavigatorOnLine properties
    on_line: bool,

    // NavigatorCookies properties
    cookie_enabled: bool,

    // NavigatorPlugins properties (static/empty for security)
    plugins_length: usize,
    mime_types_length: usize,
    java_enabled: bool,
    pdf_viewer_enabled: bool,

    // Protocol handlers storage
    protocol_handlers: ProtocolHandlers,

    // Web Locks API
    lock_manager: Option<boa_engine::JsObject>,
}

unsafe impl Trace for Navigator {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Navigator, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Navigator, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Navigator
    }
}

impl JsData for Navigator {}

impl Navigator {
    pub(crate) fn new() -> Self {
        // Use shared USER_AGENT constant - single source of truth!
        use thalora_constants::USER_AGENT;

        Self {
            // NavigatorID - Chrome 120.0 on Windows 10 (WHATWG compliant)
            user_agent: USER_AGENT.to_string(),
            app_code_name: "Mozilla".to_string(),
            app_name: "Netscape".to_string(),
            app_version: "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            platform: "Win32".to_string(),
            product: "Gecko".to_string(),
            product_sub: "20030107".to_string(),
            vendor: "Google Inc.".to_string(),  // Chrome has Google Inc. as vendor
            vendor_sub: "".to_string(),

            // NavigatorLanguage
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],

            // NavigatorOnLine
            on_line: true,

            // NavigatorCookies
            cookie_enabled: true,

            // NavigatorPlugins - empty for security/privacy
            plugins_length: 0,
            mime_types_length: 0,
            java_enabled: false,
            pdf_viewer_enabled: true,

            // Protocol handlers
            protocol_handlers: Arc::new(Mutex::new(HashMap::new())),

            // Web Locks API
            lock_manager: None,
        }
    }

    /// Set the LockManager instance for navigator.locks
    pub fn set_lock_manager(&mut self, lock_manager: boa_engine::JsObject) {
        self.lock_manager = Some(lock_manager);
    }
}


impl IntrinsicObject for Navigator {
    fn init(realm: &Realm) {
        let locks_getter_func = BuiltInBuilder::callable(realm, Self::locks_getter)
            .name(js_string!("get locks"))
            .build();

        let service_worker_getter_func = BuiltInBuilder::callable(realm, Self::service_worker_getter)
            .name(js_string!("get serviceWorker"))
            .build();

        let languages_getter_func = BuiltInBuilder::callable(realm, Self::languages_getter)
            .name(js_string!("get languages"))
            .build();

        let plugins_getter_func = BuiltInBuilder::callable(realm, Self::plugins_getter)
            .name(js_string!("get plugins"))
            .build();

        let mime_types_getter_func = BuiltInBuilder::callable(realm, Self::mime_types_getter)
            .name(js_string!("get mimeTypes"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // NavigatorID properties
            .property(js_string!("userAgent"), js_string!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appCodeName"), js_string!("Mozilla"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appName"), js_string!("Netscape"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appVersion"), js_string!("5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("platform"), js_string!("MacIntel"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("product"), js_string!("Gecko"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("productSub"), js_string!("20030107"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendor"), js_string!("Google Inc."), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendorSub"), js_string!(""), Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorLanguage properties
            .property(js_string!("language"), js_string!("en-US"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .accessor(
                js_string!("languages"),
                Some(languages_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )

            // NavigatorOnLine properties
            .property(js_string!("onLine"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorConcurrentHardware properties
            .property(
                js_string!("hardwareConcurrency"),
                std::thread::available_parallelism()
                    .map(|n| n.get() as i32)
                    .unwrap_or(4),
                Attribute::READONLY | Attribute::NON_ENUMERABLE
            )

            // NavigatorCookies properties
            .property(js_string!("cookieEnabled"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorPlugins properties
            .accessor(
                js_string!("plugins"),
                Some(plugins_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("mimeTypes"),
                Some(mime_types_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .property(js_string!("pdfViewerEnabled"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorContentUtils methods
            .method(Self::register_protocol_handler, js_string!("registerProtocolHandler"), 2)
            .method(Self::unregister_protocol_handler, js_string!("unregisterProtocolHandler"), 2)
            .method(Self::java_enabled, js_string!("javaEnabled"), 0)

            // Web Locks and Service Workers
            // Note: locks is set as an instance property, not a prototype accessor
            .accessor(
                js_string!("serviceWorker"),
                Some(service_worker_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Navigator {
    const NAME: JsString = js_string!("Navigator");
}

impl BuiltInConstructor for Navigator {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.navigator();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Navigator constructor is not meant to be called directly
        Ok(JsValue::undefined())
    }
}

// Navigator prototype methods and getters
impl Navigator {
    /// `navigator.locks` getter
    fn locks_getter(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = _this
            .as_object()
            .ok_or_else(|| {
                boa_engine::JsNativeError::typ()
                    .with_message("Navigator.prototype.locks called on non-Navigator object")
            })?;

        let navigator = obj
            .downcast_ref::<Navigator>()
            .ok_or_else(|| {
                boa_engine::JsNativeError::typ()
                    .with_message("Navigator.prototype.locks called on non-Navigator object")
            })?;

        Ok(navigator.lock_manager.clone().map(|lm| lm.into()).unwrap_or(JsValue::undefined()))
    }

    /// `navigator.serviceWorker` getter
    fn service_worker_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Create and return a ServiceWorkerContainer instance
        let service_worker_container = ServiceWorkerContainer::create(context)?;
        Ok(JsValue::from(service_worker_container))
    }

    /// `navigator.languages` getter - returns array of language preferences
    fn languages_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let languages = vec![
            JsValue::from(js_string!("en-US")),
            JsValue::from(js_string!("en")),
        ];

        let array = boa_engine::builtins::array::Array::array_create(languages.len() as u64, None, context)?;

        for (i, lang) in languages.into_iter().enumerate() {
            array.create_data_property_or_throw(i, lang, context)?;
        }

        Ok(JsValue::from(array))
    }

    /// `navigator.plugins` getter - returns empty PluginArray for security
    fn plugins_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return empty plugin array for security/privacy
        // Arrays already have a length property set to 0
        let plugin_array = boa_engine::builtins::array::Array::array_create(0, None, context)?;
        Ok(JsValue::from(plugin_array))
    }

    /// `navigator.mimeTypes` getter - returns empty MimeTypeArray for security
    fn mime_types_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return empty mime types array for security/privacy
        // Arrays already have a length property set to 0
        let mime_array = boa_engine::builtins::array::Array::array_create(0, None, context)?;
        Ok(JsValue::from(mime_array))
    }

    /// `navigator.javaEnabled()` method - returns false for security
    fn java_enabled(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // Always return false for security/privacy
        Ok(JsValue::from(false))
    }

    /// `navigator.registerProtocolHandler(scheme, url)` method
    fn register_protocol_handler(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let scheme = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let url = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        // Basic validation - scheme should not be empty
        if scheme.is_empty() {
            return Err(boa_engine::JsNativeError::typ()
                .with_message("Failed to execute 'registerProtocolHandler' on 'Navigator': The scheme provided is not valid.")
                .into());
        }

        // URL should contain %s placeholder
        if !url.contains("%s") {
            return Err(boa_engine::JsNativeError::syntax()
                .with_message("Failed to execute 'registerProtocolHandler' on 'Navigator': The url provided does not contain a '%s' token.")
                .into());
        }

        // Store the protocol handler (in a real implementation, this would integrate with the browser)
        eprintln!("Protocol handler registered: {} -> {}", scheme, url);

        Ok(JsValue::undefined())
    }

    /// `navigator.unregisterProtocolHandler(scheme, url)` method
    fn unregister_protocol_handler(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let scheme = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let url = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        // Basic validation - scheme should not be empty
        if scheme.is_empty() {
            return Err(boa_engine::JsNativeError::typ()
                .with_message("Failed to execute 'unregisterProtocolHandler' on 'Navigator': The scheme provided is not valid.")
                .into());
        }

        // Store the protocol handler removal (in a real implementation, this would integrate with the browser)
        eprintln!("Protocol handler unregistered: {} -> {}", scheme, url);

        Ok(JsValue::undefined())
    }

    /// Create a Navigator instance for the global object
    pub fn create_navigator() -> JsObject {
        let navigator = Navigator::new();
        JsObject::from_proto_and_data(None, navigator)
    }
}

#[cfg(test)]
mod tests;
