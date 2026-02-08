//! AbortController Web API implementation for Boa
//!
//! Native implementation of AbortController standard
//! https://dom.spec.whatwg.org/#interface-abortcontroller

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};

/// JavaScript `AbortController` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct AbortController;

impl IntrinsicObject for AbortController {
    fn init(realm: &Realm) {
        let signal_func = BuiltInBuilder::callable(realm, get_signal)
            .name(js_string!("get signal"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("signal"),
                Some(signal_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(abort, js_string!("abort"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AbortController {
    const NAME: JsString = StaticJsStrings::ABORT_CONTROLLER;
}

impl BuiltInConstructor for AbortController {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::abort_controller;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::abort_controller,
            context,
        )?;

        let abort_controller_data = AbortControllerData::new();

        let abort_controller = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            abort_controller_data,
        );

        // Create the AbortSignal once and store it on the controller
        let signal = crate::events::abort_signal::create_abort_signal(context)?;
        let signal_value: JsValue = signal.into();
        let controller_obj = abort_controller.upcast();
        controller_obj.set(js_string!("__signal__"), signal_value, false, context)?;

        Ok(controller_obj.into())
    }
}

/// Internal data for AbortController objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AbortControllerData {
    // Marker type — state is tracked on the AbortSignal object itself
    _private: (),
}

impl AbortControllerData {
    fn new() -> Self {
        Self { _private: () }
    }
}

/// `AbortController.prototype.signal` getter — returns the cached signal
fn get_signal(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortController.prototype.signal called on non-object")
    })?;

    // Verify this is an AbortController
    let _ = this_obj.downcast_ref::<AbortControllerData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortController.prototype.signal called on non-AbortController object")
    })?;

    // Return the cached signal created in the constructor
    this_obj.get(js_string!("__signal__"), context)
}

/// `AbortController.prototype.abort(reason)`
fn abort(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortController.prototype.abort called on non-object")
    })?;

    // Verify this is an AbortController
    let _ = this_obj.downcast_ref::<AbortControllerData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("AbortController.prototype.abort called on non-AbortController object")
    })?;

    let reason = if args.is_empty() || args.get_or_undefined(0).is_undefined() {
        // Default reason per spec: DOMException with name "AbortError"
        // Create an Error object as a stand-in
        match context
            .intrinsics()
            .constructors()
            .error()
            .constructor()
            .construct(
                &[js_string!("The operation was aborted.").into()],
                None,
                context,
            ) {
            Ok(err_obj) => err_obj.into(),
            Err(_) => JsValue::undefined(),
        }
    } else {
        args.get_or_undefined(0).clone()
    };

    // Get the cached signal
    let signal_val = this_obj.get(js_string!("__signal__"), context)?;
    if let Some(signal_obj) = signal_val.as_object() {
        // Update AbortSignalData
        if let Some(mut signal_data) = signal_obj.downcast_mut::<crate::events::abort_signal::AbortSignalData>() {
            signal_data.abort(reason.clone());
        }

        // Dispatch 'abort' event on the signal
        if let Ok(event) = context
            .intrinsics()
            .constructors()
            .event()
            .constructor()
            .construct(&[js_string!("abort").into()], None, context)
        {
            if let Ok(dispatch_fn) = signal_obj.get(js_string!("dispatchEvent"), context) {
                if let Some(callable) = dispatch_fn.as_callable() {
                    let _ = callable.call(
                        &signal_obj.clone().into(),
                        &[event.into()],
                        context,
                    );
                }
            }

            // Also call onabort property handler
            if let Ok(onabort) = signal_obj.get(js_string!("onabort"), context) {
                if let Some(callable) = onabort.as_callable() {
                    let _ = callable.call(&signal_obj.clone().into(), &[], context);
                }
            }
        }
    }

    Ok(JsValue::undefined())
}
