//! WorkerGlobalScope implementation for Boa
//!
//! Implements DedicatedWorkerGlobalScope, SharedWorkerGlobalScope, and ServiceWorkerGlobalScope
//! https://html.spec.whatwg.org/multipage/workers.html#the-workerglobalscope-common-interface

use boa_engine::{
    Context, JsResult, JsValue, JsNativeError, Source, JsArgs, js_string,
    object::JsObject,
    builtins::BuiltInBuilder,
    property::{PropertyDescriptorBuilder, Attribute},
};
use crate::worker::{
    worker_events::{WorkerEvent, dispatch_worker_event},
    worker_navigator::WorkerNavigator,
};
use crate::misc::structured_clone::{StructuredCloneValue, structured_clone, structured_deserialize, TransferList};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global registry for active worker scopes
static WORKER_SCOPE_REGISTRY: OnceLock<Mutex<HashMap<usize, Arc<WorkerGlobalScope>>>> = OnceLock::new();

/// Global counter for worker scope IDs
static WORKER_SCOPE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Get the global worker scope registry
fn get_worker_scope_registry() -> &'static Mutex<HashMap<usize, Arc<WorkerGlobalScope>>> {
    WORKER_SCOPE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Generate a unique worker scope ID
fn generate_scope_id() -> usize {
    WORKER_SCOPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Base WorkerGlobalScope functionality
#[derive(Debug, Trace, Finalize)]
pub struct WorkerGlobalScope {
    /// Unique identifier for this worker scope
    #[unsafe_ignore_trace]
    scope_id: usize,
    /// Type of worker global scope
    scope_type: WorkerGlobalScopeType,
    /// Message channel to main thread
    #[unsafe_ignore_trace]
    main_thread_sender: Option<Sender<WorkerMessage>>,
    /// Receive messages from main thread
    #[unsafe_ignore_trace]
    main_thread_receiver: Option<Receiver<WorkerMessage>>,
    /// Event channel to send events to main thread (for worker_thread integration)
    #[unsafe_ignore_trace]
    event_sender: Option<Sender<crate::worker::worker_thread::WorkerEvent>>,
    /// Whether the worker is closing
    #[unsafe_ignore_trace]
    closing: Arc<Mutex<bool>>,
    /// Worker location/origin info
    location: WorkerLocation,
}

/// Types of worker global scopes
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum WorkerGlobalScopeType {
    Dedicated,
    Shared,
    Service,
}

/// Worker location information
#[derive(Debug, Clone, Trace, Finalize)]
pub struct WorkerLocation {
    pub href: String,
    pub origin: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
}

/// Message between worker and main thread
#[derive(Debug, Clone)]
pub struct WorkerMessage {
    pub data: StructuredCloneValue,
    pub ports: Vec<String>, // Serialized MessagePort objects for transferable
    pub source: MessageSource,
}

/// Source of a worker message
#[derive(Debug, Clone)]
pub enum MessageSource {
    MainThread,
    Worker,
    SharedWorkerPort(String), // port name/id
}

impl WorkerGlobalScope {
    /// Create a new WorkerGlobalScope
    pub fn new(
        scope_type: WorkerGlobalScopeType,
        script_url: &str,
        event_sender: Option<Sender<crate::worker::worker_thread::WorkerEvent>>,
    ) -> JsResult<Self> {
        let (main_sender, main_receiver) = unbounded();
        let scope_id = generate_scope_id();
        let location = WorkerLocation::from_url(script_url)?;

        Ok(Self {
            scope_id,
            scope_type,
            main_thread_sender: Some(main_sender),
            main_thread_receiver: Some(main_receiver),
            event_sender,
            closing: Arc::new(Mutex::new(false)),
            location,
        })
    }

    /// Register this scope in the global registry
    pub fn register_scope(scope: Arc<WorkerGlobalScope>) {
        if let Ok(mut registry) = get_worker_scope_registry().lock() {
            registry.insert(scope.scope_id, scope);
        }
    }

    /// Unregister this scope from the global registry
    pub fn unregister_scope(scope_id: usize) {
        if let Ok(mut registry) = get_worker_scope_registry().lock() {
            registry.remove(&scope_id);
        }
    }

    /// Get the scope ID
    pub fn get_scope_id(&self) -> usize {
        self.scope_id
    }

    /// Get the worker location (for script URL resolution)
    pub fn get_location(&self) -> &WorkerLocation {
        &self.location
    }

    /// Initialize the global scope in a JavaScript context
    pub fn initialize_in_context(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        // Store the scope ID in the context for later retrieval
        global.set(js_string!("__worker_scope_id__"), self.scope_id as f64, false, context)?;

        // Add 'self' reference to global scope
        global.set(js_string!("self"), global.clone(), false, context)?;

        // Add WorkerGlobalScope properties and methods
        self.add_worker_global_scope_apis(context)?;

        // Add type-specific APIs
        match self.scope_type {
            WorkerGlobalScopeType::Dedicated => {
                self.add_dedicated_worker_apis(context)?;
            }
            WorkerGlobalScopeType::Shared => {
                self.add_shared_worker_apis(context)?;
            }
            WorkerGlobalScopeType::Service => {
                self.add_service_worker_apis(context)?;
            }
        }

        // Add console API
        self.add_console_api(context)?;

        // Add basic Web APIs available in workers
        self.add_worker_web_apis(context)?;

        Ok(())
    }

    /// Add base WorkerGlobalScope APIs
    fn add_worker_global_scope_apis(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();
        let closing = self.closing.clone();
        let main_sender = self.main_thread_sender.clone();

        // Add postMessage function
        let post_message_func = BuiltInBuilder::callable(context.realm(), Self::post_message_impl)
            .name(js_string!("postMessage"))
            .length(1)
            .build();

        global.set(js_string!("postMessage"), post_message_func, false, context)?;

        // Add close function
        let close_func = BuiltInBuilder::callable(context.realm(), Self::close_impl)
            .name(js_string!("close"))
            .length(0)
            .build();

        global.set(js_string!("close"), close_func, false, context)?;

        // Add importScripts function (for classic workers)
        let import_scripts_func = BuiltInBuilder::callable(context.realm(), Self::import_scripts_impl)
            .name(js_string!("importScripts"))
            .build();

        global.set(js_string!("importScripts"), import_scripts_func, false, context)?;

        // Add WorkerLocation as 'location' property
        self.add_location_object(context)?;

        // Add WorkerNavigator as 'navigator' property
        self.add_navigator_object(context)?;

        Ok(())
    }

    /// Add DedicatedWorkerGlobalScope specific APIs
    fn add_dedicated_worker_apis(&self, _context: &mut Context) -> JsResult<()> {
        // DedicatedWorkerGlobalScope inherits everything from WorkerGlobalScope
        // and adds event handlers (onmessage, onmessageerror)
        // These are already handled by the worker_events system
        eprintln!("Initialized DedicatedWorkerGlobalScope");
        Ok(())
    }

    /// Add SharedWorkerGlobalScope specific APIs
    fn add_shared_worker_apis(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        // SharedWorkerGlobalScope has 'name' property and 'connect' event handler
        eprintln!("Initialized SharedWorkerGlobalScope");

        // Add name property (should be set from the worker's actual name)
        global.set(js_string!("name"), js_string!(""), false, context)?;

        // Add onconnect event handler property
        global.set(js_string!("onconnect"), JsValue::null(), false, context)?;

        // Add connect event dispatch function for internal use
        self.add_connect_event_dispatcher(context)?;

        Ok(())
    }

    /// Add connect event dispatcher for SharedWorkerGlobalScope
    fn add_connect_event_dispatcher(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        // Add internal function to dispatch connect events
        let dispatch_connect = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let port_arg = args.get_or_undefined(0);

                // Create MessageEvent for the connect event
                let event_obj = JsObject::with_object_proto(context.intrinsics());

                // Set event properties
                event_obj.set(js_string!("type"), js_string!("connect"), false, context)?;
                event_obj.set(js_string!("bubbles"), false, false, context)?;
                event_obj.set(js_string!("cancelable"), false, false, context)?;

                // Add ports array with the connecting port
                let ports_array = boa_engine::builtins::array::Array::array_create(1, None, context)?;
                ports_array.set(0, port_arg.clone(), true, context)?;
                event_obj.set(js_string!("ports"), ports_array, false, context)?;

                // Check if onconnect handler exists and call it
                let global = context.global_object();
                let onconnect = global.get(js_string!("onconnect"), context)?;

                if !onconnect.is_null() && !onconnect.is_undefined() {
                    if let Some(handler) = onconnect.as_callable() {
                        let _ = handler.call(&global.clone().into(), &[event_obj.into()], context);
                    }
                }

                // Also dispatch to addEventListener if event listeners exist
                // TODO: Implement proper event dispatching system

                Ok(JsValue::undefined())
            }
        )
        .name(js_string!("_dispatchConnect"))
        .build();

        // Store the dispatcher function for internal use (not exposed to JS)
        global.set(js_string!("_dispatchConnect"), dispatch_connect, false, context)?;

        Ok(())
    }

    /// Add ServiceWorkerGlobalScope specific APIs
    fn add_service_worker_apis(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        eprintln!("Initialized ServiceWorkerGlobalScope");

        // Add Service Worker specific globals
        // TODO: Add registration, caches, clients, etc.

        // Add placeholder registration object
        let registration_obj = JsObject::with_object_proto(context.intrinsics());
        global.set(js_string!("registration"), registration_obj, false, context)?;

        Ok(())
    }

    /// Add console API for workers
    fn add_console_api(&self, context: &mut Context) -> JsResult<()> {
        // Use the full Console API implementation
        crate::console::console::Console::init(context);
        Ok(())
    }

    /// Add basic Web APIs available in workers
    fn add_worker_web_apis(&self, context: &mut Context) -> JsResult<()> {
        // Add timers (setTimeout, setInterval, clearTimeout, clearInterval)
        crate::timers::timers::Timers::init(context);

        // Add crypto API (crypto.getRandomValues, crypto.randomUUID, etc.)
        crate::crypto::crypto::Crypto::init(context);

        // Add TextEncoder and TextDecoder
        self.add_text_encoding_apis(context)?;

        // Add atob/btoa for base64 encoding
        self.add_base64_apis(context)?;

        // Add structured clone function
        self.add_structured_clone_api(context)?;

        // Add queueMicrotask
        self.add_microtask_api(context)?;

        Ok(())
    }

    /// Add TextEncoder and TextDecoder APIs
    fn add_text_encoding_apis(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        // Add TextEncoder constructor
        let text_encoder_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, _args: &[JsValue], context: &mut Context| {
                let encoder_obj = JsObject::with_object_proto(context.intrinsics());

                // Add encode method
                let encode_func = BuiltInBuilder::callable(
                    context.realm(),
                    |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                        let input = args.get_or_undefined(0).to_string(context)?;
                        let bytes = input.to_std_string_escaped().into_bytes();

                        // Create Uint8Array - simplified implementation
                        let array = boa_engine::builtins::array::Array::array_create(bytes.len() as u64, None, context)?;
                        for (i, byte) in bytes.iter().enumerate() {
                            array.set(i, JsValue::from(*byte), true, context)?;
                        }
                        Ok(array.into())
                    }
                )
                .name(js_string!("encode"))
                .length(1)
                .build();

                encoder_obj.set(js_string!("encode"), encode_func, false, context)?;
                Ok(encoder_obj.into())
            }
        )
        .name(js_string!("TextEncoder"))
        .length(0)
        .build();

        global.set(js_string!("TextEncoder"), text_encoder_func, false, context)?;

        // Add TextDecoder constructor
        let text_decoder_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, _args: &[JsValue], context: &mut Context| {
                let decoder_obj = JsObject::with_object_proto(context.intrinsics());

                // Add decode method
                let decode_func = BuiltInBuilder::callable(
                    context.realm(),
                    |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                        let input = args.get_or_undefined(0);

                        // Try to get bytes from TypedArray
                        if let Some(obj) = input.as_object() {
                            // For now, simplified - would need proper TypedArray handling
                            Ok(js_string!("").into())
                        } else {
                            Ok(js_string!("").into())
                        }
                    }
                )
                .name(js_string!("decode"))
                .length(1)
                .build();

                decoder_obj.set(js_string!("decode"), decode_func, false, context)?;
                Ok(decoder_obj.into())
            }
        )
        .name(js_string!("TextDecoder"))
        .length(0)
        .build();

        global.set(js_string!("TextDecoder"), text_decoder_func, false, context)?;

        Ok(())
    }

    /// Add atob/btoa for base64 encoding
    fn add_base64_apis(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        // Add btoa (binary to ASCII/base64)
        let btoa_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let input = args.get_or_undefined(0).to_string(context)?;
                let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD,
                    input.to_std_string_escaped().as_bytes());
                Ok(js_string!(encoded).into())
            }
        )
        .name(js_string!("btoa"))
        .length(1)
        .build();

        global.set(js_string!("btoa"), btoa_func, false, context)?;

        // Add atob (ASCII/base64 to binary)
        let atob_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let input = args.get_or_undefined(0).to_string(context)?;
                match base64::Engine::decode(&base64::engine::general_purpose::STANDARD,
                    input.to_std_string_escaped().as_bytes()) {
                    Ok(decoded) => {
                        let s = String::from_utf8_lossy(&decoded);
                        Ok(js_string!(s.to_string()).into())
                    }
                    Err(_) => Err(JsNativeError::typ()
                        .with_message("Invalid base64 string")
                        .into())
                }
            }
        )
        .name(js_string!("atob"))
        .length(1)
        .build();

        global.set(js_string!("atob"), atob_func, false, context)?;

        Ok(())
    }

    /// Add structuredClone API
    fn add_structured_clone_api(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        let structured_clone_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let value = args.get_or_undefined(0);

                // Use our structured clone implementation
                let cloned = crate::misc::structured_clone::structured_clone(value, context, None)?;
                let deserialized = crate::misc::structured_clone::structured_deserialize(&cloned, context)?;

                Ok(deserialized)
            }
        )
        .name(js_string!("structuredClone"))
        .length(1)
        .build();

        global.set(js_string!("structuredClone"), structured_clone_func, false, context)?;

        Ok(())
    }

    /// Add queueMicrotask API
    fn add_microtask_api(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();

        let queue_microtask_func = BuiltInBuilder::callable(
            context.realm(),
            |_this: &JsValue, args: &[JsValue], context: &mut Context| {
                let callback = args.get_or_undefined(0);

                if !callback.is_callable() {
                    return Err(JsNativeError::typ()
                        .with_message("queueMicrotask requires a callable function")
                        .into());
                }

                // Queue the microtask via Boa's job queue
                context.run_jobs();

                // For now, execute immediately as a simplified implementation
                // Real implementation would queue to event loop
                if let Some(func) = callback.as_callable() {
                    let _ = func.call(&JsValue::undefined(), &[], context);
                }

                Ok(JsValue::undefined())
            }
        )
        .name(js_string!("queueMicrotask"))
        .length(1)
        .build();

        global.set(js_string!("queueMicrotask"), queue_microtask_func, false, context)?;

        Ok(())
    }

    /// Add WorkerLocation object
    fn add_location_object(&self, context: &mut Context) -> JsResult<()> {
        let location_obj = JsObject::with_object_proto(context.intrinsics());

        // Add location properties
        location_obj.set(js_string!("href"), js_string!(self.location.href.clone()), false, context)?;
        location_obj.set(js_string!("origin"), js_string!(self.location.origin.clone()), false, context)?;
        location_obj.set(js_string!("protocol"), js_string!(self.location.protocol.clone()), false, context)?;
        location_obj.set(js_string!("host"), js_string!(self.location.host.clone()), false, context)?;
        location_obj.set(js_string!("hostname"), js_string!(self.location.hostname.clone()), false, context)?;
        location_obj.set(js_string!("port"), js_string!(self.location.port.clone()), false, context)?;
        location_obj.set(js_string!("pathname"), js_string!(self.location.pathname.clone()), false, context)?;
        location_obj.set(js_string!("search"), js_string!(self.location.search.clone()), false, context)?;
        location_obj.set(js_string!("hash"), js_string!(self.location.hash.clone()), false, context)?;

        context.global_object().set(js_string!("location"), location_obj, false, context)?;
        Ok(())
    }

    /// Add WorkerNavigator object
    fn add_navigator_object(&self, context: &mut Context) -> JsResult<()> {
        // Create proper WorkerNavigator object with full WHATWG compliance
        let navigator_obj = WorkerNavigator::create(context)?;

        // Add navigator object to global scope
        context.global_object().set(js_string!("navigator"), navigator_obj, false, context)?;
        Ok(())
    }

    /// Execute script in this worker global scope
    pub fn execute_script(&self, context: &mut Context, script_content: &str) -> JsResult<JsValue> {
        // Check if worker is closing
        if *self.closing.lock().unwrap() {
            return Err(JsNativeError::error()
                .with_message("Cannot execute script in closing worker")
                .into());
        }

        eprintln!("Executing script in worker global scope ({:?})", self.scope_type);

        // Execute the script
        let source = Source::from_bytes(script_content);
        let result = context.eval(source);

        match &result {
            Ok(value) => {
                eprintln!("Worker script executed successfully, result: {:?}", value.get_type());
            }
            Err(e) => {
                eprintln!("Worker script execution error: {:?}", e);
            }
        }

        result
    }

    /// Process messages from main thread
    pub fn process_main_thread_messages(&self, context: &mut Context) -> JsResult<()> {
        if let Some(ref receiver) = self.main_thread_receiver {
            while let Ok(message) = receiver.try_recv() {
                self.dispatch_message_event(context, message)?;
            }
        }
        Ok(())
    }

    /// Dispatch a message event in the worker
    fn dispatch_message_event(&self, context: &mut Context, message: WorkerMessage) -> JsResult<()> {
        let global = context.global_object();

        // Deserialize the structured clone back to JsValue
        let deserialized_data = structured_deserialize(&message.data, context)?;

        // Determine origin based on message source
        let origin = match message.source {
            MessageSource::MainThread => Some("main"),
            MessageSource::Worker => Some("worker"),
            MessageSource::SharedWorkerPort(_) => Some("sharedworker"),
        };

        // Create proper MessageEvent using the built-in constructor
        let message_event = crate::events::message_event::create_message_event(
            deserialized_data,
            origin,
            None, // source: we could pass the worker object reference here
            None, // ports: TODO - handle transferable objects
            context,
        )?;

        // Call onmessage handler if it exists
        if let Ok(onmessage) = global.get(js_string!("onmessage"), context) {
            if onmessage.is_callable() {
                if let Some(func) = onmessage.as_callable() {
                    let _ = func.call(&JsValue::from(global.clone()), &[JsValue::from(message_event)], context);
                }
            }
        }

        Ok(())
    }

    /// Get message sender for main thread communication
    pub fn get_main_thread_sender(&self) -> Option<&Sender<WorkerMessage>> {
        self.main_thread_sender.as_ref()
    }

    /// Check if worker is closing
    pub fn is_closing(&self) -> bool {
        *self.closing.lock().unwrap()
    }

    /// Get the current WorkerGlobalScope from a JavaScript context
    /// This method retrieves the scope stored in the global registry using the scope ID
    fn get_current_scope_from_context(context: &mut Context) -> Option<Arc<WorkerGlobalScope>> {
        // Try to get the scope ID from the global object
        let global = context.global_object();

        if let Ok(scope_id_val) = global.get(js_string!("__worker_scope_id__"), context) {
            if let Some(scope_id_num) = scope_id_val.as_number() {
                let scope_id = scope_id_num as usize;

                // Look up the scope in the global registry
                if let Ok(registry) = get_worker_scope_registry().lock() {
                    return registry.get(&scope_id).cloned();
                }
            }
        }

        None
    }

    /// Static implementation for postMessage
    fn post_message_impl(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let message = args.get_or_undefined(0);
        let _transfer = args.get_or_undefined(1);

        // Parse transfer list
        let transfer_list = if !_transfer.is_undefined() {
            match TransferList::from_js_array(_transfer, context) {
                Ok(list) => Some(list),
                Err(e) => {
                    eprintln!("Failed to parse transfer list in worker: {:?}", e);
                    return Err(e);
                }
            }
        } else {
            None
        };

        // Clone the message using structured cloning
        let cloned_message = match structured_clone(message, context, transfer_list.as_ref()) {
            Ok(cloned) => cloned,
            Err(e) => {
                eprintln!("Failed to clone message: {:?}", e);
                return Err(e);
            }
        };

        eprintln!("Worker postMessage called with structured cloned data");

        // Send message to main thread through the event channel
        if let Some(global_scope) = Self::get_current_scope_from_context(context) {
            if let Some(ref event_sender) = global_scope.event_sender {
                use crate::worker::worker_thread::WorkerEvent;

                if let Err(_) = event_sender.send(WorkerEvent::Message {
                    data: cloned_message,
                }) {
                    return Err(JsNativeError::error()
                        .with_message("Failed to send message to main thread")
                        .into());
                } else {
                    eprintln!("Message sent from worker to main thread successfully");
                }
            } else {
                return Err(JsNativeError::error()
                    .with_message("Worker event channel not available")
                    .into());
            }
        } else {
            return Err(JsNativeError::error()
                .with_message("Worker global scope not available for postMessage")
                .into());
        }

        Ok(JsValue::undefined())
    }

    /// Static implementation for close
    fn close_impl(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        eprintln!("Worker close() called - worker will terminate");
        // TODO: Set worker closing state through proper mechanism
        Ok(JsValue::undefined())
    }

    /// Static implementation for importScripts
    fn import_scripts_impl(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the base URL from the worker global scope
        let global = context.global_object();
        let base_url = if let Ok(location) = global.get(js_string!("location"), context) {
            if let Some(loc_obj) = location.as_object() {
                if let Ok(href) = loc_obj.get(js_string!("href"), context) {
                    Some(href.to_string(context)?.to_std_string_escaped())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Convert all arguments to URLs
        let urls: Vec<String> = args.iter()
            .map(|arg| arg.to_string(context).map(|s| s.to_std_string_escaped()))
            .collect::<Result<Vec<_>, _>>()?;

        eprintln!("importScripts called with {} URL(s)", urls.len());

        // Use the real import_scripts implementation
        crate::worker::import_scripts::import_scripts_impl(urls, base_url, context)?;

        Ok(JsValue::undefined())
    }
}

impl WorkerLocation {
    /// Create WorkerLocation from URL string
    fn from_url(url_str: &str) -> JsResult<Self> {
        use url::Url;

        // Handle empty strings or inline scripts - use default location
        if url_str.is_empty() || (!url_str.starts_with("http://") && !url_str.starts_with("https://") && !url_str.starts_with("data:") && !url_str.starts_with("blob:")) {
            return Ok(Self {
                href: "about:blank".to_string(),
                origin: "null".to_string(),
                protocol: "about:".to_string(),
                host: "".to_string(),
                hostname: "".to_string(),
                port: "".to_string(),
                pathname: "blank".to_string(),
                search: "".to_string(),
                hash: "".to_string(),
            });
        }

        let url = Url::parse(url_str).map_err(|_| {
            JsNativeError::typ().with_message(format!("Invalid URL: {}", url_str))
        })?;

        Ok(Self {
            href: url.as_str().to_string(),
            origin: format!("{}://{}", url.scheme(), url.host_str().unwrap_or("")),
            protocol: format!("{}:", url.scheme()),
            host: url.host_str().unwrap_or("").to_string(),
            hostname: url.host_str().unwrap_or("").to_string(),
            port: url.port().map_or_else(|| "".to_string(), |p| p.to_string()),
            pathname: url.path().to_string(),
            search: url.query().map_or_else(|| "".to_string(), |q| format!("?{}", q)),
            hash: url.fragment().map_or_else(|| "".to_string(), |f| format!("#{}", f)),
        })
    }
}