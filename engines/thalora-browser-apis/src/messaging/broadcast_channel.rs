//! BroadcastChannel Web API implementation for Boa
//!
//! Native implementation of BroadcastChannel standard
//! https://html.spec.whatwg.org/multipage/web-messaging.html#broadcasting-to-other-browsing-contexts
//!
//! This implements the complete BroadcastChannel interface for cross-context communication


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
use std::sync::{Arc, Mutex, Weak};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global registry for BroadcastChannel instances
/// Stores weak references to all active channels, grouped by channel name
static BROADCAST_REGISTRY: OnceLock<Mutex<BroadcastRegistry>> = OnceLock::new();

fn get_broadcast_registry() -> &'static Mutex<BroadcastRegistry> {
    BROADCAST_REGISTRY.get_or_init(|| Mutex::new(BroadcastRegistry::new()))
}

/// Registry that tracks all active BroadcastChannel instances
#[derive(Debug, Default)]
struct BroadcastRegistry {
    /// Map of channel name -> list of (channel_id, sender)
    /// Using channel_id to identify channels for exclusion when broadcasting
    channels: HashMap<String, Vec<(u64, Sender<BroadcastMessage>)>>,
    /// Counter for generating unique channel IDs
    next_id: u64,
}

/// Message sent through BroadcastChannel
#[derive(Debug, Clone)]
struct BroadcastMessage {
    /// Serialized message data (using structured clone format)
    data: Vec<u8>,
    /// Origin of the message (for MessageEvent)
    origin: String,
}

impl BroadcastRegistry {
    fn new() -> Self {
        Self {
            channels: HashMap::new(),
            next_id: 0,
        }
    }

    /// Register a new channel and return its unique ID
    fn register(&mut self, name: &str, sender: Sender<BroadcastMessage>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.channels
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push((id, sender));

        id
    }

    /// Unregister a channel by name and ID
    fn unregister(&mut self, name: &str, id: u64) {
        if let Some(channels) = self.channels.get_mut(name) {
            channels.retain(|(channel_id, _)| *channel_id != id);
            // Clean up empty entries
            if channels.is_empty() {
                self.channels.remove(name);
            }
        }
    }

    /// Broadcast a message to all channels with the given name, except the sender
    fn broadcast(&self, name: &str, sender_id: u64, message: BroadcastMessage) {
        if let Some(channels) = self.channels.get(name) {
            for (id, sender) in channels {
                if *id != sender_id {
                    // Send to other channels (ignore send failures - channel may be closed)
                    let _ = sender.send(message.clone());
                }
            }
        }
    }
}

/// JavaScript `BroadcastChannel` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct BroadcastChannel;

impl IntrinsicObject for BroadcastChannel {
    fn init(realm: &Realm) {
        let name_getter = BuiltInBuilder::callable(realm, get_name)
            .name(js_string!("get name"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event_target().prototype()))
            // Instance properties
            .accessor(
                js_string!("name"),
                Some(name_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            // Instance methods - add them to the constructor's prototype
            .property(
                js_string!("postMessage"),
                BuiltInBuilder::callable(realm, post_message)
                    .name(js_string!("postMessage"))
                    .length(1)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("close"),
                BuiltInBuilder::callable(realm, close)
                    .name(js_string!("close"))
                    .length(0)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for BroadcastChannel {
    const NAME: JsString = StaticJsStrings::BROADCAST_CHANNEL;
}

impl BuiltInConstructor for BroadcastChannel {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::broadcast_channel;

    /// `new BroadcastChannel(name)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("BroadcastChannel constructor requires 'new'")
                .into());
        }

        let name_arg = args.get_or_undefined(0);

        // Convert name to string
        let channel_name = name_arg.to_string(context)?.to_std_string_escaped();

        // Create the BroadcastChannel object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::broadcast_channel, context)?;

        // Create channel for receiving messages from other BroadcastChannel instances
        let (sender, receiver) = unbounded::<BroadcastMessage>();

        // Register this channel in the global registry
        let channel_id = {
            let mut registry = get_broadcast_registry().lock().map_err(|_| {
                JsNativeError::error().with_message("Failed to lock BroadcastChannel registry")
            })?;
            registry.register(&channel_name, sender)
        };

        let channel_data = BroadcastChannelData::new(channel_name.clone(), channel_id, receiver);

        let channel_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            channel_data,
        );

        // Add event handler properties (onmessage, onmessageerror)
        let channel_obj_generic = channel_obj.upcast();
        channel_obj_generic.set(js_string!("onmessage"), JsValue::null(), false, context)?;
        channel_obj_generic.set(js_string!("onmessageerror"), JsValue::null(), false, context)?;

        Ok(channel_obj_generic.into())
    }
}


/// Internal data for BroadcastChannel instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct BroadcastChannelData {
    #[unsafe_ignore_trace]
    name: String,
    /// Unique ID for this channel instance (used for registry operations)
    #[unsafe_ignore_trace]
    id: u64,
    /// Receiver for messages from other channels with the same name
    #[unsafe_ignore_trace]
    receiver: Arc<Mutex<Option<Receiver<BroadcastMessage>>>>,
    #[unsafe_ignore_trace]
    closed: Arc<Mutex<bool>>,
}

impl BroadcastChannelData {
    fn new(name: String, id: u64, receiver: Receiver<BroadcastMessage>) -> Self {
        Self {
            name,
            id,
            receiver: Arc::new(Mutex::new(Some(receiver))),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

    fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        if *closed {
            return; // Already closed
        }
        *closed = true;
        *self.receiver.lock().unwrap() = None;

        // Unregister from global registry
        if let Ok(mut registry) = get_broadcast_registry().lock() {
            registry.unregister(&self.name, self.id);
        }
    }
}


// Property getters and methods

/// `BroadcastChannel.prototype.name` getter
fn get_name(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("BroadcastChannel.name getter called on non-object")
    })?;

    let data = this_obj.downcast_ref::<BroadcastChannelData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("BroadcastChannel.name getter called on invalid object")
    })?;

    Ok(JsValue::from(js_string!(data.name.clone())))
}

/// `BroadcastChannel.prototype.postMessage(message)`
fn post_message(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::misc::structured_clone::{structured_clone, StructuredClone};

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("BroadcastChannel.postMessage called on non-object")
    })?;

    let data = this_obj.downcast_ref::<BroadcastChannelData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("BroadcastChannel.postMessage called on invalid object")
    })?;

    if data.is_closed() {
        return Err(JsNativeError::error()
            .with_message("BroadcastChannel is closed")
            .into());
    }

    let message = args.get_or_undefined(0);

    // BroadcastChannel doesn't support transferable objects
    // Perform structured cloning of the message
    let cloned_value = structured_clone(message, context, None)?;

    // Serialize the cloned value for cross-context transmission
    let serialized_data = StructuredClone::serialize_to_bytes(&cloned_value).map_err(|e| {
        JsNativeError::error().with_message(format!("Failed to serialize message: {}", e))
    })?;

    // Create the broadcast message
    let broadcast_msg = BroadcastMessage {
        data: serialized_data,
        origin: String::new(), // Origin will be set by the receiving context
    };

    // Broadcast to all other channels with the same name
    let channel_name = data.name.clone();
    let channel_id = data.id;

    if let Ok(registry) = get_broadcast_registry().lock() {
        registry.broadcast(&channel_name, channel_id, broadcast_msg);
    }

    Ok(JsValue::undefined())
}

/// `BroadcastChannel.prototype.close()`
fn close(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("BroadcastChannel.close called on non-object")
    })?;

    let data = this_obj.downcast_ref::<BroadcastChannelData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("BroadcastChannel.close called on invalid object")
    })?;

    data.close();

    eprintln!("BroadcastChannel '{}' closed", data.name);

    Ok(JsValue::undefined())
}