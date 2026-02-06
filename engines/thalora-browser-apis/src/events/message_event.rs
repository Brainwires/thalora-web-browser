//! MessageEvent implementation for Boa
//!
//! Implements the MessageEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/comms.html#messageevent


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

/// JavaScript `MessageEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MessageEvent;

impl IntrinsicObject for MessageEvent {
    fn init(realm: &Realm) {
        let data_getter = BuiltInBuilder::callable(realm, get_data)
            .name(js_string!("get data"))
            .build();

        let origin_getter = BuiltInBuilder::callable(realm, get_origin)
            .name(js_string!("get origin"))
            .build();

        let last_event_id_getter = BuiltInBuilder::callable(realm, get_last_event_id)
            .name(js_string!("get lastEventId"))
            .build();

        let source_getter = BuiltInBuilder::callable(realm, get_source)
            .name(js_string!("get source"))
            .build();

        let ports_getter = BuiltInBuilder::callable(realm, get_ports)
            .name(js_string!("get ports"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("data"),
                Some(data_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("origin"),
                Some(origin_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lastEventId"),
                Some(last_event_id_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("source"),
                Some(source_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ports"),
                Some(ports_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MessageEvent {
    const NAME: JsString = StaticJsStrings::MESSAGE_EVENT;
}

impl BuiltInConstructor for MessageEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::message_event;

    /// `new MessageEvent(type, eventInitDict)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("MessageEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        // Convert type to string
        let event_type = type_arg.to_string(context)?;

        // Create the MessageEvent object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::message_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut data = JsValue::undefined();
        let mut origin = String::new();
        let mut last_event_id = String::new();
        let mut source: Option<JsObject> = None;
        let mut ports: Option<JsObject> = None;

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("data"), context) {
                data = v;
            }
            if let Ok(v) = init_obj.get(js_string!("origin"), context) {
                origin = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("lastEventId"), context) {
                last_event_id = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("source"), context) {
                source = v.as_object();
            }
            if let Ok(v) = init_obj.get(js_string!("ports"), context) {
                ports = v.as_object();
            }
        }

        let event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        let message_event_data = MessageEventData::new(event_data, data, origin, last_event_id, source, ports);

        let message_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            message_event_data,
        );

        Ok(message_event_obj.into())
    }
}

/// Internal data for MessageEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MessageEventData {
    /// Base event data
    pub event: EventData,
    /// The data sent with the message
    data: JsValue,
    /// The origin of the message
    #[unsafe_ignore_trace]
    origin: String,
    /// The last event ID
    #[unsafe_ignore_trace]
    last_event_id: String,
    /// The source of the message
    source: Option<JsObject>,
    /// The ports array
    ports: Option<JsObject>,
}

impl MessageEventData {
    pub fn new(
        event: EventData,
        data: JsValue,
        origin: String,
        last_event_id: String,
        source: Option<JsObject>,
        ports: Option<JsObject>,
    ) -> Self {
        Self { event, data, origin, last_event_id, source, ports }
    }
}

fn get_data(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.data called on non-object")
    })?;

    let message_event = this_obj.downcast_ref::<MessageEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.data called on non-MessageEvent object")
    })?;

    Ok(message_event.data.clone())
}

fn get_origin(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.origin called on non-object")
    })?;

    let message_event = this_obj.downcast_ref::<MessageEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.origin called on non-MessageEvent object")
    })?;

    Ok(js_string!(message_event.origin.clone()).into())
}

fn get_last_event_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.lastEventId called on non-object")
    })?;

    let message_event = this_obj.downcast_ref::<MessageEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.lastEventId called on non-MessageEvent object")
    })?;

    Ok(js_string!(message_event.last_event_id.clone()).into())
}

fn get_source(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.source called on non-object")
    })?;

    let message_event = this_obj.downcast_ref::<MessageEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.source called on non-MessageEvent object")
    })?;

    Ok(message_event.source.clone().map(|s| s.into()).unwrap_or(JsValue::null()))
}

fn get_ports(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.ports called on non-object")
    })?;

    let message_event = this_obj.downcast_ref::<MessageEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageEvent.prototype.ports called on non-MessageEvent object")
    })?;

    Ok(message_event.ports.clone().map(|p| p.into()).unwrap_or(JsValue::undefined()))
}

/// Create a MessageEvent from structured clone data
pub fn create_message_event(
    data: JsValue,
    origin: Option<&str>,
    source: Option<JsValue>,
    ports: Option<JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    let event_data = EventData::new("message".to_string(), false, false);
    let message_event_data = MessageEventData::new(
        event_data,
        data,
        origin.unwrap_or("").to_string(),
        String::new(),
        source.and_then(|s| s.as_object()),
        ports.and_then(|p| p.as_object()),
    );

    let proto = context.intrinsics().constructors().message_event().prototype();
    let message_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        proto,
        message_event_data,
    );

    Ok(message_event.upcast())
}
