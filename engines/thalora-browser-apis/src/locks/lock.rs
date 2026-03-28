//! Implementation of the `Lock` interface.

use boa_engine::{Context, JsData, JsNativeError, JsResult, JsString, JsValue};
use boa_gc::{Finalize, Trace};

/// Lock mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Exclusive,
    Shared,
}

impl LockMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LockMode::Exclusive => "exclusive",
            LockMode::Shared => "shared",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "shared" => LockMode::Shared,
            _ => LockMode::Exclusive, // Default to exclusive
        }
    }
}

/// `Lock` interface representing a held lock.
#[derive(Debug, Clone, Finalize)]
pub struct Lock {
    /// The name of the locked resource
    name: String,
    /// The lock mode (exclusive or shared)
    mode: LockMode,
}

unsafe impl Trace for Lock {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed
    }
}

impl JsData for Lock {}

impl Lock {
    /// Creates a new `Lock` instance.
    pub fn new(name: String, mode: LockMode) -> Self {
        Self { name, mode }
    }

    /// Gets the lock name.
    fn name_getter(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Lock.prototype.name called on non-Lock object")
        })?;

        let lock = obj.downcast_ref::<Lock>().ok_or_else(|| {
            JsNativeError::typ().with_message("Lock.prototype.name called on non-Lock object")
        })?;

        Ok(JsValue::from(JsString::from(lock.name.clone())))
    }

    /// Gets the lock mode.
    fn mode_getter(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Lock.prototype.mode called on non-Lock object")
        })?;

        let lock = obj.downcast_ref::<Lock>().ok_or_else(|| {
            JsNativeError::typ().with_message("Lock.prototype.mode called on non-Lock object")
        })?;

        Ok(JsValue::from(JsString::from(lock.mode.as_str())))
    }
}
