//! Worker Web API implementation for Boa
//!
//! Native implementation of Worker standard
//! https://html.spec.whatwg.org/multipage/workers.html
//!
//! This implements the complete Worker interface with real JavaScript execution

use boa_engine::{
    Context, JsResult, JsValue, JsNativeError, JsArgs, js_string, JsString,
    object::JsObject,
    property::Attribute,
    builtins::{BuiltInObject, IntrinsicObject, BuiltInBuilder, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    realm::Realm,
    string::StaticJsStrings,
    JsData,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::sync::{Arc, Mutex};

use crate::worker::worker_thread::{WorkerThread, WorkerConfig, WorkerType, WorkerCommand, WorkerEvent};
use crate::misc::structured_clone::{structured_clone, structured_deserialize, StructuredCloneValue};

/// Worker object data
#[derive(Trace, Finalize, JsData)]
pub struct Worker {
    /// The actual worker thread
    #[unsafe_ignore_trace]
    worker_thread: Arc<Mutex<Option<WorkerThread>>>,
    /// Script URL
    script_url: String,
    /// Worker options
    #[unsafe_ignore_trace]
    options: WorkerOptions,
    /// Message event handler
    onmessage: GcRefCell<Option<JsObject>>,
    /// Error event handler
    onerror: GcRefCell<Option<JsObject>>,
    /// Message error event handler
    onmessageerror: GcRefCell<Option<JsObject>>,
}

/// Worker construction options
#[derive(Debug, Clone)]
pub struct WorkerOptions {
    pub name: Option<String>,
    pub worker_type: WorkerType,
}

impl Default for WorkerOptions {
    fn default() -> Self {
        Self {
            name: None,
            worker_type: WorkerType::Classic,
        }
    }
}

impl Worker {
    /// Create a new Worker instance
    pub fn new(script_url: String, options: WorkerOptions) -> JsResult<Self> {
        // Create worker configuration
        let config = WorkerConfig {
            name: options.name.clone(),
            script_url: script_url.clone(),
            worker_type: options.worker_type,
            stack_size: Some(2 * 1024 * 1024),
        };

        // Spawn the worker thread
        let worker_thread = WorkerThread::spawn(config)?;

        Ok(Self {
            worker_thread: Arc::new(Mutex::new(Some(worker_thread))),
            script_url,
            options,
            onmessage: GcRefCell::new(None),
            onerror: GcRefCell::new(None),
            onmessageerror: GcRefCell::new(None),
        })
    }

    /// Create a JS object from Worker data
    pub fn create_js_object(self, context: &mut Context) -> JsResult<JsObject> {
        let proto = context.intrinsics().constructors().worker().prototype();
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            self,
        );
        Ok(object)
    }

    /// Parse worker options from JavaScript object
    fn parse_options(opts: &JsValue, context: &mut Context) -> JsResult<WorkerOptions> {
        let mut options = WorkerOptions::default();

        if let Some(obj) = opts.as_object() {
            // Get name
            if let Ok(name_val) = obj.get(js_string!("name"), context) {
                if !name_val.is_undefined() {
                    options.name = Some(name_val.to_string(context)?.to_std_string_escaped());
                }
            }

            // Get type
            if let Ok(type_val) = obj.get(js_string!("type"), context) {
                if !type_val.is_undefined() {
                    let type_str = type_val.to_string(context)?.to_std_string_escaped();
                    options.worker_type = match type_str.as_str() {
                        "module" => WorkerType::Module,
                        _ => WorkerType::Classic,
                    };
                }
            }
        }

        Ok(options)
    }


    /// Poll for events from the worker and dispatch them
    pub fn poll_events(&self, worker_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        if let Ok(worker_thread_lock) = self.worker_thread.lock() {
            if let Some(worker_thread) = worker_thread_lock.as_ref() {
                // Process all pending events
                while let Some(event) = worker_thread.try_recv_event() {
                    match event {
                        WorkerEvent::Message { data } => {
                            self.dispatch_message_event(worker_obj, data, context)?;
                        }
                        WorkerEvent::Error { message, filename, lineno, colno } => {
                            self.dispatch_error_event(worker_obj, message, filename, lineno, colno, context)?;
                        }
                        WorkerEvent::Terminated => {
                            // Worker terminated
                        }
                        _ => {
                            // Other events (Started, ScriptExecuted, etc.) don't need to be exposed to JS
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Dispatch a message event
    fn dispatch_message_event(
        &self,
        worker_obj: &JsObject,
        data: StructuredCloneValue,
        context: &mut Context,
    ) -> JsResult<()> {
        // Deserialize the message data
        let deserialized = structured_deserialize(&data, context)?;

        // Create MessageEvent
        let event = crate::events::message_event::create_message_event(
            deserialized,
            Some("worker"),
            Some(worker_obj.clone().into()),
            None,
            context,
        )?;

        // Call onmessage handler if set
        if let Some(handler) = self.onmessage.borrow().as_ref() {
            if handler.is_callable() {
                let _ = handler.call(&JsValue::from(worker_obj.clone()), &[event.clone().into()], context);
            }
        }

        Ok(())
    }

    /// Dispatch an error event
    fn dispatch_error_event(
        &self,
        worker_obj: &JsObject,
        message: String,
        _filename: String,
        _lineno: u32,
        _colno: u32,
        context: &mut Context,
    ) -> JsResult<()> {
        // Create error object using JsNativeError
        let error_obj = JsNativeError::error()
            .with_message(message)
            .to_opaque(context);

        // Call onerror handler if set
        if let Some(handler) = self.onerror.borrow().as_ref() {
            if handler.is_callable() {
                let _ = handler.call(&JsValue::from(worker_obj.clone()), &[error_obj.into()], context);
            }
        }

        Ok(())
    }
}

/// Worker constructor function
fn worker_constructor(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get script URL (required)
    let script_url = args.get_or_undefined(0);
    if script_url.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("Worker constructor requires a script URL")
            .into());
    }

    let script_url_str = script_url
        .to_string(context)?
        .to_std_string_escaped();

    // Parse options (optional second argument)
    let options = if let Some(opts) = args.get(1) {
        Worker::parse_options(opts, context)?
    } else {
        WorkerOptions::default()
    };

    // Create the worker
    let worker = Worker::new(script_url_str, options)?;

    // Create the JavaScript object
    let worker_obj = worker.create_js_object(context)?;

    Ok(worker_obj.into())
}

/// `Worker.prototype.postMessage(message)`
fn post_message(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Worker.postMessage called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("Worker.postMessage called on wrong type"))?;

    // Get the message
    let message = args.get_or_undefined(0);

    // Structured clone the message
    let cloned = structured_clone(message, context, None)?;

    // Send to worker thread
    if let Ok(worker_thread_lock) = worker.worker_thread.lock() {
        if let Some(ref worker_thread) = *worker_thread_lock {
            worker_thread.send_command(WorkerCommand::PostMessage { message: cloned })
                .map_err(|e| JsNativeError::error().with_message(format!("Failed to post message: {:?}", e)))?;
        } else {
            return Err(JsNativeError::error()
                .with_message("Worker has been terminated")
                .into());
        }
    }

    Ok(JsValue::undefined())
}

/// `Worker.prototype.terminate()`
fn terminate(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Worker.terminate called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("Worker.terminate called on wrong type"))?;

    // Terminate the worker
    if let Ok(mut worker_thread_lock) = worker.worker_thread.lock() {
        if let Some(mut worker_thread) = worker_thread_lock.take() {
            worker_thread.terminate();
        }
    }

    Ok(JsValue::undefined())
}

/// JavaScript `Worker` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WorkerConstructor;

impl BuiltInObject for WorkerConstructor {
    const NAME: JsString = StaticJsStrings::WORKER;
}

impl IntrinsicObject for WorkerConstructor {
    fn init(realm: &Realm) {
        // Build the Worker constructor
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("postMessage"),
                BuiltInBuilder::callable(realm, post_message)
                    .name(js_string!("postMessage"))
                    .length(1)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("terminate"),
                BuiltInBuilder::callable(realm, terminate)
                    .name(js_string!("terminate"))
                    .length(0)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onmessage"),
                Some(BuiltInBuilder::callable(realm, get_onmessage).build()),
                Some(BuiltInBuilder::callable(realm, set_onmessage).build()),
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                Some(BuiltInBuilder::callable(realm, get_onerror).build()),
                Some(BuiltInBuilder::callable(realm, set_onerror).build()),
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for WorkerConstructor {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::worker;

    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        worker_constructor(args, context)
    }
}

/// Getter for onmessage
fn get_onmessage(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("get onmessage called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("get onmessage called on wrong type"))?;

    let handler = worker.onmessage.borrow().clone();
    Ok(handler.map_or(JsValue::null(), |h| h.into()))
}

/// Setter for onmessage
fn set_onmessage(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("set onmessage called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("set onmessage called on wrong type"))?;

    let handler = args.get_or_undefined(0);
    if handler.is_callable() {
        *worker.onmessage.borrow_mut() = handler.as_object().map(|o| o.clone());
    } else if handler.is_null() || handler.is_undefined() {
        *worker.onmessage.borrow_mut() = None;
    }

    Ok(JsValue::undefined())
}

/// Getter for onerror
fn get_onerror(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("get onerror called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("get onerror called on wrong type"))?;

    let handler = worker.onerror.borrow().clone();
    Ok(handler.map_or(JsValue::null(), |h| h.into()))
}

/// Setter for onerror
fn set_onerror(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let worker_obj = this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("set onerror called on non-object"))?;

    let worker = worker_obj.downcast_ref::<Worker>()
        .ok_or_else(|| JsNativeError::typ().with_message("set onerror called on wrong type"))?;

    let handler = args.get_or_undefined(0);
    if handler.is_callable() {
        *worker.onerror.borrow_mut() = handler.as_object().map(|o| o.clone());
    } else if handler.is_null() || handler.is_undefined() {
        *worker.onerror.borrow_mut() = None;
    }

    Ok(JsValue::undefined())
}

/// Register the Worker constructor in a context
pub fn register_worker_api(context: &mut Context) -> JsResult<()> {
    // Initialize Worker in the global object
    let global = context.global_object();

    WorkerConstructor::init(context.realm());
    let worker_constructor = WorkerConstructor::get(context.intrinsics());

    global.set(
        js_string!("Worker"),
        worker_constructor,
        false,
        context,
    )?;

    Ok(())
}
