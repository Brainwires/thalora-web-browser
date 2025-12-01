//! NamedNodeMap interface implementation for DOM Level 4
//!
//! The NamedNodeMap interface represents a collection of Attr objects.
//! https://dom.spec.whatwg.org/#interface-namednodemap

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::JsString,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
    property::PropertyDescriptorBuilder,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The NamedNodeMap data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NamedNodeMapData {
    /// Map of attribute name to Attr object
    #[unsafe_ignore_trace]
    attributes: Arc<Mutex<HashMap<String, JsObject>>>,
    /// Ordered list of attribute names (for indexed access)
    #[unsafe_ignore_trace]
    order: Arc<Mutex<Vec<String>>>,
}

impl NamedNodeMapData {
    /// Create a new NamedNodeMap
    pub fn new() -> Self {
        Self {
            attributes: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create from a HashMap of attributes
    pub fn from_attributes(attrs: HashMap<String, String>, context: &mut Context) -> JsResult<Self> {
        let data = Self::new();
        for (name, value) in attrs {
            // Create Attr object
            let attr = JsObject::default(context.intrinsics());
            attr.define_property_or_throw(
                js_string!("nodeType"),
                PropertyDescriptorBuilder::new()
                    .value(JsValue::from(2))
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            attr.define_property_or_throw(
                js_string!("nodeName"),
                PropertyDescriptorBuilder::new()
                    .value(js_string!(name.clone()))
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            attr.define_property_or_throw(
                js_string!("name"),
                PropertyDescriptorBuilder::new()
                    .value(js_string!(name.clone()))
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            attr.define_property_or_throw(
                js_string!("value"),
                PropertyDescriptorBuilder::new()
                    .value(js_string!(value))
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;
            attr.define_property_or_throw(
                js_string!("specified"),
                PropertyDescriptorBuilder::new()
                    .value(JsValue::from(true))
                    .writable(false)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                context,
            )?;

            data.attributes.lock().unwrap().insert(name.clone(), attr);
            data.order.lock().unwrap().push(name);
        }
        Ok(data)
    }

    /// Get the length
    pub fn length(&self) -> usize {
        self.order.lock().unwrap().len()
    }

    /// Get item by index
    pub fn get_item(&self, index: usize) -> Option<JsObject> {
        let order = self.order.lock().unwrap();
        let name = order.get(index)?;
        self.attributes.lock().unwrap().get(name).cloned()
    }

    /// Get named item
    pub fn get_named_item(&self, name: &str) -> Option<JsObject> {
        self.attributes.lock().unwrap().get(name).cloned()
    }

    /// Set named item
    pub fn set_named_item(&self, name: String, attr: JsObject) -> Option<JsObject> {
        let mut attrs = self.attributes.lock().unwrap();
        let old = attrs.insert(name.clone(), attr);
        if old.is_none() {
            self.order.lock().unwrap().push(name);
        }
        old
    }

    /// Remove named item
    pub fn remove_named_item(&self, name: &str) -> Option<JsObject> {
        let mut attrs = self.attributes.lock().unwrap();
        let removed = attrs.remove(name);
        if removed.is_some() {
            self.order.lock().unwrap().retain(|n| n != name);
        }
        removed
    }
}

/// The `NamedNodeMap` object
#[derive(Debug, Trace, Finalize)]
pub struct NamedNodeMap;

impl NamedNodeMap {
    /// Create a new NamedNodeMap
    pub fn create(context: &mut Context) -> JsResult<JsObject> {
        let data = NamedNodeMapData::new();
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().namednodemap().prototype(),
            data,
        );
        Ok(obj)
    }

    /// `NamedNodeMap.prototype.length` getter
    fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NamedNodeMap.length called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NamedNodeMapData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("NamedNodeMap.length called on non-NamedNodeMap object")
        })?;

        Ok(JsValue::new(data.length() as i32))
    }

    /// `NamedNodeMap.prototype.item(index)`
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NamedNodeMap.item called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NamedNodeMapData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("NamedNodeMap.item called on non-NamedNodeMap object")
        })?;

        let index = args.get_or_undefined(0).to_length(context)? as usize;

        match data.get_item(index) {
            Some(attr) => Ok(attr.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NamedNodeMap.prototype.getNamedItem(name)`
    fn get_named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NamedNodeMap.getNamedItem called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NamedNodeMapData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("NamedNodeMap.getNamedItem called on non-NamedNodeMap object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        match data.get_named_item(&name_str) {
            Some(attr) => Ok(attr.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NamedNodeMap.prototype.setNamedItem(attr)`
    fn set_named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NamedNodeMap.setNamedItem called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NamedNodeMapData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("NamedNodeMap.setNamedItem called on non-NamedNodeMap object")
        })?;

        let attr = args.get_or_undefined(0);
        let attr_obj = attr.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("setNamedItem argument must be an Attr")
        })?;

        // Get the name from the attr object
        let name_val = attr_obj.get(js_string!("name"), context)?;
        let name_str = name_val.to_string(context)?.to_std_string_escaped();

        match data.set_named_item(name_str, attr_obj.clone()) {
            Some(old) => Ok(old.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NamedNodeMap.prototype.removeNamedItem(name)`
    fn remove_named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NamedNodeMap.removeNamedItem called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NamedNodeMapData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("NamedNodeMap.removeNamedItem called on non-NamedNodeMap object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        match data.remove_named_item(&name_str) {
            Some(removed) => Ok(removed.into()),
            None => Err(JsNativeError::typ()
                .with_message(format!("No attribute named '{}' found", name_str))
                .into()),
        }
    }
}

impl IntrinsicObject for NamedNodeMap {
    fn init(realm: &Realm) {
        let length_getter = BuiltInBuilder::callable(realm, Self::get_length)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::item, js_string!("item"), 1)
            .method(Self::get_named_item, js_string!("getNamedItem"), 1)
            .method(Self::set_named_item, js_string!("setNamedItem"), 1)
            .method(Self::remove_named_item, js_string!("removeNamedItem"), 1)
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NamedNodeMap {
    const NAME: JsString = js_string!("NamedNodeMap");
}

impl BuiltInConstructor for NamedNodeMap {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::namednodemap;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor NamedNodeMap requires 'new'")
                .into());
        }

        let data = NamedNodeMapData::new();
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().namednodemap().prototype(),
            data,
        );

        Ok(obj.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namednodemap_data_creation() {
        let data = NamedNodeMapData::new();
        assert_eq!(data.length(), 0);
    }
}
