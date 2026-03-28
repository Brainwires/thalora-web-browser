//! IDBRequest and IDBOpenDBRequest Implementation
//!
//! IDBRequest provides an interface to access results of asynchronous requests to databases and database objects.
//! Every operation that accesses the database asynchronously returns an IDBRequest object.
//!
//! Spec: https://w3c.github.io/IndexedDB/#request-api

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// Request state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestState {
    Pending,
    Done,
}

/// Shared request data
#[derive(Debug, Clone)]
struct RequestData {
    state: RequestState,
    result: Option<JsValue>,
    error: Option<String>,
}

/// IDBRequest represents an ongoing asynchronous operation
#[derive(Debug, Clone, Finalize)]
pub struct IDBRequest {
    /// Request state and result
    data: Arc<Mutex<RequestData>>,

    /// Source of the request (IDBObjectStore, IDBIndex, or IDBCursor)
    source: Option<JsObject>,

    /// Transaction this request belongs to
    transaction: Option<JsObject>,

    /// Event handlers
    onsuccess: Arc<Mutex<Option<JsObject>>>,

    onerror: Arc<Mutex<Option<JsObject>>>,
}

unsafe impl Trace for IDBRequest {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) { unsafe {
        if let Some(source) = &self.source {
            source.trace(tracer);
        }
        if let Some(transaction) = &self.transaction {
            transaction.trace(tracer);
        }
    }}

    unsafe fn trace_non_roots(&self) {
        // Trace non-root objects
    }

    fn run_finalizer(&self) {}
}

impl JsData for IDBRequest {}

impl IDBRequest {
    /// Create a new IDBRequest
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(RequestData {
                state: RequestState::Pending,
                result: None,
                error: None,
            })),
            source: None,
            transaction: None,
            onsuccess: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new IDBRequest with source and transaction
    pub fn with_context(source: Option<JsObject>, transaction: Option<JsObject>) -> Self {
        Self {
            data: Arc::new(Mutex::new(RequestData {
                state: RequestState::Pending,
                result: None,
                error: None,
            })),
            source,
            transaction,
            onsuccess: Arc::new(Mutex::new(None)),
            onerror: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the result and mark as done
    pub fn set_result(&self, result: JsValue) {
        let mut data = self.data.lock().unwrap();
        data.state = RequestState::Done;
        data.result = Some(result);
    }

    /// Set an error
    pub fn set_error(&self, error: String) {
        let mut data = self.data.lock().unwrap();
        data.state = RequestState::Done;
        data.error = Some(error);
    }

    /// Trigger success event
    pub fn trigger_success(&self, context: &mut Context) -> JsResult<()> {
        let handler = self.onsuccess.lock().unwrap();
        if let Some(callback) = handler.as_ref() {
            if callback.is_callable() {
                // Create success event
                let event = Self::create_event("success", context)?;
                callback.call(&JsValue::undefined(), &[event], context)?;
            }
        }
        Ok(())
    }

    /// Trigger error event
    pub fn trigger_error(&self, context: &mut Context) -> JsResult<()> {
        let handler = self.onerror.lock().unwrap();
        if let Some(callback) = handler.as_ref() {
            if callback.is_callable() {
                // Create error event
                let event = Self::create_event("error", context)?;
                callback.call(&JsValue::undefined(), &[event], context)?;
            }
        }
        Ok(())
    }

    /// Create a simple event object
    fn create_event(event_type: &str, context: &mut Context) -> JsResult<JsValue> {
        let event = JsObject::with_object_proto(context.intrinsics());
        event.set(
            js_string!("type"),
            JsValue::from(JsString::from(event_type)),
            false,
            context,
        )?;
        Ok(event.into())
    }

    /// Get readyState property
    pub(crate) fn get_ready_state(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Handle both IDBRequest and IDBOpenDBRequest
        let state_str = if let Some(obj) = this.as_object() {
            if let Some(request) = obj.downcast_ref::<IDBRequest>() {
                let data = request.data.lock().unwrap();
                match data.state {
                    RequestState::Pending => "pending",
                    RequestState::Done => "done",
                }
            } else if let Some(open_request) = obj.downcast_ref::<IDBOpenDBRequest>() {
                let data = open_request.base.data.lock().unwrap();
                match data.state {
                    RequestState::Pending => "pending",
                    RequestState::Done => "done",
                }
            } else {
                return Err(JsNativeError::typ()
                    .with_message("'this' is not an IDBRequest object")
                    .into());
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("'this' is not an IDBRequest object")
                .into());
        };

        Ok(JsValue::from(JsString::from(state_str)))
    }

    /// Get result property
    pub(crate) fn get_result(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Handle both IDBRequest and IDBOpenDBRequest
        if let Some(obj) = this.as_object() {
            if let Some(request) = obj.downcast_ref::<IDBRequest>() {
                let data = request.data.lock().unwrap();
                if data.state == RequestState::Pending {
                    return Err(JsNativeError::error()
                        .with_message("Request is still pending")
                        .into());
                }
                Ok(data.result.clone().unwrap_or(JsValue::undefined()))
            } else if let Some(open_request) = obj.downcast_ref::<IDBOpenDBRequest>() {
                let data = open_request.base.data.lock().unwrap();
                if data.state == RequestState::Pending {
                    return Err(JsNativeError::error()
                        .with_message("Request is still pending")
                        .into());
                }
                Ok(data.result.clone().unwrap_or(JsValue::undefined()))
            } else {
                Err(JsNativeError::typ()
                    .with_message("'this' is not an IDBRequest object")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("'this' is not an IDBRequest object")
                .into())
        }
    }

    /// Get error property
    pub(crate) fn get_error(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Handle both IDBRequest and IDBOpenDBRequest
        if let Some(obj) = this.as_object() {
            if let Some(request) = obj.downcast_ref::<IDBRequest>() {
                let data = request.data.lock().unwrap();
                if let Some(error) = &data.error {
                    Ok(JsValue::from(JsString::from(error.clone())))
                } else {
                    Ok(JsValue::null())
                }
            } else if let Some(open_request) = obj.downcast_ref::<IDBOpenDBRequest>() {
                let data = open_request.base.data.lock().unwrap();
                if let Some(error) = &data.error {
                    Ok(JsValue::from(JsString::from(error.clone())))
                } else {
                    Ok(JsValue::null())
                }
            } else {
                Err(JsNativeError::typ()
                    .with_message("'this' is not an IDBRequest object")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("'this' is not an IDBRequest object")
                .into())
        }
    }

    /// Get source property
    pub(crate) fn get_source(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get source from the underlying request (not from RequestData)
        if let Some(obj) = this.as_object() {
            if let Some(request) = obj.downcast_ref::<IDBRequest>() {
                Ok(request
                    .source
                    .clone()
                    .map(|s| s.into())
                    .unwrap_or(JsValue::null()))
            } else if let Some(open_request) = obj.downcast_ref::<IDBOpenDBRequest>() {
                Ok(open_request
                    .base
                    .source
                    .clone()
                    .map(|s| s.into())
                    .unwrap_or(JsValue::null()))
            } else {
                Err(JsNativeError::typ()
                    .with_message("'this' is not an IDBRequest object")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("'this' is not an IDBRequest object")
                .into())
        }
    }

    /// Get transaction property
    pub(crate) fn get_transaction(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get transaction from the underlying request (works for both IDBRequest and IDBOpenDBRequest)
        if let Some(obj) = this.as_object() {
            if let Some(request) = obj.downcast_ref::<IDBRequest>() {
                Ok(request
                    .transaction
                    .clone()
                    .map(|t| t.into())
                    .unwrap_or(JsValue::null()))
            } else if let Some(open_request) = obj.downcast_ref::<IDBOpenDBRequest>() {
                Ok(open_request
                    .base
                    .transaction
                    .clone()
                    .map(|t| t.into())
                    .unwrap_or(JsValue::null()))
            } else {
                Err(JsNativeError::typ()
                    .with_message("'this' is not an IDBRequest object")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("'this' is not an IDBRequest object")
                .into())
        }
    }

    /// Set onsuccess handler
    pub(crate) fn set_onsuccess(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBRequest object")
        })?;

        let request = obj.downcast_ref::<IDBRequest>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBRequest object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onsuccess = request.onsuccess.lock().unwrap();

        if handler.is_callable() {
            *onsuccess = handler.as_object().map(|obj| obj.clone());
        } else {
            *onsuccess = None;
        }

        Ok(JsValue::undefined())
    }

    /// Set onerror handler
    pub(crate) fn set_onerror(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBRequest object")
        })?;

        let request = obj.downcast_ref::<IDBRequest>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBRequest object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onerror = request.onerror.lock().unwrap();

        if handler.is_callable() {
            *onerror = handler.as_object().map(|obj| obj.clone());
        } else {
            *onerror = None;
        }

        Ok(JsValue::undefined())
    }
}

impl Default for IDBRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl IntrinsicObject for IDBRequest {
    fn init(realm: &Realm) {
        let _intrinsics = realm.intrinsics();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("readyState"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_ready_state)
                        .name(js_string!("get readyState"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("result"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_result)
                        .name(js_string!("get result"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("error"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_error)
                        .name(js_string!("get error"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("source"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_source)
                        .name(js_string!("get source"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("transaction"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_transaction)
                        .name(js_string!("get transaction"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onsuccess"),
                None,
                Some(
                    BuiltInBuilder::callable(realm, Self::set_onsuccess)
                        .name(js_string!("set onsuccess"))
                        .build(),
                ),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                None,
                Some(
                    BuiltInBuilder::callable(realm, Self::set_onerror)
                        .name(js_string!("set onerror"))
                        .build(),
                ),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBRequest {
    const NAME: JsString = js_string!("IDBRequest");
}

impl BuiltInConstructor for IDBRequest {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100; // Estimated prototype property count
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100; // Constructor properties

    const CONSTRUCTOR_ARGUMENTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |constructors| constructors.idb_request();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBRequest cannot be constructed directly
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

/// IDBOpenDBRequest extends IDBRequest for database opening operations
#[derive(Debug, Clone, Finalize)]
pub struct IDBOpenDBRequest {
    /// Base request
    base: IDBRequest,

    /// onupgradeneeded handler
    onupgradeneeded: Arc<Mutex<Option<JsObject>>>,

    /// onblocked handler
    onblocked: Arc<Mutex<Option<JsObject>>>,
}

unsafe impl Trace for IDBOpenDBRequest {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) { unsafe {
        self.base.trace(tracer);
    }}

    unsafe fn trace_non_roots(&self) {}

    fn run_finalizer(&self) {}
}

impl JsData for IDBOpenDBRequest {}

impl IDBOpenDBRequest {
    /// Create a new IDBOpenDBRequest
    pub fn new() -> Self {
        Self {
            base: IDBRequest::new(),
            onupgradeneeded: Arc::new(Mutex::new(None)),
            onblocked: Arc::new(Mutex::new(None)),
        }
    }

    /// Get base request
    pub fn base(&self) -> &IDBRequest {
        &self.base
    }

    /// Trigger upgradeneeded event
    pub fn trigger_upgradeneeded(
        &self,
        old_version: u32,
        new_version: u32,
        context: &mut Context,
    ) -> JsResult<()> {
        let handler = self.onupgradeneeded.lock().unwrap();
        if let Some(callback) = handler.as_ref() {
            if callback.is_callable() {
                // Create version change event
                let event = JsObject::with_object_proto(context.intrinsics());
                event.set(
                    js_string!("type"),
                    JsValue::from(JsString::from("upgradeneeded")),
                    false,
                    context,
                )?;
                event.set(
                    js_string!("oldVersion"),
                    JsValue::from(old_version),
                    false,
                    context,
                )?;
                event.set(
                    js_string!("newVersion"),
                    JsValue::from(new_version),
                    false,
                    context,
                )?;

                callback.call(&JsValue::undefined(), &[event.into()], context)?;
            }
        }
        Ok(())
    }

    /// Set onupgradeneeded handler
    pub(crate) fn set_onupgradeneeded(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBRequest object")
        })?;

        let request = obj.downcast_ref::<IDBOpenDBRequest>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBOpenDBRequest object")
        })?;

        let handler = args.get_or_undefined(0);
        let mut onupgradeneeded = request.onupgradeneeded.lock().unwrap();

        if handler.is_callable() {
            *onupgradeneeded = handler.as_object().map(|obj| obj.clone());
        } else {
            *onupgradeneeded = None;
        }

        Ok(JsValue::undefined())
    }
}

impl Default for IDBOpenDBRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_state_transitions() {
        let request = IDBRequest::new();

        // Initial state
        {
            let data = request.data.lock().unwrap();
            assert_eq!(data.state, RequestState::Pending);
            assert!(data.result.is_none());
            assert!(data.error.is_none());
        }

        // Set result
        request.set_result(JsValue::from(42));
        {
            let data = request.data.lock().unwrap();
            assert_eq!(data.state, RequestState::Done);
            assert!(data.result.is_some());
        }
    }

    #[test]
    fn test_request_error() {
        let request = IDBRequest::new();

        request.set_error("Test error".to_string());

        let data = request.data.lock().unwrap();
        assert_eq!(data.state, RequestState::Done);
        assert_eq!(data.error, Some("Test error".to_string()));
    }
}
