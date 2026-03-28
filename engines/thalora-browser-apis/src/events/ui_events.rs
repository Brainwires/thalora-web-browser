//! UI Event Web API implementations for Boa
//!
//! Native implementations of UI Events (KeyboardEvent, MouseEvent, FocusEvent, InputEvent)
//! https://w3c.github.io/uievents/

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

use super::event::EventData;

// ============================================================================
// UIEvent - Base class for UI events
// ============================================================================

/// The `UIEvent` data object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct UIEventData {
    /// Base event data
    pub event: EventData,
    /// The view (window) where the event occurred
    pub view: Option<JsObject>,
    /// Detail value (e.g., click count for mouse events)
    pub detail: i32,
}

impl UIEventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            event: EventData::new(event_type, bubbles, cancelable),
            view: None,
            detail: 0,
        }
    }
}

/// The `UIEvent` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct UIEvent;

impl IntrinsicObject for UIEvent {
    fn init(realm: &Realm) {
        let view_func = BuiltInBuilder::callable(realm, get_ui_view)
            .name(js_string!("get view"))
            .build();

        let detail_func = BuiltInBuilder::callable(realm, get_ui_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("view"),
                Some(view_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for UIEvent {
    const NAME: JsString = StaticJsStrings::UI_EVENT;
}

impl BuiltInConstructor for UIEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4; // 2 accessors x 2 slots each
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::ui_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling UIEvent constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        let mut bubbles = false;
        let mut cancelable = false;
        let mut view = None;
        let mut detail = 0i32;

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("view"), context) {
                view = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                detail = v.to_i32(context)?;
            }
        }

        let mut data = UIEventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        data.view = view;
        data.detail = detail;

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::ui_event, context)?;
        let event =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

fn get_ui_view(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("UIEvent method called on non-object"))?;

    if let Some(data) = this_obj.downcast_ref::<UIEventData>() {
        return Ok(data.view.clone().map_or(JsValue::null(), |v| v.into()));
    }
    if let Some(data) = this_obj.downcast_ref::<KeyboardEventData>() {
        return Ok(data
            .ui_event
            .view
            .clone()
            .map_or(JsValue::null(), |v| v.into()));
    }
    if let Some(data) = this_obj.downcast_ref::<MouseEventData>() {
        return Ok(data
            .ui_event
            .view
            .clone()
            .map_or(JsValue::null(), |v| v.into()));
    }
    if let Some(data) = this_obj.downcast_ref::<FocusEventData>() {
        return Ok(data
            .ui_event
            .view
            .clone()
            .map_or(JsValue::null(), |v| v.into()));
    }
    if let Some(data) = this_obj.downcast_ref::<InputEventData>() {
        return Ok(data
            .ui_event
            .view
            .clone()
            .map_or(JsValue::null(), |v| v.into()));
    }

    Err(JsNativeError::typ()
        .with_message("UIEvent method called on non-UIEvent object")
        .into())
}

fn get_ui_detail(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("UIEvent method called on non-object"))?;

    if let Some(data) = this_obj.downcast_ref::<UIEventData>() {
        return Ok(JsValue::from(data.detail));
    }
    if let Some(data) = this_obj.downcast_ref::<KeyboardEventData>() {
        return Ok(JsValue::from(data.ui_event.detail));
    }
    if let Some(data) = this_obj.downcast_ref::<MouseEventData>() {
        return Ok(JsValue::from(data.ui_event.detail));
    }
    if let Some(data) = this_obj.downcast_ref::<FocusEventData>() {
        return Ok(JsValue::from(data.ui_event.detail));
    }
    if let Some(data) = this_obj.downcast_ref::<InputEventData>() {
        return Ok(JsValue::from(data.ui_event.detail));
    }

    Err(JsNativeError::typ()
        .with_message("UIEvent method called on non-UIEvent object")
        .into())
}

// ============================================================================
// KeyboardEvent
// ============================================================================

/// Key location constants
pub mod key_location {
    pub const DOM_KEY_LOCATION_STANDARD: u32 = 0;
    pub const DOM_KEY_LOCATION_LEFT: u32 = 1;
    pub const DOM_KEY_LOCATION_RIGHT: u32 = 2;
    pub const DOM_KEY_LOCATION_NUMPAD: u32 = 3;
}

/// The `KeyboardEvent` data object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct KeyboardEventData {
    /// Base UI event data
    pub ui_event: UIEventData,
    /// The key value of the key pressed
    pub key: String,
    /// The physical key code
    pub code: String,
    /// The location of the key on the keyboard
    pub location: u32,
    /// Whether the Ctrl key was pressed
    pub ctrl_key: bool,
    /// Whether the Shift key was pressed
    pub shift_key: bool,
    /// Whether the Alt key was pressed
    pub alt_key: bool,
    /// Whether the Meta key was pressed
    pub meta_key: bool,
    /// Whether the key is being held down (repeat)
    pub repeat: bool,
    /// Whether the event is composing
    pub is_composing: bool,
    /// Legacy: key code (deprecated but still used)
    pub key_code: u32,
    /// Legacy: char code (deprecated but still used)
    pub char_code: u32,
    /// Legacy: which key (deprecated but still used)
    pub which: u32,
}

impl KeyboardEventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            ui_event: UIEventData::new(event_type, bubbles, cancelable),
            key: String::new(),
            code: String::new(),
            location: key_location::DOM_KEY_LOCATION_STANDARD,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
            key_code: 0,
            char_code: 0,
            which: 0,
        }
    }

    /// Check if any modifier key is pressed
    pub fn get_modifier_state(&self, key: &str) -> bool {
        match key {
            "Control" => self.ctrl_key,
            "Shift" => self.shift_key,
            "Alt" => self.alt_key,
            "Meta" => self.meta_key,
            _ => false,
        }
    }
}

/// The `KeyboardEvent` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct KeyboardEvent;

impl IntrinsicObject for KeyboardEvent {
    fn init(realm: &Realm) {
        // Create accessor functions
        let key_func = BuiltInBuilder::callable(realm, get_key)
            .name(js_string!("get key"))
            .build();
        let code_func = BuiltInBuilder::callable(realm, get_code)
            .name(js_string!("get code"))
            .build();
        let location_func = BuiltInBuilder::callable(realm, get_location)
            .name(js_string!("get location"))
            .build();
        let ctrl_key_func = BuiltInBuilder::callable(realm, get_ctrl_key)
            .name(js_string!("get ctrlKey"))
            .build();
        let shift_key_func = BuiltInBuilder::callable(realm, get_shift_key)
            .name(js_string!("get shiftKey"))
            .build();
        let alt_key_func = BuiltInBuilder::callable(realm, get_alt_key)
            .name(js_string!("get altKey"))
            .build();
        let meta_key_func = BuiltInBuilder::callable(realm, get_meta_key)
            .name(js_string!("get metaKey"))
            .build();
        let repeat_func = BuiltInBuilder::callable(realm, get_repeat)
            .name(js_string!("get repeat"))
            .build();
        let is_composing_func = BuiltInBuilder::callable(realm, get_keyboard_is_composing)
            .name(js_string!("get isComposing"))
            .build();
        let key_code_func = BuiltInBuilder::callable(realm, get_key_code)
            .name(js_string!("get keyCode"))
            .build();
        let char_code_func = BuiltInBuilder::callable(realm, get_char_code)
            .name(js_string!("get charCode"))
            .build();
        let which_func = BuiltInBuilder::callable(realm, get_keyboard_which)
            .name(js_string!("get which"))
            .build();
        let view_func = BuiltInBuilder::callable(realm, get_ui_view)
            .name(js_string!("get view"))
            .build();
        let detail_func = BuiltInBuilder::callable(realm, get_ui_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("key"),
                Some(key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("code"),
                Some(code_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("location"),
                Some(location_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ctrlKey"),
                Some(ctrl_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shiftKey"),
                Some(shift_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("altKey"),
                Some(alt_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("metaKey"),
                Some(meta_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("repeat"),
                Some(repeat_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isComposing"),
                Some(is_composing_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("keyCode"),
                Some(key_code_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("charCode"),
                Some(char_code_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("which"),
                Some(which_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("view"),
                Some(view_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(get_modifier_state, js_string!("getModifierState"), 1)
            .static_property(
                js_string!("DOM_KEY_LOCATION_STANDARD"),
                key_location::DOM_KEY_LOCATION_STANDARD,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("DOM_KEY_LOCATION_LEFT"),
                key_location::DOM_KEY_LOCATION_LEFT,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("DOM_KEY_LOCATION_RIGHT"),
                key_location::DOM_KEY_LOCATION_RIGHT,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("DOM_KEY_LOCATION_NUMPAD"),
                key_location::DOM_KEY_LOCATION_NUMPAD,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for KeyboardEvent {
    const NAME: JsString = StaticJsStrings::KEYBOARD_EVENT;
}

impl BuiltInConstructor for KeyboardEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 29; // 14 accessors * 2 + 1 method
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 4; // 4 static properties

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::keyboard_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling KeyboardEvent constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        let mut data = KeyboardEventData::new(event_type.to_std_string_escaped(), false, false);

        if let Some(init_obj) = event_init.as_object() {
            // Base event properties
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                data.ui_event.event.set_bubbles(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                data.ui_event.event.set_cancelable(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("view"), context) {
                data.ui_event.view = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                data.ui_event.detail = v.to_i32(context)?;
            }
            // Keyboard-specific properties
            if let Ok(v) = init_obj.get(js_string!("key"), context) {
                data.key = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("code"), context) {
                data.code = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("location"), context) {
                data.location = v.to_u32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("ctrlKey"), context) {
                data.ctrl_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("shiftKey"), context) {
                data.shift_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("altKey"), context) {
                data.alt_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("metaKey"), context) {
                data.meta_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("repeat"), context) {
                data.repeat = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("isComposing"), context) {
                data.is_composing = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("keyCode"), context) {
                data.key_code = v.to_u32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("charCode"), context) {
                data.char_code = v.to_u32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("which"), context) {
                data.which = v.to_u32(context)?;
            }
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::keyboard_event,
            context,
        )?;
        let event =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

// KeyboardEvent accessor functions
fn get_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(js_string!(data.key.clone())))
}

fn get_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(js_string!(data.code.clone())))
}

fn get_location(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.location))
}

fn get_ctrl_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.ctrl_key))
}

fn get_shift_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.shift_key))
}

fn get_alt_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.alt_key))
}

fn get_meta_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.meta_key))
}

fn get_repeat(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.repeat))
}

fn get_keyboard_is_composing(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.is_composing))
}

fn get_key_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.key_code))
}

fn get_char_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.char_code))
}

fn get_keyboard_which(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;
    let data = this_obj
        .downcast_ref::<KeyboardEventData>()
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("KeyboardEvent method called on non-KeyboardEvent object")
        })?;
    Ok(JsValue::from(data.which))
}

fn get_modifier_state(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("KeyboardEvent method called on non-object")
    })?;

    let key = args.get_or_undefined(0).to_string(context)?;

    if let Some(data) = this_obj.downcast_ref::<KeyboardEventData>() {
        return Ok(JsValue::from(
            data.get_modifier_state(&key.to_std_string_escaped()),
        ));
    }
    if let Some(data) = this_obj.downcast_ref::<MouseEventData>() {
        return Ok(JsValue::from(
            data.get_modifier_state(&key.to_std_string_escaped()),
        ));
    }

    Err(JsNativeError::typ()
        .with_message("getModifierState called on non-keyboard/mouse event object")
        .into())
}

// ============================================================================
// MouseEvent
// ============================================================================

/// Mouse button constants
pub mod mouse_button {
    pub const PRIMARY: i16 = 0; // Usually the left button
    pub const AUXILIARY: i16 = 1; // Usually the wheel/middle button
    pub const SECONDARY: i16 = 2; // Usually the right button
    pub const FOURTH: i16 = 3; // Browser back
    pub const FIFTH: i16 = 4; // Browser forward
}

/// The `MouseEvent` data object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct MouseEventData {
    /// Base UI event data
    pub ui_event: UIEventData,
    /// X coordinate relative to the viewport
    pub client_x: f64,
    /// Y coordinate relative to the viewport
    pub client_y: f64,
    /// X coordinate relative to the screen
    pub screen_x: f64,
    /// Y coordinate relative to the screen
    pub screen_y: f64,
    /// X coordinate relative to the page
    pub page_x: f64,
    /// Y coordinate relative to the page
    pub page_y: f64,
    /// X offset from the target element
    pub offset_x: f64,
    /// Y offset from the target element
    pub offset_y: f64,
    /// Movement in X since last event
    pub movement_x: f64,
    /// Movement in Y since last event
    pub movement_y: f64,
    /// Which mouse button was pressed
    pub button: i16,
    /// Which mouse buttons are currently pressed
    pub buttons: u16,
    /// Whether the Ctrl key was pressed
    pub ctrl_key: bool,
    /// Whether the Shift key was pressed
    pub shift_key: bool,
    /// Whether the Alt key was pressed
    pub alt_key: bool,
    /// Whether the Meta key was pressed
    pub meta_key: bool,
    /// The secondary target for the event
    pub related_target: Option<JsObject>,
}

impl MouseEventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            ui_event: UIEventData::new(event_type, bubbles, cancelable),
            client_x: 0.0,
            client_y: 0.0,
            screen_x: 0.0,
            screen_y: 0.0,
            page_x: 0.0,
            page_y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            movement_x: 0.0,
            movement_y: 0.0,
            button: 0,
            buttons: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            related_target: None,
        }
    }

    /// Check if any modifier key is pressed
    pub fn get_modifier_state(&self, key: &str) -> bool {
        match key {
            "Control" => self.ctrl_key,
            "Shift" => self.shift_key,
            "Alt" => self.alt_key,
            "Meta" => self.meta_key,
            _ => false,
        }
    }
}

/// The `MouseEvent` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct MouseEvent;

impl IntrinsicObject for MouseEvent {
    fn init(realm: &Realm) {
        let client_x_func = BuiltInBuilder::callable(realm, get_client_x)
            .name(js_string!("get clientX"))
            .build();
        let client_y_func = BuiltInBuilder::callable(realm, get_client_y)
            .name(js_string!("get clientY"))
            .build();
        let screen_x_func = BuiltInBuilder::callable(realm, get_screen_x)
            .name(js_string!("get screenX"))
            .build();
        let screen_y_func = BuiltInBuilder::callable(realm, get_screen_y)
            .name(js_string!("get screenY"))
            .build();
        let page_x_func = BuiltInBuilder::callable(realm, get_page_x)
            .name(js_string!("get pageX"))
            .build();
        let page_y_func = BuiltInBuilder::callable(realm, get_page_y)
            .name(js_string!("get pageY"))
            .build();
        let offset_x_func = BuiltInBuilder::callable(realm, get_offset_x)
            .name(js_string!("get offsetX"))
            .build();
        let offset_y_func = BuiltInBuilder::callable(realm, get_offset_y)
            .name(js_string!("get offsetY"))
            .build();
        let movement_x_func = BuiltInBuilder::callable(realm, get_movement_x)
            .name(js_string!("get movementX"))
            .build();
        let movement_y_func = BuiltInBuilder::callable(realm, get_movement_y)
            .name(js_string!("get movementY"))
            .build();
        let button_func = BuiltInBuilder::callable(realm, get_button)
            .name(js_string!("get button"))
            .build();
        let buttons_func = BuiltInBuilder::callable(realm, get_buttons)
            .name(js_string!("get buttons"))
            .build();
        let ctrl_key_func = BuiltInBuilder::callable(realm, get_mouse_ctrl_key)
            .name(js_string!("get ctrlKey"))
            .build();
        let shift_key_func = BuiltInBuilder::callable(realm, get_mouse_shift_key)
            .name(js_string!("get shiftKey"))
            .build();
        let alt_key_func = BuiltInBuilder::callable(realm, get_mouse_alt_key)
            .name(js_string!("get altKey"))
            .build();
        let meta_key_func = BuiltInBuilder::callable(realm, get_mouse_meta_key)
            .name(js_string!("get metaKey"))
            .build();
        let related_target_func = BuiltInBuilder::callable(realm, get_mouse_related_target)
            .name(js_string!("get relatedTarget"))
            .build();
        let view_func = BuiltInBuilder::callable(realm, get_ui_view)
            .name(js_string!("get view"))
            .build();
        let detail_func = BuiltInBuilder::callable(realm, get_ui_detail)
            .name(js_string!("get detail"))
            .build();

        // Aliases for x/y (same as clientX/clientY)
        let x_func = BuiltInBuilder::callable(realm, get_client_x)
            .name(js_string!("get x"))
            .build();
        let y_func = BuiltInBuilder::callable(realm, get_client_y)
            .name(js_string!("get y"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("clientX"),
                Some(client_x_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientY"),
                Some(client_y_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("screenX"),
                Some(screen_x_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("screenY"),
                Some(screen_y_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pageX"),
                Some(page_x_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pageY"),
                Some(page_y_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetX"),
                Some(offset_x_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetY"),
                Some(offset_y_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("movementX"),
                Some(movement_x_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("movementY"),
                Some(movement_y_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("button"),
                Some(button_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("buttons"),
                Some(buttons_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ctrlKey"),
                Some(ctrl_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shiftKey"),
                Some(shift_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("altKey"),
                Some(alt_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("metaKey"),
                Some(meta_key_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("relatedTarget"),
                Some(related_target_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(js_string!("x"), Some(x_func), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("y"), Some(y_func), None, Attribute::CONFIGURABLE)
            .accessor(
                js_string!("view"),
                Some(view_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(get_modifier_state, js_string!("getModifierState"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MouseEvent {
    const NAME: JsString = StaticJsStrings::MOUSE_EVENT;
}

impl BuiltInConstructor for MouseEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 43; // 21 accessors * 2 + 1 method
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::mouse_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling MouseEvent constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        let mut data = MouseEventData::new(event_type.to_std_string_escaped(), true, true);

        if let Some(init_obj) = event_init.as_object() {
            // Base event properties
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                data.ui_event.event.set_bubbles(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                data.ui_event.event.set_cancelable(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("view"), context) {
                data.ui_event.view = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                data.ui_event.detail = v.to_i32(context)?;
            }
            // Mouse-specific properties
            if let Ok(v) = init_obj.get(js_string!("clientX"), context) {
                data.client_x = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("clientY"), context) {
                data.client_y = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("screenX"), context) {
                data.screen_x = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("screenY"), context) {
                data.screen_y = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("pageX"), context) {
                data.page_x = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("pageY"), context) {
                data.page_y = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("offsetX"), context) {
                data.offset_x = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("offsetY"), context) {
                data.offset_y = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("movementX"), context) {
                data.movement_x = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("movementY"), context) {
                data.movement_y = v.to_number(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("button"), context) {
                data.button = v.to_i32(context)? as i16;
            }
            if let Ok(v) = init_obj.get(js_string!("buttons"), context) {
                data.buttons = v.to_u32(context)? as u16;
            }
            if let Ok(v) = init_obj.get(js_string!("ctrlKey"), context) {
                data.ctrl_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("shiftKey"), context) {
                data.shift_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("altKey"), context) {
                data.alt_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("metaKey"), context) {
                data.meta_key = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("relatedTarget"), context) {
                data.related_target = v.as_object();
            }
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::mouse_event, context)?;
        let event =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

// MouseEvent accessor functions
fn get_client_x(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.client_x))
}

fn get_client_y(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.client_y))
}

fn get_screen_x(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.screen_x))
}

fn get_screen_y(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.screen_y))
}

fn get_page_x(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.page_x))
}

fn get_page_y(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.page_y))
}

fn get_offset_x(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.offset_x))
}

fn get_offset_y(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.offset_y))
}

fn get_movement_x(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.movement_x))
}

fn get_movement_y(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.movement_y))
}

fn get_button(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.button))
}

fn get_buttons(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.buttons))
}

fn get_mouse_ctrl_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.ctrl_key))
}

fn get_mouse_shift_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.shift_key))
}

fn get_mouse_alt_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.alt_key))
}

fn get_mouse_meta_key(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(JsValue::from(data.meta_key))
}

fn get_mouse_related_target(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<MouseEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MouseEvent method called on non-MouseEvent object")
    })?;
    Ok(data
        .related_target
        .clone()
        .map_or(JsValue::null(), |t| t.into()))
}

// ============================================================================
// FocusEvent
// ============================================================================

/// The `FocusEvent` data object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct FocusEventData {
    /// Base UI event data
    pub ui_event: UIEventData,
    /// The secondary target (element losing/gaining focus)
    pub related_target: Option<JsObject>,
}

impl FocusEventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            ui_event: UIEventData::new(event_type, bubbles, cancelable),
            related_target: None,
        }
    }
}

/// The `FocusEvent` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct FocusEvent;

impl IntrinsicObject for FocusEvent {
    fn init(realm: &Realm) {
        let related_target_func = BuiltInBuilder::callable(realm, get_focus_related_target)
            .name(js_string!("get relatedTarget"))
            .build();
        let view_func = BuiltInBuilder::callable(realm, get_ui_view)
            .name(js_string!("get view"))
            .build();
        let detail_func = BuiltInBuilder::callable(realm, get_ui_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("relatedTarget"),
                Some(related_target_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("view"),
                Some(view_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for FocusEvent {
    const NAME: JsString = StaticJsStrings::FOCUS_EVENT;
}

impl BuiltInConstructor for FocusEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 6; // 3 accessors * 2
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::focus_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling FocusEvent constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        // Focus events don't bubble by default (except focusin/focusout)
        let bubbles = matches!(
            event_type.to_std_string_escaped().as_str(),
            "focusin" | "focusout"
        );
        let mut data = FocusEventData::new(event_type.to_std_string_escaped(), bubbles, false);

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                data.ui_event.event.set_bubbles(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                data.ui_event.event.set_cancelable(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("view"), context) {
                data.ui_event.view = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                data.ui_event.detail = v.to_i32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("relatedTarget"), context) {
                data.related_target = v.as_object();
            }
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::focus_event, context)?;
        let event =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

fn get_focus_related_target(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FocusEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<FocusEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("FocusEvent method called on non-FocusEvent object")
    })?;
    Ok(data
        .related_target
        .clone()
        .map_or(JsValue::null(), |t| t.into()))
}

// ============================================================================
// InputEvent
// ============================================================================

/// The `InputEvent` data object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct InputEventData {
    /// Base UI event data
    pub ui_event: UIEventData,
    /// The inserted characters (if any)
    pub data: Option<String>,
    /// The type of input (e.g., "insertText", "deleteContentBackward")
    pub input_type: String,
    /// Whether the event is part of a composition session
    pub is_composing: bool,
    /// Data transfer object (for paste operations)
    pub data_transfer: Option<JsObject>,
}

impl InputEventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            ui_event: UIEventData::new(event_type, bubbles, cancelable),
            data: None,
            input_type: String::new(),
            is_composing: false,
            data_transfer: None,
        }
    }
}

/// The `InputEvent` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct InputEvent;

impl IntrinsicObject for InputEvent {
    fn init(realm: &Realm) {
        let data_func = BuiltInBuilder::callable(realm, get_input_data)
            .name(js_string!("get data"))
            .build();
        let input_type_func = BuiltInBuilder::callable(realm, get_input_type)
            .name(js_string!("get inputType"))
            .build();
        let is_composing_func = BuiltInBuilder::callable(realm, get_input_is_composing)
            .name(js_string!("get isComposing"))
            .build();
        let data_transfer_func = BuiltInBuilder::callable(realm, get_input_data_transfer)
            .name(js_string!("get dataTransfer"))
            .build();
        let view_func = BuiltInBuilder::callable(realm, get_ui_view)
            .name(js_string!("get view"))
            .build();
        let detail_func = BuiltInBuilder::callable(realm, get_ui_detail)
            .name(js_string!("get detail"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("data"),
                Some(data_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("inputType"),
                Some(input_type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isComposing"),
                Some(is_composing_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("dataTransfer"),
                Some(data_transfer_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("view"),
                Some(view_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(get_target_ranges, js_string!("getTargetRanges"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for InputEvent {
    const NAME: JsString = StaticJsStrings::INPUT_EVENT;
}

impl BuiltInConstructor for InputEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 13; // 6 accessors * 2 + 1 method
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::input_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling InputEvent constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        let mut data = InputEventData::new(event_type.to_std_string_escaped(), true, false);

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                data.ui_event.event.set_bubbles(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                data.ui_event.event.set_cancelable(v.to_boolean());
            }
            if let Ok(v) = init_obj.get(js_string!("view"), context) {
                data.ui_event.view = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("detail"), context) {
                data.ui_event.detail = v.to_i32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("data"), context) {
                if !v.is_null_or_undefined() {
                    data.data = Some(v.to_string(context)?.to_std_string_escaped());
                }
            }
            if let Ok(v) = init_obj.get(js_string!("inputType"), context) {
                data.input_type = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("isComposing"), context) {
                data.is_composing = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("dataTransfer"), context) {
                data.data_transfer = v.as_object();
            }
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::input_event, context)?;
        let event =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

fn get_input_data(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<InputEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-InputEvent object")
    })?;
    Ok(data
        .data
        .as_ref()
        .map_or(JsValue::null(), |d| JsValue::from(js_string!(d.clone()))))
}

fn get_input_type(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<InputEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-InputEvent object")
    })?;
    Ok(JsValue::from(js_string!(data.input_type.clone())))
}

fn get_input_is_composing(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<InputEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-InputEvent object")
    })?;
    Ok(JsValue::from(data.is_composing))
}

fn get_input_data_transfer(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-object")
    })?;
    let data = this_obj.downcast_ref::<InputEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-InputEvent object")
    })?;
    Ok(data
        .data_transfer
        .clone()
        .map_or(JsValue::null(), |dt| dt.into()))
}

fn get_target_ranges(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-object")
    })?;
    let _data = this_obj.downcast_ref::<InputEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("InputEvent method called on non-InputEvent object")
    })?;
    // Returns an empty array - StaticRange not yet implemented
    let array = boa_engine::builtins::array::Array::array_create(0, None, context)?;
    Ok(array.into())
}
