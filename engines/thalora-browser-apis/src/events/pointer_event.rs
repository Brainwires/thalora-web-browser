//! PointerEvent implementation for Boa
//!
//! Implements the PointerEvent interface as defined in:
//! https://www.w3.org/TR/pointerevents3/

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

/// JavaScript `PointerEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PointerEvent;

impl IntrinsicObject for PointerEvent {
    fn init(realm: &Realm) {
        let pointer_id_getter = BuiltInBuilder::callable(realm, get_pointer_id)
            .name(js_string!("get pointerId"))
            .build();

        let width_getter = BuiltInBuilder::callable(realm, get_width)
            .name(js_string!("get width"))
            .build();

        let height_getter = BuiltInBuilder::callable(realm, get_height)
            .name(js_string!("get height"))
            .build();

        let pressure_getter = BuiltInBuilder::callable(realm, get_pressure)
            .name(js_string!("get pressure"))
            .build();

        let tangential_pressure_getter = BuiltInBuilder::callable(realm, get_tangential_pressure)
            .name(js_string!("get tangentialPressure"))
            .build();

        let tilt_x_getter = BuiltInBuilder::callable(realm, get_tilt_x)
            .name(js_string!("get tiltX"))
            .build();

        let tilt_y_getter = BuiltInBuilder::callable(realm, get_tilt_y)
            .name(js_string!("get tiltY"))
            .build();

        let twist_getter = BuiltInBuilder::callable(realm, get_twist)
            .name(js_string!("get twist"))
            .build();

        let pointer_type_getter = BuiltInBuilder::callable(realm, get_pointer_type)
            .name(js_string!("get pointerType"))
            .build();

        let is_primary_getter = BuiltInBuilder::callable(realm, get_is_primary)
            .name(js_string!("get isPrimary"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("pointerId"),
                Some(pointer_id_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("width"),
                Some(width_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("height"),
                Some(height_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pressure"),
                Some(pressure_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tangentialPressure"),
                Some(tangential_pressure_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tiltX"),
                Some(tilt_x_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tiltY"),
                Some(tilt_y_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("twist"),
                Some(twist_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pointerType"),
                Some(pointer_type_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isPrimary"),
                Some(is_primary_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PointerEvent {
    const NAME: JsString = StaticJsStrings::POINTER_EVENT;
}

impl BuiltInConstructor for PointerEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::pointer_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("PointerEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::pointer_event,
            context,
        )?;

        let (
            pointer_id,
            width,
            height,
            pressure,
            tangential_pressure,
            tilt_x,
            tilt_y,
            twist,
            pointer_type,
            is_primary,
        ) = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                let pointer_id = init_obj
                    .get(js_string!("pointerId"), context)
                    .ok()
                    .and_then(|v| v.to_i32(context).ok())
                    .unwrap_or(0);
                let width = init_obj
                    .get(js_string!("width"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(1.0);
                let height = init_obj
                    .get(js_string!("height"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(1.0);
                let pressure = init_obj
                    .get(js_string!("pressure"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(0.0) as f32;
                let tangential_pressure = init_obj
                    .get(js_string!("tangentialPressure"), context)
                    .ok()
                    .and_then(|v| v.to_number(context).ok())
                    .unwrap_or(0.0) as f32;
                let tilt_x = init_obj
                    .get(js_string!("tiltX"), context)
                    .ok()
                    .and_then(|v| v.to_i32(context).ok())
                    .unwrap_or(0);
                let tilt_y = init_obj
                    .get(js_string!("tiltY"), context)
                    .ok()
                    .and_then(|v| v.to_i32(context).ok())
                    .unwrap_or(0);
                let twist = init_obj
                    .get(js_string!("twist"), context)
                    .ok()
                    .and_then(|v| v.to_i32(context).ok())
                    .unwrap_or(0);
                let pointer_type = init_obj
                    .get(js_string!("pointerType"), context)
                    .ok()
                    .map(|v| v.to_string(context).ok())
                    .flatten()
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                let is_primary = init_obj
                    .get(js_string!("isPrimary"), context)
                    .ok()
                    .map(|v| v.to_boolean())
                    .unwrap_or(false);
                (
                    pointer_id,
                    width,
                    height,
                    pressure,
                    tangential_pressure,
                    tilt_x,
                    tilt_y,
                    twist,
                    pointer_type,
                    is_primary,
                )
            } else {
                (0, 1.0, 1.0, 0.0, 0.0, 0, 0, 0, String::new(), false)
            }
        } else {
            (0, 1.0, 1.0, 0.0, 0.0, 0, 0, 0, String::new(), false)
        };

        let pointer_event_data = PointerEventData::new(
            event_type.to_std_string_escaped(),
            pointer_id,
            width,
            height,
            pressure,
            tangential_pressure,
            tilt_x,
            tilt_y,
            twist,
            pointer_type,
            is_primary,
        );

        let pointer_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            pointer_event_data,
        );

        let pointer_event_generic = pointer_event_obj.upcast();

        // Set Event interface properties
        pointer_event_generic.set(js_string!("type"), event_type, false, context)?;
        pointer_event_generic.set(js_string!("bubbles"), true, false, context)?;
        pointer_event_generic.set(js_string!("cancelable"), true, false, context)?;
        pointer_event_generic.set(js_string!("composed"), true, false, context)?;
        pointer_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        pointer_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        pointer_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        pointer_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        pointer_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        pointer_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                    pointer_event_generic.set(
                        js_string!("bubbles"),
                        bubbles_val.to_boolean(),
                        false,
                        context,
                    )?;
                }
                if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                    pointer_event_generic.set(
                        js_string!("cancelable"),
                        cancelable_val.to_boolean(),
                        false,
                        context,
                    )?;
                }
            }
        }

        Ok(pointer_event_generic.into())
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct PointerEventData {
    #[unsafe_ignore_trace]
    event_type: String,
    #[unsafe_ignore_trace]
    pointer_id: i32,
    #[unsafe_ignore_trace]
    width: f64,
    #[unsafe_ignore_trace]
    height: f64,
    #[unsafe_ignore_trace]
    pressure: f32,
    #[unsafe_ignore_trace]
    tangential_pressure: f32,
    #[unsafe_ignore_trace]
    tilt_x: i32,
    #[unsafe_ignore_trace]
    tilt_y: i32,
    #[unsafe_ignore_trace]
    twist: i32,
    #[unsafe_ignore_trace]
    pointer_type: String,
    #[unsafe_ignore_trace]
    is_primary: bool,
}

impl PointerEventData {
    #[allow(clippy::too_many_arguments)]
    fn new(
        event_type: String,
        pointer_id: i32,
        width: f64,
        height: f64,
        pressure: f32,
        tangential_pressure: f32,
        tilt_x: i32,
        tilt_y: i32,
        twist: i32,
        pointer_type: String,
        is_primary: bool,
    ) -> Self {
        Self {
            event_type,
            pointer_id,
            width,
            height,
            pressure,
            tangential_pressure,
            tilt_x,
            tilt_y,
            twist,
            pointer_type,
            is_primary,
        }
    }
}

fn get_pointer_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.pointerId called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.pointerId called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.pointer_id))
}

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.width called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.width called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.width))
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.height called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.height called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.height))
}

fn get_pressure(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.pressure called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.pressure called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.pressure))
}

fn get_tangential_pressure(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.tangentialPressure called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message(
            "PointerEvent.prototype.tangentialPressure called on non-PointerEvent object",
        )
    })?;

    Ok(JsValue::from(data.tangential_pressure))
}

fn get_tilt_x(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.tiltX called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.tiltX called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.tilt_x))
}

fn get_tilt_y(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.tiltY called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.tiltY called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.tilt_y))
}

fn get_twist(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.twist called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.twist called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.twist))
}

fn get_pointer_type(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.pointerType called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.pointerType called on non-PointerEvent object")
    })?;

    Ok(js_string!(data.pointer_type.clone()).into())
}

fn get_is_primary(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PointerEvent.prototype.isPrimary called on non-object")
    })?;

    let data = this_obj.downcast_ref::<PointerEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PointerEvent.prototype.isPrimary called on non-PointerEvent object")
    })?;

    Ok(JsValue::from(data.is_primary))
}
