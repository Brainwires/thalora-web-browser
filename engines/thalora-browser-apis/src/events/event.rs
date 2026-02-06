//! Event Web API implementation for Boa
//!
//! Native implementation of DOM Events with proper propagation
//! https://dom.spec.whatwg.org/#interface-event


use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};

/// Event phases
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum EventPhase {
    None = 0,
    CapturingPhase = 1,
    AtTarget = 2,
    BubblingPhase = 3,
}

/// The `Event` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct EventData {
    /// Event type (e.g., "click", "load")
    event_type: String,
    /// Whether the event bubbles
    bubbles: bool,
    /// Whether the event is cancelable
    cancelable: bool,
    /// Whether preventDefault() has been called
    default_prevented: bool,
    /// Current event phase
    phase: EventPhase,
    /// Current target (changes during propagation)
    current_target: Option<JsObject>,
    /// Event target (original target, doesn't change)
    target: Option<JsObject>,
    /// Whether stopPropagation() has been called
    stop_propagation: bool,
    /// Whether stopImmediatePropagation() has been called
    stop_immediate_propagation: bool,
    /// Timestamp when event was created
    timestamp: f64,
    /// Whether the event is trusted (created by user agent)
    is_trusted: bool,
    /// Whether the event is composed (crosses shadow boundaries)
    composed: bool,
}

impl EventData {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Self {
            event_type,
            bubbles,
            cancelable,
            default_prevented: false,
            phase: EventPhase::None,
            current_target: None,
            target: None,
            stop_propagation: false,
            stop_immediate_propagation: false,
            timestamp: 0.0, // Would use Date.now() equivalent in real implementation
            is_trusted: false,
            composed: false,
        }
    }

    /// Create a new trusted event (for browser-initiated events like mouse clicks)
    /// Trusted events have is_trusted set to true, indicating they were created
    /// by the user agent rather than by JavaScript code.
    pub fn new_trusted(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        let mut event = Self::new(event_type, bubbles, cancelable);
        event.is_trusted = true;
        // Trusted events get a proper timestamp
        event.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        event
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.stop_immediate_propagation = true;
        self.stop_propagation = true;
    }

    pub fn init_event(&mut self, event_type: String, bubbles: bool, cancelable: bool) {
        if self.phase != EventPhase::None {
            return; // Cannot initialize during dispatch
        }

        self.event_type = event_type;
        self.bubbles = bubbles;
        self.cancelable = cancelable;
        self.default_prevented = false;
        self.stop_propagation = false;
        self.stop_immediate_propagation = false;
    }

    // Getters for read-only properties
    pub fn get_type(&self) -> &str { &self.event_type }
    pub fn get_bubbles(&self) -> bool { self.bubbles }
    pub fn get_cancelable(&self) -> bool { self.cancelable }
    pub fn get_default_prevented(&self) -> bool { self.default_prevented }
    pub fn get_phase(&self) -> EventPhase { self.phase.clone() }
    pub fn get_current_target(&self) -> Option<JsObject> { self.current_target.clone() }
    pub fn get_target(&self) -> Option<JsObject> { self.target.clone() }
    pub fn get_timestamp(&self) -> f64 { self.timestamp }
    pub fn get_is_trusted(&self) -> bool { self.is_trusted }
    pub fn get_composed(&self) -> bool { self.composed }

    // Internal setters for event propagation
    pub fn set_phase(&mut self, phase: EventPhase) { self.phase = phase; }
    pub fn set_current_target(&mut self, target: Option<JsObject>) { self.current_target = target; }
    pub fn set_target(&mut self, target: Option<JsObject>) { self.target = target; }
    pub fn set_is_trusted(&mut self, trusted: bool) { self.is_trusted = trusted; }

    // Setters for subclass event construction
    pub fn set_bubbles(&mut self, bubbles: bool) { self.bubbles = bubbles; }
    pub fn set_cancelable(&mut self, cancelable: bool) { self.cancelable = cancelable; }
    pub fn set_composed(&mut self, composed: bool) { self.composed = composed; }

    pub fn should_stop_propagation(&self) -> bool { self.stop_propagation }
    pub fn should_stop_immediate_propagation(&self) -> bool { self.stop_immediate_propagation }
}

/// The `Event` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Event;

impl IntrinsicObject for Event {
    fn init(realm: &Realm) {
        let type_func = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let bubbles_func = BuiltInBuilder::callable(realm, get_bubbles)
            .name(js_string!("get bubbles"))
            .build();

        let cancelable_func = BuiltInBuilder::callable(realm, get_cancelable)
            .name(js_string!("get cancelable"))
            .build();

        let default_prevented_func = BuiltInBuilder::callable(realm, get_default_prevented)
            .name(js_string!("get defaultPrevented"))
            .build();

        let event_phase_func = BuiltInBuilder::callable(realm, get_event_phase)
            .name(js_string!("get eventPhase"))
            .build();

        let current_target_func = BuiltInBuilder::callable(realm, get_current_target)
            .name(js_string!("get currentTarget"))
            .build();

        let target_func = BuiltInBuilder::callable(realm, get_target)
            .name(js_string!("get target"))
            .build();

        let timestamp_func = BuiltInBuilder::callable(realm, get_timestamp)
            .name(js_string!("get timeStamp"))
            .build();

        let is_trusted_func = BuiltInBuilder::callable(realm, get_is_trusted)
            .name(js_string!("get isTrusted"))
            .build();

        let composed_func = BuiltInBuilder::callable(realm, get_composed)
            .name(js_string!("get composed"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("type"),
                Some(type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("bubbles"),
                Some(bubbles_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cancelable"),
                Some(cancelable_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("defaultPrevented"),
                Some(default_prevented_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("eventPhase"),
                Some(event_phase_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("currentTarget"),
                Some(current_target_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("target"),
                Some(target_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("timeStamp"),
                Some(timestamp_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isTrusted"),
                Some(is_trusted_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("composed"),
                Some(composed_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(prevent_default, js_string!("preventDefault"), 0)
            .method(stop_propagation, js_string!("stopPropagation"), 0)
            .method(stop_immediate_propagation, js_string!("stopImmediatePropagation"), 0)
            .method(init_event, js_string!("initEvent"), 3)
            .static_property(
                js_string!("NONE"),
                0,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("CAPTURING_PHASE"),
                1,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("AT_TARGET"),
                2,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("BUBBLING_PHASE"),
                3,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Event {
    const NAME: JsString = StaticJsStrings::EVENT;
}

impl BuiltInConstructor for Event {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If new_target is undefined then this function was called without new
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling Event constructor without `new` is forbidden")
                .into());
        }

        let event_type = args.get_or_undefined(0).to_string(context)?;
        let event_init = args.get_or_undefined(1);

        let mut bubbles = false;
        let mut cancelable = false;
        let mut composed = false;

        // Parse EventInit dictionary if provided
        if let Some(init_obj) = event_init.as_object() {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = bubbles_val.to_boolean();
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = cancelable_val.to_boolean();
            }
            if let Ok(composed_val) = init_obj.get(js_string!("composed"), context) {
                composed = composed_val.to_boolean();
            }
        }

        let mut data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        data.composed = composed;

        let prototype = get_prototype_from_constructor(new_target, StandardConstructors::event, context)?;
        let event = JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(event.into())
    }
}

// Accessor functions

/// Get the event type.
/// Handles EventData, UIEventData, MouseEventData, and all Event subclasses.
fn get_type(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(js_string!(event_data.get_type())))
}

/// Helper function to get EventData from any event type (EventData, UIEventData, MouseEventData, etc.)
fn get_event_data_from_obj(this_obj: &JsObject) -> Option<EventData> {
    use super::ui_events::{UIEventData, MouseEventData, KeyboardEventData, FocusEventData, InputEventData};
    use super::custom_event::CustomEventData;
    use super::message_event::MessageEventData;
    use super::error_event::ErrorEventData;
    use super::progress_event::ProgressEventData;
    use super::hash_change_event::HashChangeEventData;
    use super::pop_state_event::PopStateEventData;
    use super::close_event::CloseEventData;
    use super::pageswap_event::PageSwapEventData;
    use crate::storage::storage_event::StorageEvent;

    // Try EventData directly
    if let Some(data) = this_obj.downcast_ref::<EventData>() {
        return Some(data.clone());
    }
    // Try MouseEventData (MouseEventData.ui_event.event)
    if let Some(data) = this_obj.downcast_ref::<MouseEventData>() {
        return Some(data.ui_event.event.clone());
    }
    // Try KeyboardEventData (KeyboardEventData.ui_event.event)
    if let Some(data) = this_obj.downcast_ref::<KeyboardEventData>() {
        return Some(data.ui_event.event.clone());
    }
    // Try FocusEventData (FocusEventData.ui_event.event)
    if let Some(data) = this_obj.downcast_ref::<FocusEventData>() {
        return Some(data.ui_event.event.clone());
    }
    // Try InputEventData (InputEventData.ui_event.event)
    if let Some(data) = this_obj.downcast_ref::<InputEventData>() {
        return Some(data.ui_event.event.clone());
    }
    // Try UIEventData (UIEventData.event)
    if let Some(data) = this_obj.downcast_ref::<UIEventData>() {
        return Some(data.event.clone());
    }
    // Try CustomEventData
    if let Some(data) = this_obj.downcast_ref::<CustomEventData>() {
        return Some(data.event.clone());
    }
    // Try MessageEventData
    if let Some(data) = this_obj.downcast_ref::<MessageEventData>() {
        return Some(data.event.clone());
    }
    // Try ErrorEventData
    if let Some(data) = this_obj.downcast_ref::<ErrorEventData>() {
        return Some(data.event.clone());
    }
    // Try ProgressEventData
    if let Some(data) = this_obj.downcast_ref::<ProgressEventData>() {
        return Some(data.event.clone());
    }
    // Try HashChangeEventData
    if let Some(data) = this_obj.downcast_ref::<HashChangeEventData>() {
        return Some(data.event.clone());
    }
    // Try PopStateEventData
    if let Some(data) = this_obj.downcast_ref::<PopStateEventData>() {
        return Some(data.event.clone());
    }
    // Try CloseEventData
    if let Some(data) = this_obj.downcast_ref::<CloseEventData>() {
        return Some(data.event.clone());
    }
    // Try PageSwapEventData
    if let Some(data) = this_obj.downcast_ref::<PageSwapEventData>() {
        return Some(data.event.clone());
    }
    // Try StorageEvent
    if let Some(data) = this_obj.downcast_ref::<StorageEvent>() {
        return Some(data.event.clone());
    }
    None
}

/// Helper function to mutably access EventData from any event type (EventData, UIEventData, MouseEventData, etc.)
/// This is needed for methods like preventDefault, stopPropagation that modify the event.
fn with_event_data_mut<F, R>(this_obj: &JsObject, f: F) -> Option<R>
where
    F: FnOnce(&mut EventData) -> R,
{
    use super::ui_events::{UIEventData, MouseEventData, KeyboardEventData, FocusEventData, InputEventData};
    use super::custom_event::CustomEventData;
    use super::message_event::MessageEventData;
    use super::error_event::ErrorEventData;
    use super::progress_event::ProgressEventData;
    use super::hash_change_event::HashChangeEventData;
    use super::pop_state_event::PopStateEventData;
    use super::close_event::CloseEventData;
    use super::pageswap_event::PageSwapEventData;
    use crate::storage::storage_event::StorageEvent;

    // Try EventData directly
    if let Some(mut data) = this_obj.downcast_mut::<EventData>() {
        return Some(f(&mut data));
    }
    // Try MouseEventData
    if let Some(mut data) = this_obj.downcast_mut::<MouseEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    // Try KeyboardEventData
    if let Some(mut data) = this_obj.downcast_mut::<KeyboardEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    // Try FocusEventData
    if let Some(mut data) = this_obj.downcast_mut::<FocusEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    // Try InputEventData
    if let Some(mut data) = this_obj.downcast_mut::<InputEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    // Try UIEventData
    if let Some(mut data) = this_obj.downcast_mut::<UIEventData>() {
        return Some(f(&mut data.event));
    }
    // Try CustomEventData
    if let Some(mut data) = this_obj.downcast_mut::<CustomEventData>() {
        return Some(f(&mut data.event));
    }
    // Try MessageEventData
    if let Some(mut data) = this_obj.downcast_mut::<MessageEventData>() {
        return Some(f(&mut data.event));
    }
    // Try ErrorEventData
    if let Some(mut data) = this_obj.downcast_mut::<ErrorEventData>() {
        return Some(f(&mut data.event));
    }
    // Try ProgressEventData
    if let Some(mut data) = this_obj.downcast_mut::<ProgressEventData>() {
        return Some(f(&mut data.event));
    }
    // Try HashChangeEventData
    if let Some(mut data) = this_obj.downcast_mut::<HashChangeEventData>() {
        return Some(f(&mut data.event));
    }
    // Try PopStateEventData
    if let Some(mut data) = this_obj.downcast_mut::<PopStateEventData>() {
        return Some(f(&mut data.event));
    }
    // Try CloseEventData
    if let Some(mut data) = this_obj.downcast_mut::<CloseEventData>() {
        return Some(f(&mut data.event));
    }
    // Try PageSwapEventData
    if let Some(mut data) = this_obj.downcast_mut::<PageSwapEventData>() {
        return Some(f(&mut data.event));
    }
    // Try StorageEvent
    if let Some(mut data) = this_obj.downcast_mut::<StorageEvent>() {
        return Some(f(&mut data.event));
    }
    None
}

/// Get whether the event bubbles.
fn get_bubbles(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_bubbles()))
}

/// Get whether the event is cancelable.
fn get_cancelable(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_cancelable()))
}

/// Get whether preventDefault() has been called.
fn get_default_prevented(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_default_prevented()))
}

/// Get the current event phase.
fn get_event_phase(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    let phase_num = match event_data.get_phase() {
        EventPhase::None => 0u32,
        EventPhase::CapturingPhase => 1u32,
        EventPhase::AtTarget => 2u32,
        EventPhase::BubblingPhase => 3u32,
    };
    Ok(JsValue::from(phase_num))
}

/// Get the current target.
fn get_current_target(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(event_data.get_current_target().map_or(JsValue::null(), |obj| obj.into()))
}

/// Get the event target.
fn get_target(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(event_data.get_target().map_or(JsValue::null(), |obj| obj.into()))
}

/// Get the event timestamp.
fn get_timestamp(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_timestamp()))
}

/// Get whether the event is trusted.
fn get_is_trusted(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_is_trusted()))
}

/// Get whether the event is composed.
fn get_composed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_data = get_event_data_from_obj(&this_obj).ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-Event object")
    })?;

    Ok(JsValue::from(event_data.get_composed()))
}

// Event methods

/// Prevent the default action.
fn prevent_default(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    with_event_data_mut(&this_obj, |event_data| {
        event_data.prevent_default();
    });

    Ok(JsValue::undefined())
}

/// Stop event propagation.
fn stop_propagation(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    with_event_data_mut(&this_obj, |event_data| {
        event_data.stop_propagation();
    });

    Ok(JsValue::undefined())
}

/// Stop immediate event propagation.
fn stop_immediate_propagation(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    with_event_data_mut(&this_obj, |event_data| {
        event_data.stop_immediate_propagation();
    });

    Ok(JsValue::undefined())
}

/// Initialize the event (legacy method).
fn init_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Event method called on non-object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let bubbles = args.get_or_undefined(1).to_boolean();
    let cancelable = args.get_or_undefined(2).to_boolean();
    let event_type_str = event_type.to_std_string_escaped();

    with_event_data_mut(&this_obj, |event_data| {
        event_data.init_event(event_type_str.clone(), bubbles, cancelable);
    });

    Ok(JsValue::undefined())
}