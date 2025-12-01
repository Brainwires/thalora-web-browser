//! HTMLCollection interface implementation for DOM Level 4
//!
//! The HTMLCollection interface represents a live collection of elements.
//! https://dom.spec.whatwg.org/#interface-htmlcollection

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::HashMap;

/// The HTMLCollection data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLCollectionData {
    /// The collection of elements
    elements: GcRefCell<Vec<JsObject>>,
    /// Named items (by id and name)
    #[unsafe_ignore_trace]
    named_items: std::sync::Arc<std::sync::Mutex<HashMap<String, JsObject>>>,
}

impl HTMLCollectionData {
    /// Create a new HTMLCollection with the given elements
    pub fn new(elements: Vec<JsObject>) -> Self {
        Self {
            elements: GcRefCell::new(elements),
            named_items: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Create an empty HTMLCollection
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Get the length of the HTMLCollection
    pub fn length(&self) -> usize {
        self.elements.borrow().len()
    }

    /// Get the element at the specified index
    pub fn get_item(&self, index: usize) -> Option<JsObject> {
        self.elements.borrow().get(index).cloned()
    }

    /// Get an element by name or id
    pub fn get_named_item(&self, name: &str) -> Option<JsObject> {
        self.named_items.lock().unwrap().get(name).cloned()
    }

    /// Add a named item
    pub fn add_named_item(&self, name: String, element: JsObject) {
        self.named_items.lock().unwrap().insert(name, element);
    }

    /// Get all elements as a vector
    pub fn elements(&self) -> Vec<JsObject> {
        self.elements.borrow().clone()
    }

    /// Add an element to the collection
    pub fn add_element(&self, element: JsObject) {
        self.elements.borrow_mut().push(element);
    }
}

/// The `HTMLCollection` object
#[derive(Debug, Trace, Finalize)]
pub struct HTMLCollection;

impl HTMLCollection {
    /// Create a new HTMLCollection from a vector of elements
    pub fn create_from_elements(elements: Vec<JsObject>, context: &mut Context) -> JsResult<JsObject> {
        let collection_data = HTMLCollectionData::new(elements);

        let collection_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().htmlcollection().prototype(),
            collection_data,
        );

        Ok(collection_obj)
    }

    /// `HTMLCollection.prototype.length` getter
    fn get_length_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLCollection.length called on non-object")
        })?;

        let collection_data = this_obj.downcast_ref::<HTMLCollectionData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLCollection.length called on non-HTMLCollection object")
        })?;

        Ok(JsValue::new(collection_data.length() as i32))
    }

    /// `HTMLCollection.prototype.item(index)`
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLCollection.item called on non-object")
        })?;

        let collection_data = this_obj.downcast_ref::<HTMLCollectionData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLCollection.item called on non-HTMLCollection object")
        })?;

        let index = args.get_or_undefined(0).to_length(context)? as usize;

        match collection_data.get_item(index) {
            Some(element) => Ok(element.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `HTMLCollection.prototype.namedItem(name)`
    fn named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLCollection.namedItem called on non-object")
        })?;

        let collection_data = this_obj.downcast_ref::<HTMLCollectionData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("HTMLCollection.namedItem called on non-HTMLCollection object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_escaped();

        // First check named items map
        if let Some(element) = collection_data.get_named_item(&name_str) {
            return Ok(element.into());
        }

        // Then search through elements for matching id or name attribute
        for element in collection_data.elements() {
            // Try to get id attribute
            if let Ok(id_val) = element.get(js_string!("id"), context) {
                if let Ok(id_str) = id_val.to_string(context) {
                    if id_str.to_std_string_escaped() == name_str {
                        return Ok(element.into());
                    }
                }
            }
            // Try to get name attribute
            if let Ok(name_val) = element.get(js_string!("name"), context) {
                if let Ok(elem_name_str) = name_val.to_string(context) {
                    if elem_name_str.to_std_string_escaped() == name_str {
                        return Ok(element.into());
                    }
                }
            }
        }

        Ok(JsValue::null())
    }
}

impl IntrinsicObject for HTMLCollection {
    fn init(realm: &Realm) {
        let length_get_func = BuiltInBuilder::callable(realm, Self::get_length_accessor)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .method(Self::item, js_string!("item"), 1)
            .method(Self::named_item, js_string!("namedItem"), 1)
            // Properties
            .accessor(
                js_string!("length"),
                Some(length_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLCollection {
    const NAME: JsString = js_string!("HTMLCollection");
}

impl BuiltInConstructor for HTMLCollection {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::htmlcollection;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // HTMLCollection constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor HTMLCollection requires 'new'")
                .into());
        }

        // Create a new empty HTMLCollection object
        let collection_data = HTMLCollectionData::empty();

        let collection_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().htmlcollection().prototype(),
            collection_data,
        );

        Ok(collection_obj.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_htmlcollection_data_creation() {
        let data = HTMLCollectionData::empty();
        assert_eq!(data.length(), 0);
    }
}
