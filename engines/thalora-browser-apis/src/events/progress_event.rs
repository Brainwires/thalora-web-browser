//! ProgressEvent implementation for Boa
//!
//! Implements the ProgressEvent interface as defined in:
//! https://xhr.spec.whatwg.org/#interface-progressevent

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

/// JavaScript `ProgressEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ProgressEvent;

impl IntrinsicObject for ProgressEvent {
    fn init(realm: &Realm) {
        let length_computable_getter = BuiltInBuilder::callable(realm, get_length_computable)
            .name(js_string!("get lengthComputable"))
            .build();

        let loaded_getter = BuiltInBuilder::callable(realm, get_loaded)
            .name(js_string!("get loaded"))
            .build();

        let total_getter = BuiltInBuilder::callable(realm, get_total)
            .name(js_string!("get total"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("lengthComputable"),
                Some(length_computable_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("loaded"),
                Some(loaded_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("total"),
                Some(total_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ProgressEvent {
    const NAME: JsString = StaticJsStrings::PROGRESS_EVENT;
}

impl BuiltInConstructor for ProgressEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 6;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::progress_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ProgressEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::progress_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut length_computable = false;
        let mut loaded = 0u64;
        let mut total = 0u64;

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("lengthComputable"), context) {
                length_computable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("loaded"), context) {
                loaded = v.to_number(context)? as u64;
            }
            if let Ok(v) = init_obj.get(js_string!("total"), context) {
                total = v.to_number(context)? as u64;
            }
        }

        let event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        let progress_event_data = ProgressEventData::new(event_data, length_computable, loaded, total);

        let progress_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            progress_event_data,
        );

        Ok(progress_event_obj.into())
    }
}

/// Internal data for ProgressEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ProgressEventData {
    /// Base event data
    pub event: EventData,
    #[unsafe_ignore_trace]
    length_computable: bool,
    #[unsafe_ignore_trace]
    loaded: u64,
    #[unsafe_ignore_trace]
    total: u64,
}

impl ProgressEventData {
    pub fn new(event: EventData, length_computable: bool, loaded: u64, total: u64) -> Self {
        Self { event, length_computable, loaded, total }
    }
}

fn get_length_computable(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.lengthComputable called on non-object")
    })?;

    let progress_event = this_obj.downcast_ref::<ProgressEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.lengthComputable called on non-ProgressEvent object")
    })?;

    Ok(JsValue::from(progress_event.length_computable))
}

fn get_loaded(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.loaded called on non-object")
    })?;

    let progress_event = this_obj.downcast_ref::<ProgressEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.loaded called on non-ProgressEvent object")
    })?;

    Ok(JsValue::from(progress_event.loaded as f64))
}

fn get_total(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.total called on non-object")
    })?;

    let progress_event = this_obj.downcast_ref::<ProgressEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.total called on non-ProgressEvent object")
    })?;

    Ok(JsValue::from(progress_event.total as f64))
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
    fn test_progress_event_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof ProgressEvent === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_progress_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new ProgressEvent('progress', { lengthComputable: true, loaded: 50, total: 100 });
            event.type === 'progress' && event.lengthComputable === true && event.loaded === 50 && event.total === 100;
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
