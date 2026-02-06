//! CustomEvent implementation for Boa
//!
//! Implements the CustomEvent interface as defined in:
//! https://dom.spec.whatwg.org/#interface-customevent

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

/// JavaScript `CustomEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct CustomEvent;

impl IntrinsicObject for CustomEvent {
    fn init(realm: &Realm) {
        let detail_getter = BuiltInBuilder::callable(realm, get_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("detail"),
                Some(detail_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(init_custom_event, js_string!("initCustomEvent"), 4)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CustomEvent {
    const NAME: JsString = StaticJsStrings::CUSTOM_EVENT;
}

impl BuiltInConstructor for CustomEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::custom_event;

    /// `new CustomEvent(type, eventInitDict)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CustomEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::custom_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut composed = false;
        let mut detail = JsValue::null();

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("composed"), context) {
                composed = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                detail = v;
            }
        }

        let mut event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        event_data.set_composed(composed);

        let custom_event_data = CustomEventData::new(event_data, detail);
        let custom_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            custom_event_data,
        );

        Ok(custom_event_obj.into())
    }
}

/// Internal data for CustomEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CustomEventData {
    /// Base event data
    pub event: EventData,
    /// The detail value
    detail: JsValue,
}

impl CustomEventData {
    pub fn new(event: EventData, detail: JsValue) -> Self {
        Self { event, detail }
    }
}

/// `CustomEvent.prototype.detail` getter
fn get_detail(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CustomEvent.prototype.detail called on non-object")
    })?;

    let custom_event = this_obj.downcast_ref::<CustomEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("CustomEvent.prototype.detail called on non-CustomEvent object")
    })?;

    Ok(custom_event.detail.clone())
}

/// `CustomEvent.prototype.initCustomEvent(type, bubbles, cancelable, detail)`
fn init_custom_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CustomEvent.prototype.initCustomEvent called on non-object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let bubbles = args.get_or_undefined(1).to_boolean();
    let cancelable = args.get_or_undefined(2).to_boolean();
    let detail = args.get_or_undefined(3).clone();

    // Update the embedded event data
    if let Some(mut custom_event) = this_obj.downcast_mut::<CustomEventData>() {
        custom_event.event.init_event(event_type, bubbles, cancelable);
        custom_event.detail = detail;
    }

    Ok(JsValue::undefined())
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
    fn test_custom_event_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof CustomEvent === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_custom_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new CustomEvent('myevent', { detail: { key: 'value' } });
            event.type === 'myevent';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_custom_event_detail() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new CustomEvent('myevent', { detail: 42 });
            event.detail === 42;
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
