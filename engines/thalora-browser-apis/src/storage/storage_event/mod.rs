//! Implementation of the `StorageEvent` Web API.
//!
//! The `StorageEvent` is fired at a Window when a storage area changes.
//! This happens when storage is changed from a different window/context.
//!
//! More information:
//! - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storageevent-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageEvent)

use boa_engine::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use boa_engine::context::intrinsics::StandardConstructor;
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue, builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics, js_string, object::JsObject, property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};

/// `StorageEvent` implementation for the Web Storage API events.
#[derive(Debug, Clone, Finalize)]
pub struct StorageEvent {
    /// The key being changed. null if clear() was called.
    key: std::cell::RefCell<Option<String>>,
    /// The old value. null if the key is new.
    old_value: std::cell::RefCell<Option<String>>,
    /// The new value. null if the key was deleted.
    new_value: std::cell::RefCell<Option<String>>,
    /// The URL of the document whose key changed.
    url: std::cell::RefCell<String>,
    /// The Storage object that was affected.
    storage_area: std::cell::RefCell<Option<JsObject>>,
}

unsafe impl Trace for StorageEvent {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(ref storage_area) = *self.storage_area.borrow() {
            unsafe {
                storage_area.trace(tracer);
            }
        }
    }

    unsafe fn trace_non_roots(&self) {
        if let Some(ref storage_area) = *self.storage_area.borrow() {
            unsafe {
                storage_area.trace_non_roots();
            }
        }
    }

    fn run_finalizer(&self) {
        // No cleanup needed for StorageEvent
    }
}

impl JsData for StorageEvent {}

impl StorageEvent {
    /// Creates a new `StorageEvent` instance.
    pub(crate) fn new(
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        url: String,
        storage_area: Option<JsObject>,
    ) -> Self {
        Self {
            key: std::cell::RefCell::new(key),
            old_value: std::cell::RefCell::new(old_value),
            new_value: std::cell::RefCell::new(new_value),
            url: std::cell::RefCell::new(url),
            storage_area: std::cell::RefCell::new(storage_area),
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
        let event = StorageEvent::new(
            key.clone(),
            old_value.clone(),
            new_value.clone(),
            url.clone(),
            storage_area.clone(),
        );

        let event_obj = JsObject::from_proto_and_data(
            Some(
                context
                    .intrinsics()
                    .constructors()
                    .storage_event()
                    .prototype(),
            ),
            event,
        );

        // Set event properties
        event_obj
            .set(
                js_string!("type"),
                JsValue::from(JsString::from("storage")),
                false,
                context,
            )
            .ok();
        event_obj
            .set(js_string!("bubbles"), JsValue::from(false), false, context)
            .ok();
        event_obj
            .set(
                js_string!("cancelable"),
                JsValue::from(false),
                false,
                context,
            )
            .ok();

        // Set StorageEvent-specific properties
        event_obj
            .set(
                js_string!("key"),
                key.map(|k| JsValue::from(JsString::from(k)))
                    .unwrap_or(JsValue::null()),
                false,
                context,
            )
            .ok();

        event_obj
            .set(
                js_string!("oldValue"),
                old_value
                    .map(|v| JsValue::from(JsString::from(v)))
                    .unwrap_or(JsValue::null()),
                false,
                context,
            )
            .ok();

        event_obj
            .set(
                js_string!("newValue"),
                new_value
                    .map(|v| JsValue::from(JsString::from(v)))
                    .unwrap_or(JsValue::null()),
                false,
                context,
            )
            .ok();

        event_obj
            .set(
                js_string!("url"),
                JsValue::from(JsString::from(url)),
                false,
                context,
            )
            .ok();

        event_obj
            .set(
                js_string!("storageArea"),
                storage_area
                    .map(|s| JsValue::from(s))
                    .unwrap_or(JsValue::null()),
                false,
                context,
            )
            .ok();

        event_obj
    }
}

impl IntrinsicObject for StorageEvent {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("key"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_key)
                        .name(js_string!("get key"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("oldValue"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_old_value)
                        .name(js_string!("get oldValue"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("newValue"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_new_value)
                        .name(js_string!("get newValue"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("url"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_url)
                        .name(js_string!("get url"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("storageArea"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_storage_area)
                        .name(js_string!("get storageArea"))
                        .build(),
                ),
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
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |constructors| constructors.storage_event();

    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let event_type = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let event_init = args.get_or_undefined(1);

        let mut key: Option<String> = None;
        let mut old_value: Option<String> = None;
        let mut new_value: Option<String> = None;
        let mut url = String::from("about:blank");
        let mut storage_area: Option<JsObject> = None;

        if let Some(init_obj) = event_init.as_object() {
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

        let storage_event = StorageEvent::new(key, old_value, new_value, url, storage_area);
        let event_obj = JsObject::from_proto_and_data(
            Some(
                context
                    .intrinsics()
                    .constructors()
                    .storage_event()
                    .prototype(),
            ),
            storage_event,
        );

        // Set basic event properties
        event_obj.set(
            js_string!("type"),
            JsValue::from(JsString::from(event_type)),
            false,
            context,
        )?;
        event_obj.set(js_string!("bubbles"), JsValue::from(false), false, context)?;
        event_obj.set(
            js_string!("cancelable"),
            JsValue::from(false),
            false,
            context,
        )?;

        Ok(event_obj.into())
    }
}

// StorageEvent prototype methods
impl StorageEvent {
    /// `StorageEvent.prototype.key` getter
    fn get_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let key_value = storage_event.key.borrow().clone();
        Ok(key_value
            .map(|k| JsValue::from(JsString::from(k)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.oldValue` getter
    fn get_old_value(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let old_value = storage_event.old_value.borrow().clone();
        Ok(old_value
            .map(|v| JsValue::from(JsString::from(v)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.newValue` getter
    fn get_new_value(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let new_value = storage_event.new_value.borrow().clone();
        Ok(new_value
            .map(|v| JsValue::from(JsString::from(v)))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.url` getter
    fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let url = storage_event.url.borrow().clone();
        Ok(JsValue::from(JsString::from(url)))
    }

    /// `StorageEvent.prototype.storageArea` getter
    fn get_storage_area(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        let storage_area = storage_event.storage_area.borrow().clone();
        Ok(storage_area
            .map(|s| JsValue::from(s))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.initStorageEvent(type, bubbles, cancelable, key, oldValue, newValue, url, storageArea)`
    fn init_storage_event(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // First, extract all values from args before borrowing the object
        let event_type = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let _bubbles = args.get_or_undefined(1).to_boolean();
        let _cancelable = args.get_or_undefined(2).to_boolean();

        let key_value =
            if args.get_or_undefined(3).is_null() || args.get_or_undefined(3).is_undefined() {
                None
            } else {
                Some(
                    args.get_or_undefined(3)
                        .to_string(context)?
                        .to_std_string_escaped(),
                )
            };

        let old_value =
            if args.get_or_undefined(4).is_null() || args.get_or_undefined(4).is_undefined() {
                None
            } else {
                Some(
                    args.get_or_undefined(4)
                        .to_string(context)?
                        .to_std_string_escaped(),
                )
            };

        let new_value =
            if args.get_or_undefined(5).is_null() || args.get_or_undefined(5).is_undefined() {
                None
            } else {
                Some(
                    args.get_or_undefined(5)
                        .to_string(context)?
                        .to_std_string_escaped(),
                )
            };

        let url_value = args
            .get_or_undefined(6)
            .to_string(context)?
            .to_std_string_escaped();

        let storage_area_value =
            if args.get_or_undefined(7).is_null() || args.get_or_undefined(7).is_undefined() {
                None
            } else {
                args.get_or_undefined(7).as_object().map(|o| o.clone())
            };

        // Now borrow the object and update fields
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a StorageEvent object")
        })?;

        {
            let storage_event = obj.downcast_ref::<StorageEvent>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a StorageEvent object")
            })?;

            // Update internal fields using RefCell::replace()
            storage_event.key.replace(key_value);
            storage_event.old_value.replace(old_value);
            storage_event.new_value.replace(new_value);
            storage_event.url.replace(url_value);
            storage_event.storage_area.replace(storage_area_value);
        } // Drop storage_event reference here

        // Set properties on the event object for compatibility
        obj.set(
            js_string!("type"),
            JsValue::from(JsString::from(event_type)),
            false,
            context,
        )?;

        Ok(JsValue::undefined())
    }
}

#[cfg(test)]
mod tests;
