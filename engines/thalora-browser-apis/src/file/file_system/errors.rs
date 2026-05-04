//! DOMException helper for the File System API.
//!
//! The WHATWG File System spec rejects promises with specific DOMException names
//! (`NotFoundError`, `TypeMismatchError`, `InvalidModificationError`, etc.). This
//! helper builds an Error-like JsObject with `name` and `message` properties so
//! `err.name === "NotFoundError"` works in user code.

use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsString, JsValue, js_string,
    property::PropertyDescriptor,
};

pub mod names {
    pub const NOT_FOUND: &str = "NotFoundError";
    pub const TYPE_MISMATCH: &str = "TypeMismatchError";
    pub const INVALID_MODIFICATION: &str = "InvalidModificationError";
    pub const NO_MODIFICATION_ALLOWED: &str = "NoModificationAllowedError";
    pub const QUOTA_EXCEEDED: &str = "QuotaExceededError";
    pub const ABORT: &str = "AbortError";
    pub const INVALID_STATE: &str = "InvalidStateError";
    pub const SECURITY: &str = "SecurityError";
    pub const SYNTAX: &str = "SyntaxError";
}

/// Build a DOMException-shaped JsValue suitable for promise rejection.
pub fn dom_exception(name: &str, message: &str, context: &mut Context) -> JsResult<JsValue> {
    let error_proto = context
        .intrinsics()
        .constructors()
        .error()
        .prototype();
    let obj = JsObject::with_object_proto(context.intrinsics());
    obj.set_prototype(Some(error_proto));

    obj.define_property_or_throw(
        js_string!("name"),
        PropertyDescriptor::builder()
            .value(JsValue::from(JsString::from(name)))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    obj.define_property_or_throw(
        js_string!("message"),
        PropertyDescriptor::builder()
            .value(JsValue::from(JsString::from(message)))
            .writable(true)
            .enumerable(false)
            .configurable(true)
            .build(),
        context,
    )?;
    Ok(obj.into())
}

/// Reject a freshly-created Promise with a DOMException, returning the Promise as JsValue.
/// Use this from any spec method that returns a Promise.
pub fn reject_with(name: &str, message: &str, context: &mut Context) -> JsResult<JsValue> {
    let err = dom_exception(name, message, context)?;
    let (promise, resolvers) = boa_engine::object::JsPromise::new_pending(context);
    resolvers
        .reject
        .call(&JsValue::undefined(), &[err], context)?;
    Ok(JsValue::from(promise))
}

/// Map an `io::Error` to the most appropriate DOMException name.
pub fn map_io_error(err: &std::io::Error) -> &'static str {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::NotFound => names::NOT_FOUND,
        ErrorKind::PermissionDenied => names::NO_MODIFICATION_ALLOWED,
        ErrorKind::AlreadyExists => names::TYPE_MISMATCH,
        ErrorKind::InvalidInput | ErrorKind::InvalidData => names::SYNTAX,
        _ => names::INVALID_STATE,
    }
}

/// Convenience: build a TypeError-bearing JsResult for argument validation that
/// the spec actually labels TypeError (not a DOMException).
pub fn type_error(message: &str) -> JsNativeError {
    JsNativeError::typ().with_message(message.to_string())
}
