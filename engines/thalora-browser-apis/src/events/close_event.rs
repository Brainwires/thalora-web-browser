//! CloseEvent implementation for Boa
//!
//! Implements the CloseEvent interface as defined in:
//! https://websockets.spec.whatwg.org/#closeevent

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

/// JavaScript `CloseEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct CloseEvent;

impl IntrinsicObject for CloseEvent {
    fn init(realm: &Realm) {
        let was_clean_getter = BuiltInBuilder::callable(realm, get_was_clean)
            .name(js_string!("get wasClean"))
            .build();

        let code_getter = BuiltInBuilder::callable(realm, get_code)
            .name(js_string!("get code"))
            .build();

        let reason_getter = BuiltInBuilder::callable(realm, get_reason)
            .name(js_string!("get reason"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("wasClean"),
                Some(was_clean_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("code"),
                Some(code_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("reason"),
                Some(reason_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CloseEvent {
    const NAME: JsString = StaticJsStrings::CLOSE_EVENT;
}

impl BuiltInConstructor for CloseEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::close_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CloseEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::close_event, context)?;

        let (was_clean, code, reason) = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                let was_clean = init_obj
                    .get(js_string!("wasClean"), context)
                    .ok()
                    .map(|v| v.to_boolean())
                    .unwrap_or(false);
                let code = init_obj
                    .get(js_string!("code"), context)
                    .ok()
                    .and_then(|v| v.to_u32(context).ok())
                    .unwrap_or(0) as u16;
                let reason = init_obj
                    .get(js_string!("reason"), context)
                    .ok()
                    .and_then(|v| v.to_string(context).ok())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                (was_clean, code, reason)
            } else {
                (false, 0, String::new())
            }
        } else {
            (false, 0, String::new())
        };

        let close_event_data =
            CloseEventData::new(event_type.to_std_string_escaped(), was_clean, code, reason);
        let close_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            close_event_data,
        );

        let close_event_generic = close_event_obj.upcast();

        // Set Event interface properties
        close_event_generic.set(js_string!("type"), event_type, false, context)?;
        close_event_generic.set(js_string!("bubbles"), false, false, context)?;
        close_event_generic.set(js_string!("cancelable"), false, false, context)?;
        close_event_generic.set(js_string!("composed"), false, false, context)?;
        close_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        close_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        close_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        close_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        close_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        close_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                close_event_generic.set(
                    js_string!("bubbles"),
                    bubbles_val.to_boolean(),
                    false,
                    context,
                )?;
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                close_event_generic.set(
                    js_string!("cancelable"),
                    cancelable_val.to_boolean(),
                    false,
                    context,
                )?;
            }
        }

        Ok(close_event_generic.into())
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct CloseEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    #[unsafe_ignore_trace]
    was_clean: bool,
    #[unsafe_ignore_trace]
    code: u16,
    #[unsafe_ignore_trace]
    reason: String,
}

impl CloseEventData {
    fn new(event_type: String, was_clean: bool, code: u16, reason: String) -> Self {
        Self {
            event_type,
            was_clean,
            code,
            reason,
        }
    }
}

fn get_was_clean(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CloseEvent.prototype.wasClean called on non-object")
    })?;

    let close_event = this_obj.downcast_ref::<CloseEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CloseEvent.prototype.wasClean called on non-CloseEvent object")
    })?;

    Ok(JsValue::from(close_event.was_clean))
}

fn get_code(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CloseEvent.prototype.code called on non-object")
    })?;

    let close_event = this_obj.downcast_ref::<CloseEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CloseEvent.prototype.code called on non-CloseEvent object")
    })?;

    Ok(JsValue::from(close_event.code))
}

fn get_reason(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CloseEvent.prototype.reason called on non-object")
    })?;

    let close_event = this_obj.downcast_ref::<CloseEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CloseEvent.prototype.reason called on non-CloseEvent object")
    })?;

    Ok(js_string!(close_event.reason.clone()).into())
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
    fn test_close_event_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof CloseEvent === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_close_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new CloseEvent('close', { wasClean: true, code: 1000, reason: 'Normal closure' });
            event.type === 'close' && event.wasClean === true && event.code === 1000 && event.reason === 'Normal closure';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
