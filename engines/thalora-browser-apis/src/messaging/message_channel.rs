//! MessageChannel Web API implementation for Boa
//!
//! Native implementation of MessageChannel standard
//! https://html.spec.whatwg.org/multipage/web-messaging.html#message-channels
//!
//! This implements the complete MessageChannel interface for creating communication channels


use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
#[cfg(feature = "native")]
use crate::worker::worker_events;
use boa_gc::{Finalize, Trace};
use super::message_port::MessagePortData;

/// JavaScript `MessageChannel` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct MessageChannel;

/// Internal data for MessageChannel instances
#[derive(Debug, Trace, Finalize, JsData)]
struct MessageChannelData {
    port1: JsObject,
    port2: JsObject,
}

impl IntrinsicObject for MessageChannel {
    fn init(realm: &Realm) {
        let port1_func = BuiltInBuilder::callable(realm, get_port1)
            .name(js_string!("get port1"))
            .build();

        let port2_func = BuiltInBuilder::callable(realm, get_port2)
            .name(js_string!("get port2"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance properties
            .accessor(
                js_string!("port1"),
                Some(port1_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("port2"),
                Some(port2_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().message_channel().constructor()
    }
}

impl BuiltInObject for MessageChannel {
    const NAME: JsString = StaticJsStrings::MESSAGE_CHANNEL;
}

impl BuiltInConstructor for MessageChannel {
    const PROTOTYPE_STORAGE_SLOTS: usize = 100; // prototype property capacity
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100; // static property capacity
    const CONSTRUCTOR_ARGUMENTS: usize = 0; // no required parameters

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::message_channel;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Ensure 'new' was used
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("MessageChannel constructor requires 'new'")
                .into());
        }

        // Create entangled MessagePort pair
        let (port1_data, port2_data) = MessagePortData::create_entangled_pair();

        // Create MessagePort objects using the proper MessagePort prototype
        let port1 = port1_data.create_js_object(context)?;
        let port2 = port2_data.create_js_object(context)?;

        // Create the MessageChannel object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::message_channel, context)?;
        let channel_data = MessageChannelData { port1, port2 };
        let channel_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            channel_data,
        );

        Ok(channel_obj.into())
    }
}

/// Get the port1 property of the MessageChannel
fn get_port1(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageChannel port1 getter called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessageChannelData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("'this' is not a MessageChannel object")
    })?;

    Ok(data.port1.clone().into())
}

/// Get the port2 property of the MessageChannel
fn get_port2(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageChannel port2 getter called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessageChannelData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("'this' is not a MessageChannel object")
    })?;

    Ok(data.port2.clone().into())
}

/// Helper function to create a MessagePort object with proper prototype and methods
fn create_message_port_object(data: MessagePortData, context: &mut Context) -> JsResult<JsObject> {
    // Create the object with MessagePort prototype
    let prototype = context.intrinsics().constructors().message_port().prototype();
    let port_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        data,
    );

    // Add MessagePort methods
    let post_message_func = BuiltInBuilder::callable(context.realm(), message_port_post_message)
        .name(js_string!("postMessage"))
        .length(1)
        .build();

    let start_func = BuiltInBuilder::callable(context.realm(), message_port_start)
        .name(js_string!("start"))
        .length(0)
        .build();

    let close_func = BuiltInBuilder::callable(context.realm(), message_port_close)
        .name(js_string!("close"))
        .length(0)
        .build();

    // Define methods on the port object using PropertyDescriptorBuilder
    use boa_engine::property::PropertyDescriptorBuilder;

    let port_obj_generic = port_obj.upcast();

    port_obj_generic.define_property_or_throw(
        js_string!("postMessage"),
        PropertyDescriptorBuilder::new()
            .value(post_message_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    port_obj_generic.define_property_or_throw(
        js_string!("start"),
        PropertyDescriptorBuilder::new()
            .value(start_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    port_obj_generic.define_property_or_throw(
        js_string!("close"),
        PropertyDescriptorBuilder::new()
            .value(close_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Add event handler properties to MessagePort (native only)
    #[cfg(feature = "native")]
    worker_events::add_worker_event_handlers(&port_obj_generic, context)?;

    Ok(port_obj_generic)
}

/// MessagePort postMessage implementation
fn message_port_post_message(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::misc::structured_clone::{structured_clone, structured_deserialize};

    let message = args.get_or_undefined(0);

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.postMessage called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    if !data.is_entangled() {
        return Err(JsNativeError::typ()
            .with_message("Cannot post message on disentangled port")
            .into());
    }

    // Parse optional transfer list from second argument
    // Per spec: postMessage(message, transfer?) or postMessage(message, options?)
    let transfer_list = if args.len() > 1 {
        let transfer_arg = args.get_or_undefined(1);
        if transfer_arg.is_object() {
            // Could be an options object {transfer: [...]} or an array directly
            // For now, we don't support transferables through MessagePort
            None
        } else {
            None
        }
    } else {
        None
    };

    // Perform structured cloning of the message
    let cloned_value = structured_clone(message, context, transfer_list.as_ref())?;

    // Deserialize back to JsValue for the message queue
    // In a full implementation, we'd send the serialized form for cross-thread transfer
    let deserialized_message = structured_deserialize(&cloned_value, context)?;

    data.post_message(deserialized_message)?;

    Ok(JsValue::undefined())
}

/// MessagePort start implementation
fn message_port_start(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.start called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    data.start();
    Ok(JsValue::undefined())
}

/// MessagePort close implementation
fn message_port_close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.close called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    data.close();
    Ok(JsValue::undefined())
}