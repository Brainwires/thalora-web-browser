//! Implementation of the `StorageEvent` Web API.
//!
//! The `StorageEvent` is fired at a Window when a storage area changes.
//! This happens when storage is changed from a different window/context.
//!
//! More information:
//! - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storageevent-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageEvent)

use boa_gc::{Finalize, Trace};
use boa_engine::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_engine::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use boa_engine::context::intrinsics::StandardConstructor;
use boa_engine::object::internal_methods::get_prototype_from_constructor;
use boa_engine::context::intrinsics::StandardConstructors;

use crate::events::event::EventData;

/// `StorageEvent` implementation for the Web Storage API events.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct StorageEvent {
    /// Base event data
    pub event: EventData,
    /// The key being changed. null if clear() was called.
    #[unsafe_ignore_trace]
    key: Option<String>,
    /// The old value. null if the key is new.
    #[unsafe_ignore_trace]
    old_value: Option<String>,
    /// The new value. null if the key was deleted.
    #[unsafe_ignore_trace]
    new_value: Option<String>,
    /// The URL of the document whose key changed.
    #[unsafe_ignore_trace]
    url: String,
    /// The Storage object that was affected.
    storage_area: Option<JsObject>,
}

impl StorageEvent {
    /// Creates a new `StorageEvent` instance.
    pub(crate) fn new(
        event: EventData,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        url: String,
        storage_area: Option<JsObject>,
    ) -> Self {
        Self {
            event,
            key,
            old_value,
            new_value,
            url,
            storage_area,
        }
    }

    /// Creates a StorageEvent object from parameters
    pub fn create_storage_event(
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        url: String,
        storage_area: Option<JsObject>,
        context: &mut Context,
    ) -> JsObject {
        let event_data = EventData::new("storage".to_string(), false, false);
        let event = StorageEvent::new(event_data, key, old_value, new_value, url, storage_area);

        let event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().storage_event().prototype(),
            event
        );

        event_obj.upcast()
    }
}

impl IntrinsicObject for StorageEvent {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("key"),
                Some(BuiltInBuilder::callable(realm, Self::get_key)
                    .name(js_string!("get key"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("oldValue"),
                Some(BuiltInBuilder::callable(realm, Self::get_old_value)
                    .name(js_string!("get oldValue"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("newValue"),
                Some(BuiltInBuilder::callable(realm, Self::get_new_value)
                    .name(js_string!("get newValue"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("url"),
                Some(BuiltInBuilder::callable(realm, Self::get_url)
                    .name(js_string!("get url"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("storageArea"),
                Some(BuiltInBuilder::callable(realm, Self::get_storage_area)
                    .name(js_string!("get storageArea"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::init_storage_event, js_string!("initStorageEvent"), 5)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for StorageEvent {
    const NAME: JsString = js_string!("StorageEvent");
}

impl BuiltInConstructor for StorageEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 12;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&boa_engine::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.storage_event();

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let event_init = args.get_or_undefined(1);

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::storage_event, context)?;

        let mut bubbles = false;
        let mut cancelable = false;
        let mut key: Option<String> = None;
        let mut old_value: Option<String> = None;
        let mut new_value: Option<String> = None;
        let mut url = String::from("about:blank");
        let mut storage_area: Option<JsObject> = None;

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(key_val) = init_obj.get(js_string!("key"), context) {
                if !key_val.is_null() && !key_val.is_undefined() {
                    key = Some(key_val.to_string(context)?.to_std_string_escaped());
                }
            }
            if let Ok(old_val) = init_obj.get(js_string!("oldValue"), context) {
                if !old_val.is_null() && !old_val.is_undefined() {
                    old_value = Some(old_val.to_string(context)?.to_std_string_escaped());
                }
            }
            if let Ok(new_val) = init_obj.get(js_string!("newValue"), context) {
                if !new_val.is_null() && !new_val.is_undefined() {
                    new_value = Some(new_val.to_string(context)?.to_std_string_escaped());
                }
            }
            if let Ok(url_val) = init_obj.get(js_string!("url"), context) {
                if !url_val.is_null() && !url_val.is_undefined() {
                    url = url_val.to_string(context)?.to_std_string_escaped();
                }
            }
            if let Ok(storage_val) = init_obj.get(js_string!("storageArea"), context) {
                if let Some(storage_obj) = storage_val.as_object() {
                    storage_area = Some(storage_obj.clone());
                }
            }
        }

        let event_data = EventData::new(event_type, bubbles, cancelable);
        let storage_event = StorageEvent::new(event_data, key, old_value, new_value, url, storage_area);

        let event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            storage_event
        );

        Ok(event_obj.into())
    }
}

// StorageEvent prototype methods
impl StorageEvent {
    /// `StorageEvent.prototype.key` getter
    fn get_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.key.clone()
            .map(|k| JsValue::from(JsString::from(k)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.oldValue` getter
    fn get_old_value(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.old_value.clone()
            .map(|v| JsValue::from(JsString::from(v)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.newValue` getter
    fn get_new_value(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.new_value.clone()
            .map(|v| JsValue::from(JsString::from(v)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.url` getter
    fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(JsValue::from(JsString::from(storage_event.url.clone())))
    }

    /// `StorageEvent.prototype.storageArea` getter
    fn get_storage_area(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.storage_area.clone()
            .map(|s| JsValue::from(s))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.initStorageEvent(type, bubbles, cancelable, key, oldValue, newValue, url, storageArea)`
    fn init_storage_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // First, extract all values from args before borrowing the object
        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let bubbles = args.get_or_undefined(1).to_boolean();
        let cancelable = args.get_or_undefined(2).to_boolean();

        let key_value = if args.get_or_undefined(3).is_null() || args.get_or_undefined(3).is_undefined() {
            None
        } else {
            Some(args.get_or_undefined(3).to_string(context)?.to_std_string_escaped())
        };

        let old_value = if args.get_or_undefined(4).is_null() || args.get_or_undefined(4).is_undefined() {
            None
        } else {
            Some(args.get_or_undefined(4).to_string(context)?.to_std_string_escaped())
        };

        let new_value = if args.get_or_undefined(5).is_null() || args.get_or_undefined(5).is_undefined() {
            None
        } else {
            Some(args.get_or_undefined(5).to_string(context)?.to_std_string_escaped())
        };

        let url_value = args.get_or_undefined(6).to_string(context)?.to_std_string_escaped();

        let storage_area_value = if args.get_or_undefined(7).is_null() || args.get_or_undefined(7).is_undefined() {
            None
        } else {
            args.get_or_undefined(7).as_object().map(|o| o.clone())
        };

        // Now borrow the object and update fields
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        if let Some(mut storage_event) = obj.downcast_mut::<StorageEvent>() {
            // Update the embedded EventData
            storage_event.event.init_event(event_type, bubbles, cancelable);
            // Update StorageEvent-specific fields
            storage_event.key = key_value;
            storage_event.old_value = old_value;
            storage_event.new_value = new_value;
            storage_event.url = url_value;
            storage_event.storage_area = storage_area_value;
        }

        Ok(JsValue::undefined())
    }
}



#[cfg(test)]
mod tests;
