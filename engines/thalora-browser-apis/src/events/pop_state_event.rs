//! PopStateEvent implementation for Boa
//!
//! Implements the PopStateEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/browsing-the-web.html#popstateevent

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};

use super::event::EventData;

/// JavaScript `PopStateEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PopStateEvent;

impl IntrinsicObject for PopStateEvent {
    fn init(realm: &Realm) {
        let state_getter = BuiltInBuilder::callable(realm, get_state)
            .name(js_string!("get state"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
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
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

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

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::pop_state_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut state = JsValue::null();

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("state"), context) {
                state = v;
            }
        }

        let event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        let pop_state_event_data = PopStateEventData::new(event_data, state);

        let pop_state_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            pop_state_event_data,
        );

        Ok(pop_state_event_obj.into())
    }
}

/// Internal data for PopStateEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct PopStateEventData {
    /// Base event data
    pub event: EventData,
    state: JsValue,
}

impl PopStateEventData {
    pub fn new(event: EventData, state: JsValue) -> Self {
        Self { event, state }
    }
}

fn get_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PopStateEvent.prototype.state called on non-object")
    })?;

    let pop_state_event = this_obj.downcast_ref::<PopStateEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("PopStateEvent.prototype.state called on non-PopStateEvent object")
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
        let result = context.eval(Source::from_bytes("typeof PopStateEvent === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_pop_state_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new PopStateEvent('popstate', { state: { page: 1 } });
            event.type === 'popstate' && event.state.page === 1;
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
