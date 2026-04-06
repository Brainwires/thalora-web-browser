//! ProgressEvent implementation for Boa
//!
//! Implements the ProgressEvent interface as defined in:
//! https://xhr.spec.whatwg.org/#interface-progressevent

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
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

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

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::progress_event,
            context,
        )?;

        let (length_computable, loaded, total) = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                let length_computable = init_obj
                    .get(js_string!("lengthComputable"), context)
                    .ok()
                    .map(|v| v.to_boolean())
                    .unwrap_or(false);
                let loaded = init_obj
                    .get(js_string!("loaded"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(0.0) as u64;
                let total = init_obj
                    .get(js_string!("total"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(0.0) as u64;
                (length_computable, loaded, total)
            } else {
                (false, 0, 0)
            }
        } else {
            (false, 0, 0)
        };

        let progress_event_data = ProgressEventData::new(
            event_type.to_std_string_escaped(),
            length_computable,
            loaded,
            total,
        );
        let progress_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            progress_event_data,
        );

        let progress_event_generic = progress_event_obj.upcast();

        // Set Event interface properties
        progress_event_generic.set(js_string!("type"), event_type, false, context)?;
        progress_event_generic.set(js_string!("bubbles"), false, false, context)?;
        progress_event_generic.set(js_string!("cancelable"), false, false, context)?;
        progress_event_generic.set(js_string!("composed"), false, false, context)?;
        progress_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        progress_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        progress_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        progress_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        progress_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        progress_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                progress_event_generic.set(
                    js_string!("bubbles"),
                    bubbles_val.to_boolean(),
                    false,
                    context,
                )?;
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                progress_event_generic.set(
                    js_string!("cancelable"),
                    cancelable_val.to_boolean(),
                    false,
                    context,
                )?;
            }
        }

        Ok(progress_event_generic.into())
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct ProgressEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    #[unsafe_ignore_trace]
    length_computable: bool,
    #[unsafe_ignore_trace]
    loaded: u64,
    #[unsafe_ignore_trace]
    total: u64,
}

impl ProgressEventData {
    fn new(event_type: String, length_computable: bool, loaded: u64, total: u64) -> Self {
        Self {
            event_type,
            length_computable,
            loaded,
            total,
        }
    }
}

fn get_length_computable(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ProgressEvent.prototype.lengthComputable called on non-object")
    })?;

    let progress_event = this_obj
        .downcast_ref::<ProgressEventData>()
        .ok_or_else(|| {
            JsNativeError::typ().with_message(
                "ProgressEvent.prototype.lengthComputable called on non-ProgressEvent object",
            )
        })?;

    Ok(JsValue::from(progress_event.length_computable))
}

fn get_loaded(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.loaded called on non-object")
    })?;

    let progress_event = this_obj
        .downcast_ref::<ProgressEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("ProgressEvent.prototype.loaded called on non-ProgressEvent object")
        })?;

    Ok(JsValue::from(progress_event.loaded as f64))
}

fn get_total(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ProgressEvent.prototype.total called on non-object")
    })?;

    let progress_event = this_obj
        .downcast_ref::<ProgressEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("ProgressEvent.prototype.total called on non-ProgressEvent object")
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
        let result = context
            .eval(Source::from_bytes("typeof ProgressEvent === 'function'"))
            .unwrap();
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
