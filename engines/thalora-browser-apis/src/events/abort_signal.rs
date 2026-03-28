//! AbortSignal implementation for Boa
//!
//! Implements the AbortSignal interface as defined in:
//! https://dom.spec.whatwg.org/#interface-abortsignal

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `AbortSignal` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct AbortSignal;

impl IntrinsicObject for AbortSignal {
    fn init(realm: &Realm) {
        let aborted_getter = BuiltInBuilder::callable(realm, get_aborted)
            .name(js_string!("get aborted"))
            .build();

        let reason_getter = BuiltInBuilder::callable(realm, get_reason)
            .name(js_string!("get reason"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("aborted"),
                Some(aborted_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("reason"),
                Some(reason_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(throw_if_aborted, js_string!("throwIfAborted"), 0)
            .static_method(abort_static, js_string!("abort"), 0)
            .static_method(timeout_static, js_string!("timeout"), 1)
            .static_method(any_static, js_string!("any"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AbortSignal {
    const NAME: JsString = StaticJsStrings::ABORT_SIGNAL;
}

impl BuiltInConstructor for AbortSignal {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::abort_signal;

    /// AbortSignal cannot be constructed directly
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // AbortSignal is typically not constructed directly
        // It's obtained from AbortController.signal or via static methods
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

/// Internal data for AbortSignal instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AbortSignalData {
    #[unsafe_ignore_trace]
    aborted: bool,
    reason: JsValue,
}

impl AbortSignalData {
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: JsValue::undefined(),
        }
    }

    pub fn new_aborted(reason: JsValue) -> Self {
        Self {
            aborted: true,
            reason,
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    pub fn abort(&mut self, reason: JsValue) {
        if !self.aborted {
            self.aborted = true;
            self.reason = reason;
        }
    }
}

/// Create an AbortSignal object (for internal use)
pub fn create_abort_signal(context: &mut Context) -> JsResult<JsObject> {
    let proto = context
        .intrinsics()
        .constructors()
        .abort_signal()
        .prototype();
    let signal_data = AbortSignalData::new();
    let signal_obj =
        JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), proto, signal_data);
    Ok(signal_obj.upcast())
}

/// Create an already-aborted AbortSignal
pub fn create_aborted_signal(reason: JsValue, context: &mut Context) -> JsResult<JsObject> {
    let proto = context
        .intrinsics()
        .constructors()
        .abort_signal()
        .prototype();
    let signal_data = AbortSignalData::new_aborted(reason);
    let signal_obj =
        JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), proto, signal_data);
    Ok(signal_obj.upcast())
}

fn get_aborted(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortSignal.prototype.aborted called on non-object")
    })?;

    let signal = this_obj.downcast_ref::<AbortSignalData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortSignal.prototype.aborted called on non-AbortSignal object")
    })?;

    Ok(JsValue::from(signal.is_aborted()))
}

fn get_reason(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortSignal.prototype.reason called on non-object")
    })?;

    let signal = this_obj.downcast_ref::<AbortSignalData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortSignal.prototype.reason called on non-AbortSignal object")
    })?;

    Ok(signal.reason.clone())
}

fn throw_if_aborted(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortSignal.prototype.throwIfAborted called on non-object")
    })?;

    let signal = this_obj.downcast_ref::<AbortSignalData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortSignal.prototype.throwIfAborted called on non-AbortSignal object")
    })?;

    if signal.is_aborted() {
        // The reason should be the thrown error
        return Err(JsNativeError::error()
            .with_message("The operation was aborted")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `AbortSignal.abort(reason)`
fn abort_static(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let reason = args.get_or_undefined(0).clone();
    let signal = create_aborted_signal(reason, context)?;
    Ok(signal.into())
}

/// `AbortSignal.timeout(milliseconds)`
fn timeout_static(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _milliseconds = args.get_or_undefined(0).to_u32(context)?;
    // Create a signal that will be aborted after timeout
    // For now, just create a non-aborted signal (full implementation would require async)
    let signal = create_abort_signal(context)?;
    Ok(signal.into())
}

/// `AbortSignal.any(signals)`
fn any_static(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let signals_arg = args.get_or_undefined(0);

    // Check if any signal is already aborted
    if let Some(signals_obj) = signals_arg.as_object() {
        // Try to iterate over the signals
        if let Ok(length) = signals_obj.get(js_string!("length"), context) {
            let len = length.to_u32(context)?;
            for i in 0..len {
                if let Ok(signal_val) = signals_obj.get(i, context) {
                    if let Some(signal_obj) = signal_val.as_object() {
                        if let Some(signal_data) = signal_obj.downcast_ref::<AbortSignalData>() {
                            if signal_data.is_aborted() {
                                // Return an already-aborted signal with the same reason
                                return Ok(create_aborted_signal(
                                    signal_data.reason.clone(),
                                    context,
                                )?
                                .into());
                            }
                        }
                    }
                }
            }
        }
    }

    // No signals are aborted, create a new linked signal
    let signal = create_abort_signal(context)?;
    Ok(signal.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_abort_signal_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof AbortSignal === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_abort_signal_abort_static() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const signal = AbortSignal.abort();
            signal.aborted === true;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_abort_signal_timeout_static() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const signal = AbortSignal.timeout(1000);
            typeof signal === 'object';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
