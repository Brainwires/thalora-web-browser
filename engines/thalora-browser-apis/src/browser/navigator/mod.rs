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
use crate::browser::clipboard;
use crate::browser::permissions;
use crate::browser::vibration;
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
            // NavigatorID - Chrome 131.0 on Windows 10 (WHATWG compliant)
            user_agent: USER_AGENT.to_string(),
            app_code_name: "Mozilla".to_string(),
            app_name: "Netscape".to_string(),
            app_version: "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
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

        let clipboard_getter_func = BuiltInBuilder::callable(realm, Self::clipboard_getter)
            .name(js_string!("get clipboard"))
            .build();

        let permissions_getter_func = BuiltInBuilder::callable(realm, Self::permissions_getter)
            .name(js_string!("get permissions"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // NavigatorID properties - use shared constants for consistency
            .property(js_string!("userAgent"), js_string!(thalora_constants::USER_AGENT), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appCodeName"), js_string!("Mozilla"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appName"), js_string!("Netscape"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appVersion"), js_string!("5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("platform"), js_string!("Win32"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("product"), js_string!("Gecko"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("productSub"), js_string!("20030107"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendor"), js_string!("Google Inc."), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendorSub"), js_string!(""), Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // Additional properties Cloudflare checks
            .property(js_string!("deviceMemory"), JsValue::from(8), Attribute::CONFIGURABLE)
            .property(js_string!("maxTouchPoints"), JsValue::from(0), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("webdriver"), JsValue::from(false), Attribute::CONFIGURABLE)

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

            // Vibration API
            .method(Self::vibrate, js_string!("vibrate"), 1)

            // Beacon API
            .method(Self::send_beacon, js_string!("sendBeacon"), 2)

            // Web Locks and Service Workers
            // Note: locks is set as an instance property, not a prototype accessor
            .accessor(
                js_string!("serviceWorker"),
                Some(service_worker_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            // Clipboard API
            .accessor(
                js_string!("clipboard"),
                Some(clipboard_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            // Permissions API
            .accessor(
                js_string!("permissions"),
                Some(permissions_getter_func),
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

    /// `navigator.clipboard` getter
    fn clipboard_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Create and return a Clipboard instance
        let clipboard_obj = clipboard::create_clipboard(context)?;
        Ok(JsValue::from(clipboard_obj))
    }

    /// `navigator.permissions` getter
    fn permissions_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Create and return a Permissions instance
        let permissions_obj = permissions::create_permissions(context)?;
        Ok(JsValue::from(permissions_obj))
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

    /// `navigator.plugins` getter - returns Chrome-like PluginArray
    /// Cloudflare checks navigator.plugins.length > 0
    fn plugins_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Chrome reports 5 plugins (PDF Viewer, Chrome PDF Viewer, etc.)
        let plugin_array = boa_engine::builtins::array::Array::array_create(5, None, context)?;

        let plugin_names = [
            "PDF Viewer",
            "Chrome PDF Viewer",
            "Chromium PDF Viewer",
            "Microsoft Edge PDF Viewer",
            "WebKit built-in PDF",
        ];
        for (i, name) in plugin_names.iter().enumerate() {
            let plugin = JsObject::default(context.intrinsics());
            plugin.set(js_string!("name"), js_string!(*name), false, context)?;
            plugin.set(js_string!("description"), js_string!("Portable Document Format"), false, context)?;
            plugin.set(js_string!("filename"), js_string!("internal-pdf-viewer"), false, context)?;
            plugin.set(js_string!("length"), JsValue::from(1), false, context)?;
            plugin_array.create_data_property_or_throw(i, plugin, context)?;
        }

        Ok(JsValue::from(plugin_array))
    }

    /// `navigator.mimeTypes` getter - returns Chrome-like MimeTypeArray
    fn mime_types_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Chrome reports 2 mime types for PDF
        let mime_array = boa_engine::builtins::array::Array::array_create(2, None, context)?;

        let mime_types = [
            ("application/pdf", "Portable Document Format", "pdf"),
            ("text/pdf", "Portable Document Format", "pdf"),
        ];
        for (i, (type_str, desc, suffixes)) in mime_types.iter().enumerate() {
            let mime = JsObject::default(context.intrinsics());
            mime.set(js_string!("type"), js_string!(*type_str), false, context)?;
            mime.set(js_string!("description"), js_string!(*desc), false, context)?;
            mime.set(js_string!("suffixes"), js_string!(*suffixes), false, context)?;
            mime_array.create_data_property_or_throw(i, mime, context)?;
        }

        Ok(JsValue::from(mime_array))
    }

    /// `navigator.javaEnabled()` method - returns false for security
    fn java_enabled(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // Always return false for security/privacy
        Ok(JsValue::from(false))
    }

    /// `navigator.vibrate(pattern)` method - vibrates the device
    fn vibrate(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        vibration::navigator_vibrate(_this, args, context)
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

    /// `navigator.sendBeacon(url, data?)` method - sends a small amount of data asynchronously
    ///
    /// The Beacon API is used to send analytics and diagnostics to a web server.
    /// It returns true if the user agent successfully queued the data for transfer.
    /// https://developer.mozilla.org/en-US/docs/Web/API/Navigator/sendBeacon
    /// https://w3c.github.io/beacon/
    fn send_beacon(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let url = args.get_or_undefined(0);

        // URL is required
        if url.is_undefined() || url.is_null() {
            return Err(boa_engine::JsNativeError::typ()
                .with_message("Failed to execute 'sendBeacon' on 'Navigator': 1 argument required, but only 0 present.")
                .into());
        }

        let url_str = url.to_string(context)?.to_std_string_escaped();

        // Validate URL - must be valid and use http/https
        let parsed_url = match url::Url::parse(&url_str) {
            Ok(u) => u,
            Err(_) => {
                return Err(boa_engine::JsNativeError::typ()
                    .with_message("Failed to execute 'sendBeacon' on 'Navigator': The URL provided is invalid.")
                    .into());
            }
        };

        // Per spec, only http and https are allowed
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(boa_engine::JsNativeError::typ()
                .with_message("Failed to execute 'sendBeacon' on 'Navigator': Beacons are only supported over HTTP(S).")
                .into());
        }

        // Get optional data and determine content type
        let data = args.get_or_undefined(1);
        let (body_data, content_type): (Vec<u8>, &str) = if data.is_undefined() || data.is_null() {
            (Vec::new(), "text/plain;charset=UTF-8")
        } else if let Some(data_string) = data.as_string() {
            // String data
            (data_string.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
        } else if let Some(obj) = data.as_object() {
            // Check for Blob
            if let Ok(blob_type) = obj.get(js_string!("type"), context) {
                if let Some(type_str) = blob_type.as_string() {
                    let mime = type_str.to_std_string_escaped();
                    // Try to get arrayBuffer or text from Blob
                    if let Ok(text_val) = obj.get(js_string!("_data"), context) {
                        if let Some(text) = text_val.as_string() {
                            (text.to_std_string_escaped().into_bytes(),
                             if mime.is_empty() { "application/octet-stream" } else { Box::leak(mime.into_boxed_str()) })
                        } else {
                            (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                        }
                    } else {
                        (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                    }
                } else {
                    (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                }
            }
            // Check for FormData
            else if let Ok(entries) = obj.get(js_string!("_entries"), context) {
                if !entries.is_undefined() {
                    // FormData - serialize as URL-encoded
                    (data.to_string(context)?.to_std_string_escaped().into_bytes(), "application/x-www-form-urlencoded")
                } else {
                    (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                }
            }
            // Check for URLSearchParams
            else if let Ok(to_string) = obj.get(js_string!("toString"), context) {
                if to_string.is_callable() {
                    if let Ok(result) = to_string.as_callable().unwrap().call(&data, &[], context) {
                        (result.to_string(context)?.to_std_string_escaped().into_bytes(), "application/x-www-form-urlencoded")
                    } else {
                        (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                    }
                } else {
                    (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
                }
            } else {
                (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
            }
        } else {
            (data.to_string(context)?.to_std_string_escaped().into_bytes(), "text/plain;charset=UTF-8")
        };

        // Per spec: payload must be <= 64KB
        if body_data.len() > 65536 {
            // Return false if payload too large (per spec)
            return Ok(JsValue::from(false));
        }

        // Spawn async task to send the beacon
        // The key behavior of sendBeacon is that it returns immediately and sends in the background
        #[cfg(feature = "native")]
        {
            let url_clone = url_str.clone();
            let content_type_owned = content_type.to_string();

            std::thread::spawn(move || {
                // Use a blocking runtime for the HTTP request
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(_) => return,
                };

                rt.block_on(async {
                    // Build and send the POST request
                    let client = match rquest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .build()
                    {
                        Ok(c) => c,
                        Err(_) => return,
                    };

                    let _ = client
                        .post(&url_clone)
                        .header("Content-Type", content_type_owned)
                        .body(body_data)
                        .send()
                        .await;

                    // We don't care about the response - beacon is fire-and-forget
                });
            });
        }

        // Return true - the beacon was successfully queued
        Ok(JsValue::from(true))
    }

    /// Create a Navigator instance for the global object
    pub fn create_navigator() -> JsObject {
        let navigator = Navigator::new();
        JsObject::from_proto_and_data(None, navigator)
    }
}

#[cfg(test)]
mod tests;
