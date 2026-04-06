//! PopStateEvent implementation for Boa
//!
//! Implements the PopStateEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/browsing-the-web.html#popstateevent

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

/// JavaScript `PopStateEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PopStateEvent;

impl IntrinsicObject for PopStateEvent {
    fn init(realm: &Realm) {
        let state_getter = BuiltInBuilder::callable(realm, get_state)
            .name(js_string!("get state"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("state"),
                Some(state_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PopStateEvent {
    const NAME: JsString = StaticJsStrings::POP_STATE_EVENT;
}

impl BuiltInConstructor for PopStateEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::pop_state_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("PopStateEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::pop_state_event,
            context,
        )?;

        let state = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                init_obj
                    .get(js_string!("state"), context)
                    .unwrap_or(JsValue::null())
            } else {
                JsValue::null()
            }
        } else {
            JsValue::null()
        };

        let pop_state_event_data =
            PopStateEventData::new(event_type.to_std_string_escaped(), state);
        let pop_state_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            pop_state_event_data,
        );

        let pop_state_event_generic = pop_state_event_obj.upcast();

        // Set Event interface properties
        pop_state_event_generic.set(js_string!("type"), event_type, false, context)?;
        pop_state_event_generic.set(js_string!("bubbles"), false, false, context)?;
        pop_state_event_generic.set(js_string!("cancelable"), false, false, context)?;
        pop_state_event_generic.set(js_string!("composed"), false, false, context)?;
        pop_state_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        pop_state_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        pop_state_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        pop_state_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        pop_state_event_generic.set(
            js_string!("currentTarget"),
            JsValue::null(),
            false,
            context,
        )?;
        pop_state_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                pop_state_event_generic.set(
                    js_string!("bubbles"),
                    bubbles_val.to_boolean(),
                    false,
                    context,
                )?;
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                pop_state_event_generic.set(
                    js_string!("cancelable"),
                    cancelable_val.to_boolean(),
                    false,
                    context,
                )?;
            }
        }

        Ok(pop_state_event_generic.into())
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct PopStateEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    state: JsValue,
}

impl PopStateEventData {
    fn new(event_type: String, state: JsValue) -> Self {
        Self { event_type, state }
    }
}

fn get_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PopStateEvent.prototype.state called on non-object")
    })?;

    let pop_state_event = this_obj
        .downcast_ref::<PopStateEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("PopStateEvent.prototype.state called on non-PopStateEvent object")
        })?;

    Ok(pop_state_event.state.clone())
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
    fn test_pop_state_event_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof PopStateEvent === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_pop_state_event_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const event = new PopStateEvent('popstate', { state: { page: 1 } });
            event.type === 'popstate' && event.state.page === 1;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
