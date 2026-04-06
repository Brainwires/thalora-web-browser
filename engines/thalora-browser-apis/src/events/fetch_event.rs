//! FetchEvent implementation for Boa
//!
//! Implements the FetchEvent interface as defined in:
//! https://w3c.github.io/ServiceWorker/#fetchevent-interface
//!
//! FetchEvent is dispatched to a service worker's global scope when a fetch
//! request is intercepted by the service worker.

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, JsPromise, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::cell::Cell;

/// JavaScript `FetchEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct FetchEvent;

impl IntrinsicObject for FetchEvent {
    fn init(realm: &Realm) {
        let request_getter = BuiltInBuilder::callable(realm, get_request)
            .name(js_string!("get request"))
            .build();

        let client_id_getter = BuiltInBuilder::callable(realm, get_client_id)
            .name(js_string!("get clientId"))
            .build();

        let resulting_client_id_getter = BuiltInBuilder::callable(realm, get_resulting_client_id)
            .name(js_string!("get resultingClientId"))
            .build();

        let replaces_client_id_getter = BuiltInBuilder::callable(realm, get_replaces_client_id)
            .name(js_string!("get replacesClientId"))
            .build();

        let handled_getter = BuiltInBuilder::callable(realm, get_handled)
            .name(js_string!("get handled"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("request"),
                Some(request_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientId"),
                Some(client_id_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("resultingClientId"),
                Some(resulting_client_id_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("replacesClientId"),
                Some(replaces_client_id_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("handled"),
                Some(handled_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(respond_with, js_string!("respondWith"), 1)
            .method(wait_until, js_string!("waitUntil"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for FetchEvent {
    const NAME: JsString = StaticJsStrings::FETCH_EVENT;
}

impl BuiltInConstructor for FetchEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::fetch_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("FetchEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::fetch_event, context)?;

        // Parse init dict for request, clientId, etc.
        let mut request_obj = None;
        let mut client_id = String::new();
        let mut resulting_client_id = String::new();
        let mut replaces_client_id = String::new();

        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            // Get request (required per spec)
            if let Ok(req_val) = init_obj.get(js_string!("request"), context)
                && req_val.is_object()
            {
                request_obj = req_val.as_object();
            }

            // Get clientId
            if let Ok(cid_val) = init_obj.get(js_string!("clientId"), context)
                && !cid_val.is_undefined()
            {
                client_id = cid_val.to_string(context)?.to_std_string_escaped();
            }

            // Get resultingClientId
            if let Ok(rcid_val) = init_obj.get(js_string!("resultingClientId"), context)
                && !rcid_val.is_undefined()
            {
                resulting_client_id = rcid_val.to_string(context)?.to_std_string_escaped();
            }

            // Get replacesClientId
            if let Ok(rpcid_val) = init_obj.get(js_string!("replacesClientId"), context)
                && !rpcid_val.is_undefined()
            {
                replaces_client_id = rpcid_val.to_string(context)?.to_std_string_escaped();
            }
        }

        let fetch_event_data = FetchEventData::new(
            event_type.to_std_string_escaped(),
            request_obj,
            client_id,
            resulting_client_id,
            replaces_client_id,
        );
        let fetch_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            fetch_event_data,
        );

        let fetch_event_generic = fetch_event_obj.upcast();

        // Set Event interface properties
        fetch_event_generic.set(js_string!("type"), event_type, false, context)?;
        fetch_event_generic.set(js_string!("bubbles"), false, false, context)?;
        fetch_event_generic.set(js_string!("cancelable"), true, false, context)?;
        fetch_event_generic.set(js_string!("composed"), false, false, context)?;
        fetch_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        fetch_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        fetch_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        fetch_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        fetch_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        fetch_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        // Parse bubbles/cancelable from init dict
        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                fetch_event_generic.set(
                    js_string!("bubbles"),
                    bubbles_val.to_boolean(),
                    false,
                    context,
                )?;
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                fetch_event_generic.set(
                    js_string!("cancelable"),
                    cancelable_val.to_boolean(),
                    false,
                    context,
                )?;
            }
        }

        Ok(fetch_event_generic.into())
    }
}

/// Internal data for FetchEvent instances
#[derive(Debug, Trace, Finalize, JsData)]
pub(crate) struct FetchEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    /// The request associated with this fetch event
    request: GcRefCell<Option<JsObject>>,
    /// The response provided via respondWith()
    response: GcRefCell<Option<JsValue>>,
    /// Whether respondWith() has been called
    #[unsafe_ignore_trace]
    respond_with_called: Cell<bool>,
    /// Client ID string
    #[unsafe_ignore_trace]
    client_id: String,
    /// Resulting client ID string
    #[unsafe_ignore_trace]
    resulting_client_id: String,
    /// Replaces client ID string
    #[unsafe_ignore_trace]
    replaces_client_id: String,
    /// Promises passed to waitUntil()
    wait_until_promises: GcRefCell<Vec<JsValue>>,
}

impl FetchEventData {
    fn new(
        event_type: String,
        request: Option<JsObject>,
        client_id: String,
        resulting_client_id: String,
        replaces_client_id: String,
    ) -> Self {
        Self {
            event_type,
            request: GcRefCell::new(request),
            response: GcRefCell::new(None),
            respond_with_called: Cell::new(false),
            client_id,
            resulting_client_id,
            replaces_client_id,
            wait_until_promises: GcRefCell::new(Vec::new()),
        }
    }
}

/// `FetchEvent.prototype.request` getter
fn get_request(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FetchEvent.prototype.request called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.request called on non-FetchEvent object")
    })?;

    let request = fetch_event.request.borrow();
    match request.as_ref() {
        Some(req) => Ok(req.clone().into()),
        None => Ok(JsValue::undefined()),
    }
}

/// `FetchEvent.prototype.clientId` getter
fn get_client_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FetchEvent.prototype.clientId called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.clientId called on non-FetchEvent object")
    })?;

    Ok(js_string!(fetch_event.client_id.clone()).into())
}

/// `FetchEvent.prototype.resultingClientId` getter
fn get_resulting_client_id(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.resultingClientId called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.resultingClientId called on non-FetchEvent object")
    })?;

    Ok(js_string!(fetch_event.resulting_client_id.clone()).into())
}

/// `FetchEvent.prototype.replacesClientId` getter
fn get_replaces_client_id(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.replacesClientId called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.replacesClientId called on non-FetchEvent object")
    })?;

    Ok(js_string!(fetch_event.replaces_client_id.clone()).into())
}

/// `FetchEvent.prototype.handled` getter - returns a Promise that resolves when respondWith is called
fn get_handled(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FetchEvent.prototype.handled called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.handled called on non-FetchEvent object")
    })?;

    // Return a resolved promise if respondWith was already called, pending otherwise
    if fetch_event.respond_with_called.get() {
        Ok(JsPromise::resolve(JsValue::undefined(), context)?.into())
    } else {
        // Return a pending promise (will be resolved when respondWith is called)
        let (promise, _resolvers) = JsPromise::new_pending(context);
        Ok(promise.into())
    }
}

/// `FetchEvent.prototype.respondWith(response)` - provide a response for this fetch
fn respond_with(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FetchEvent.prototype.respondWith called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.respondWith called on non-FetchEvent object")
    })?;

    // Per spec, respondWith can only be called once
    if fetch_event.respond_with_called.get() {
        return Err(JsNativeError::error()
            .with_message("FetchEvent.respondWith() has already been called")
            .into());
    }

    let response_arg = args.get_or_undefined(0);

    // Store the response (could be a Response object or a Promise<Response>)
    *fetch_event.response.borrow_mut() = Some(response_arg.clone());
    fetch_event.respond_with_called.set(true);

    Ok(JsValue::undefined())
}

/// `FetchEvent.prototype.waitUntil(promise)` - extend the event lifetime
fn wait_until(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FetchEvent.prototype.waitUntil called on non-object")
    })?;

    let fetch_event = this_obj.downcast_ref::<FetchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("FetchEvent.prototype.waitUntil called on non-FetchEvent object")
    })?;

    let promise_arg = args.get_or_undefined(0);
    fetch_event
        .wait_until_promises
        .borrow_mut()
        .push(promise_arg.clone());

    Ok(JsValue::undefined())
}

/// Check if a FetchEvent had respondWith() called, and if so, return the response value
pub fn get_fetch_event_response(event_obj: &JsObject) -> Option<JsValue> {
    let fetch_event = event_obj.downcast_ref::<FetchEventData>()?;
    if fetch_event.respond_with_called.get() {
        fetch_event.response.borrow().clone()
    } else {
        None
    }
}

/// Create a FetchEvent with a Request object for service worker dispatch
pub fn create_fetch_event(
    request_url: &str,
    request_method: &str,
    request_headers: &std::collections::HashMap<String, String>,
    client_id: &str,
    context: &mut Context,
) -> JsResult<JsObject> {
    // Create a Request-like object
    let request_obj = JsObject::with_object_proto(context.intrinsics());
    request_obj.set(js_string!("url"), js_string!(request_url), false, context)?;
    request_obj.set(
        js_string!("method"),
        js_string!(request_method),
        false,
        context,
    )?;
    request_obj.set(js_string!("mode"), js_string!("navigate"), false, context)?;
    request_obj.set(
        js_string!("credentials"),
        js_string!("same-origin"),
        false,
        context,
    )?;
    request_obj.set(
        js_string!("destination"),
        js_string!("document"),
        false,
        context,
    )?;
    request_obj.set(js_string!("redirect"), js_string!("follow"), false, context)?;
    request_obj.set(
        js_string!("referrer"),
        js_string!("about:client"),
        false,
        context,
    )?;

    // Create headers object on the request
    let headers_obj = JsObject::with_object_proto(context.intrinsics());
    for (key, value) in request_headers {
        headers_obj.set(
            js_string!(key.as_str()),
            js_string!(value.as_str()),
            false,
            context,
        )?;
    }

    // Add headers.get() method using NativeFunction::from_closure
    // (captures a JsObject which is not Send, so needs unsafe)
    let headers_clone = headers_obj.clone();
    let get_header_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, args, context| {
            let key = args.get_or_undefined(0).to_string(context)?;
            let val = headers_clone.get(key, context)?;
            if val.is_undefined() {
                Ok(JsValue::null())
            } else {
                Ok(val)
            }
        })
    };
    headers_obj.set(
        js_string!("get"),
        get_header_fn.to_js_function(context.realm()),
        false,
        context,
    )?;

    request_obj.set(js_string!("headers"), headers_obj, false, context)?;

    // Add clone() method to request
    let req_url = request_url.to_string();
    let req_method = request_method.to_string();
    let req_headers = request_headers.clone();
    // SAFETY: Captured values are plain String/HashMap, no GC-traced values
    let clone_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, _args, context| {
            let cloned = create_fetch_event_request(&req_url, &req_method, &req_headers, context)?;
            Ok(cloned.into())
        })
    };
    request_obj.set(
        js_string!("clone"),
        clone_fn.to_js_function(context.realm()),
        false,
        context,
    )?;

    // Create the FetchEvent init dict
    let init_obj = JsObject::with_object_proto(context.intrinsics());
    init_obj.set(js_string!("request"), request_obj, false, context)?;
    init_obj.set(
        js_string!("clientId"),
        js_string!(client_id),
        false,
        context,
    )?;
    init_obj.set(
        js_string!("resultingClientId"),
        js_string!(""),
        false,
        context,
    )?;
    init_obj.set(
        js_string!("replacesClientId"),
        js_string!(""),
        false,
        context,
    )?;

    // Construct the FetchEvent using the intrinsic constructor
    let fetch_event_constructor = context
        .intrinsics()
        .constructors()
        .fetch_event()
        .constructor();

    let fetch_event = fetch_event_constructor.construct(
        &[js_string!("fetch").into(), init_obj.into()],
        Some(&fetch_event_constructor),
        context,
    )?;

    // Mark as trusted since this is browser-dispatched
    fetch_event.set(js_string!("isTrusted"), true, false, context)?;

    Ok(fetch_event)
}

/// Helper to create a Request-like object (used by clone())
fn create_fetch_event_request(
    url: &str,
    method: &str,
    headers: &std::collections::HashMap<String, String>,
    context: &mut Context,
) -> JsResult<JsObject> {
    let request_obj = JsObject::with_object_proto(context.intrinsics());
    request_obj.set(js_string!("url"), js_string!(url), false, context)?;
    request_obj.set(js_string!("method"), js_string!(method), false, context)?;
    request_obj.set(js_string!("mode"), js_string!("navigate"), false, context)?;
    request_obj.set(
        js_string!("credentials"),
        js_string!("same-origin"),
        false,
        context,
    )?;

    let headers_obj = JsObject::with_object_proto(context.intrinsics());
    for (key, value) in headers {
        headers_obj.set(
            js_string!(key.as_str()),
            js_string!(value.as_str()),
            false,
            context,
        )?;
    }
    request_obj.set(js_string!("headers"), headers_obj, false, context)?;

    Ok(request_obj)
}
