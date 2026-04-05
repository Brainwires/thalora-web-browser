//! TouchEvent implementation for Boa
//!
//! Implements the TouchEvent interface as defined in:
//! https://www.w3.org/TR/touch-events/
//!
//! This is primarily for API surface compatibility so sites that check
//! `'ontouchstart' in window` or `typeof TouchEvent` can function correctly.

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

/// JavaScript `TouchEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct TouchEvent;

impl IntrinsicObject for TouchEvent {
    fn init(realm: &Realm) {
        let touches_getter = BuiltInBuilder::callable(realm, get_touches)
            .name(js_string!("get touches"))
            .build();

        let target_touches_getter = BuiltInBuilder::callable(realm, get_target_touches)
            .name(js_string!("get targetTouches"))
            .build();

        let changed_touches_getter = BuiltInBuilder::callable(realm, get_changed_touches)
            .name(js_string!("get changedTouches"))
            .build();

        let alt_key_getter = BuiltInBuilder::callable(realm, get_alt_key)
            .name(js_string!("get altKey"))
            .build();

        let meta_key_getter = BuiltInBuilder::callable(realm, get_meta_key)
            .name(js_string!("get metaKey"))
            .build();

        let ctrl_key_getter = BuiltInBuilder::callable(realm, get_ctrl_key)
            .name(js_string!("get ctrlKey"))
            .build();

        let shift_key_getter = BuiltInBuilder::callable(realm, get_shift_key)
            .name(js_string!("get shiftKey"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("touches"),
                Some(touches_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("targetTouches"),
                Some(target_touches_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("changedTouches"),
                Some(changed_touches_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("altKey"),
                Some(alt_key_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("metaKey"),
                Some(meta_key_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ctrlKey"),
                Some(ctrl_key_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shiftKey"),
                Some(shift_key_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TouchEvent {
    const NAME: JsString = StaticJsStrings::TOUCH_EVENT;
}

impl BuiltInConstructor for TouchEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::touch_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("TouchEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::touch_event, context)?;

        let (touches, target_touches, changed_touches, alt_key, meta_key, ctrl_key, shift_key) =
            if !event_init_dict.is_undefined() {
                if let Some(init_obj) = event_init_dict.as_object() {
                    let touches = init_obj
                        .get(js_string!("touches"), context)
                        .ok()
                        .filter(|v| !v.is_undefined())
                        .unwrap_or(JsValue::undefined());
                    let target_touches = init_obj
                        .get(js_string!("targetTouches"), context)
                        .ok()
                        .filter(|v| !v.is_undefined())
                        .unwrap_or(JsValue::undefined());
                    let changed_touches = init_obj
                        .get(js_string!("changedTouches"), context)
                        .ok()
                        .filter(|v| !v.is_undefined())
                        .unwrap_or(JsValue::undefined());
                    let alt_key = init_obj
                        .get(js_string!("altKey"), context)
                        .ok()
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    let meta_key = init_obj
                        .get(js_string!("metaKey"), context)
                        .ok()
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    let ctrl_key = init_obj
                        .get(js_string!("ctrlKey"), context)
                        .ok()
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    let shift_key = init_obj
                        .get(js_string!("shiftKey"), context)
                        .ok()
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    (
                        touches,
                        target_touches,
                        changed_touches,
                        alt_key,
                        meta_key,
                        ctrl_key,
                        shift_key,
                    )
                } else {
                    (
                        JsValue::undefined(),
                        JsValue::undefined(),
                        JsValue::undefined(),
                        false,
                        false,
                        false,
                        false,
                    )
                }
            } else {
                (
                    JsValue::undefined(),
                    JsValue::undefined(),
                    JsValue::undefined(),
                    false,
                    false,
                    false,
                    false,
                )
            };

        let touch_event_data = TouchEventData::new(
            event_type.to_std_string_escaped(),
            touches,
            target_touches,
            changed_touches,
            alt_key,
            meta_key,
            ctrl_key,
            shift_key,
        );

        let touch_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            touch_event_data,
        );

        let touch_event_generic = touch_event_obj.upcast();

        // Set Event interface properties
        touch_event_generic.set(js_string!("type"), event_type, false, context)?;
        touch_event_generic.set(js_string!("bubbles"), true, false, context)?;
        touch_event_generic.set(js_string!("cancelable"), true, false, context)?;
        touch_event_generic.set(js_string!("composed"), true, false, context)?;
        touch_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        touch_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        touch_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        touch_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        touch_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        touch_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                    touch_event_generic.set(
                        js_string!("bubbles"),
                        bubbles_val.to_boolean(),
                        false,
                        context,
                    )?;
                }
                if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                    touch_event_generic.set(
                        js_string!("cancelable"),
                        cancelable_val.to_boolean(),
                        false,
                        context,
                    )?;
                }
            }
        }

        Ok(touch_event_generic.into())
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct TouchEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    #[unsafe_ignore_trace]
    touches: JsValue,
    #[unsafe_ignore_trace]
    target_touches: JsValue,
    #[unsafe_ignore_trace]
    changed_touches: JsValue,
    #[unsafe_ignore_trace]
    alt_key: bool,
    #[unsafe_ignore_trace]
    meta_key: bool,
    #[unsafe_ignore_trace]
    ctrl_key: bool,
    #[unsafe_ignore_trace]
    shift_key: bool,
}

impl TouchEventData {
    #[allow(clippy::too_many_arguments)]
    fn new(
        event_type: String,
        touches: JsValue,
        target_touches: JsValue,
        changed_touches: JsValue,
        alt_key: bool,
        meta_key: bool,
        ctrl_key: bool,
        shift_key: bool,
    ) -> Self {
        Self {
            event_type,
            touches,
            target_touches,
            changed_touches,
            alt_key,
            meta_key,
            ctrl_key,
            shift_key,
        }
    }
}

fn get_touches(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.touches called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.touches called on non-TouchEvent object")
    })?;

    Ok(data.touches.clone())
}

fn get_target_touches(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.targetTouches called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.targetTouches called on non-TouchEvent object")
    })?;

    Ok(data.target_touches.clone())
}

fn get_changed_touches(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.changedTouches called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.changedTouches called on non-TouchEvent object")
    })?;

    Ok(data.changed_touches.clone())
}

fn get_alt_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.altKey called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.altKey called on non-TouchEvent object")
    })?;

    Ok(JsValue::from(data.alt_key))
}

fn get_meta_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.metaKey called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.metaKey called on non-TouchEvent object")
    })?;

    Ok(JsValue::from(data.meta_key))
}

fn get_ctrl_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.ctrlKey called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.ctrlKey called on non-TouchEvent object")
    })?;

    Ok(JsValue::from(data.ctrl_key))
}

fn get_shift_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TouchEvent.prototype.shiftKey called on non-object")
    })?;

    let data = this_obj.downcast_ref::<TouchEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TouchEvent.prototype.shiftKey called on non-TouchEvent object")
    })?;

    Ok(JsValue::from(data.shift_key))
}
