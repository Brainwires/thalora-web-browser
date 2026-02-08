//! DOMStringMap implementation (element.dataset)
//!
//! Returns a plain JS object populated with all `data-*` attributes from the element,
//! with keys converted from kebab-case to camelCase per the HTML specification.
//!
//! See: https://html.spec.whatwg.org/multipage/dom.html#domstringmap

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string, object::JsObject, property::Attribute, realm::Realm,
    string::JsString, Context, JsData, JsNativeError, JsResult, JsValue,
};
use crate::dom::element::with_element_data;
use boa_gc::{Finalize, Trace};

/// Convert a `data-*` attribute name to its camelCase dataset property name.
///
/// Per the HTML spec:
/// 1. Strip the `data-` prefix
/// 2. For each ASCII lowercase letter immediately preceded by a `-`, remove the `-`
///    and uppercase the letter
///
/// Examples:
/// - `data-foo-bar` → `fooBar`
/// - `data-v-app` → `vApp`
/// - `data-server-rendered` → `serverRendered`
/// - `data-id` → `id`
fn data_attr_to_camel_case(attr_name: &str) -> String {
    // Strip the "data-" prefix
    let name = &attr_name[5..]; // "data-" is 5 chars

    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = false;

    for ch in name.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Internal data for DOMStringMap objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DOMStringMapData {
    /// The associated element object
    element: JsObject,
}

impl DOMStringMapData {
    pub fn new(element: JsObject) -> Self {
        Self { element }
    }
}

/// The `DOMStringMap` builtin
#[derive(Debug, Trace, Finalize)]
pub struct DOMStringMap;

impl DOMStringMap {
    /// Create a DOMStringMap object for the given element.
    ///
    /// Populates the returned JS object with all current `data-*` attributes
    /// from the element, converted to camelCase keys.
    pub fn create_for_element(element: JsObject, context: &mut Context) -> JsResult<JsObject> {
        let data = DOMStringMapData::new(element.clone());
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().domstringmap().prototype(),
            data,
        );
        let obj = obj.upcast();

        // Populate with current data-* attributes from the element
        let attrs = with_element_data(&element, |el| {
            let attributes = el.attributes.lock().unwrap();
            let mut data_attrs = Vec::new();
            for (key, value) in attributes.iter() {
                if key.starts_with("data-") && key.len() > 5 {
                    let camel_key = data_attr_to_camel_case(key);
                    data_attrs.push((camel_key, value.clone()));
                }
            }
            data_attrs
        }, "DOMStringMap: not an element")?;

        for (key, value) in attrs {
            obj.set(
                JsString::from(key),
                JsString::from(value),
                false,
                context,
            )?;
        }

        Ok(obj)
    }
}

impl IntrinsicObject for DOMStringMap {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DOMStringMap {
    const NAME: JsString = js_string!("DOMStringMap");
}

impl BuiltInConstructor for DOMStringMap {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::domstringmap;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // DOMStringMap is not directly constructable
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_attr_to_camel_case() {
        assert_eq!(data_attr_to_camel_case("data-foo-bar"), "fooBar");
        assert_eq!(data_attr_to_camel_case("data-v-app"), "vApp");
        assert_eq!(data_attr_to_camel_case("data-server-rendered"), "serverRendered");
        assert_eq!(data_attr_to_camel_case("data-id"), "id");
        assert_eq!(data_attr_to_camel_case("data-x"), "x");
        assert_eq!(data_attr_to_camel_case("data-foo-bar-baz"), "fooBarBaz");
    }
}
