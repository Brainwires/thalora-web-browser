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

/// JavaScript `CustomEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct CustomEvent;

impl IntrinsicObject for CustomEvent {
    fn init(realm: &Realm) {
        let detail_getter = BuiltInBuilder::callable(realm, get_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
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
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

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

        // Get detail from eventInitDict
        let detail = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                init_obj.get(js_string!("detail"), context).unwrap_or(JsValue::null())
            } else {
                JsValue::null()
            }
        } else {
            JsValue::null()
        };

        let custom_event_data = CustomEventData::new(event_type.to_std_string_escaped(), detail);
        let custom_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            custom_event_data,
        );

        let custom_event_generic = custom_event_obj.upcast();

        // Set Event interface properties
        custom_event_generic.set(js_string!("type"), event_type, false, context)?;
        custom_event_generic.set(js_string!("bubbles"), false, false, context)?;
        custom_event_generic.set(js_string!("cancelable"), false, false, context)?;
        custom_event_generic.set(js_string!("composed"), false, false, context)?;
        custom_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        custom_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        custom_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        custom_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        custom_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        custom_event_generic.set(js_string!("timeStamp"), context.clock().now().millis_since_epoch(), false, context)?;

        // Parse eventInitDict for Event properties
        if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                    custom_event_generic.set(js_string!("bubbles"), bubbles_val.to_boolean(), false, context)?;
                }
                if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                    custom_event_generic.set(js_string!("cancelable"), cancelable_val.to_boolean(), false, context)?;
                }
                if let Ok(composed_val) = init_obj.get(js_string!("composed"), context) {
                    custom_event_generic.set(js_string!("composed"), composed_val.to_boolean(), false, context)?;
                }
            }
        }

        Ok(custom_event_generic.into())
    }
}

/// Internal data for CustomEvent instances
#[derive(Debug, Trace, Finalize, JsData)]
struct CustomEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    detail: JsValue,
}

impl CustomEventData {
    fn new(event_type: String, detail: JsValue) -> Self {
        Self { event_type, detail }
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

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let bubbles = args.get_or_undefined(1).to_boolean();
    let cancelable = args.get_or_undefined(2).to_boolean();
    let detail = args.get_or_undefined(3).clone();

    this_obj.set(js_string!("type"), event_type, false, context)?;
    this_obj.set(js_string!("bubbles"), bubbles, false, context)?;
    this_obj.set(js_string!("cancelable"), cancelable, false, context)?;

    // Update detail in the data if possible
    if let Some(mut custom_event) = this_obj.downcast_mut::<CustomEventData>() {
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
