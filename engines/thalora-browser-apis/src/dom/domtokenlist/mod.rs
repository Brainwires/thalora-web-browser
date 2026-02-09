//! DOMTokenList implementation (classList) - minimal spec-aligned subset
//!
//! Implements: add, remove, toggle, contains, item, length, toString
use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string, object::JsObject, property::Attribute, realm::Realm,
    string::JsString, Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use crate::dom::element::with_element_data;
use boa_gc::{Finalize, Trace};

/// Internal data for DOMTokenList objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DOMTokenListData {
    /// The associated element object
    element: JsObject,
}

impl DOMTokenListData {
    pub fn new(element: JsObject) -> Self {
        Self { element }
    }

    fn class_name(&self, context: &mut Context) -> Option<String> {
        if let Ok(result) = with_element_data(&self.element, |ed| {
            ed.get_class_name()
        }, "not element") {
            Some(result)
        } else {
            None
        }
    }

    fn set_class_name(&self, value: String) {
        let _ = with_element_data(&self.element, |ed| {
            ed.set_class_name(value.clone());
            ed.set_attribute("class".to_string(), value);
        }, "not element");
    }
}

/// The `DOMTokenList` object
#[derive(Debug, Trace, Finalize)]
pub struct DOMTokenList;

impl DOMTokenList {
    fn split_tokens(class_name: &str) -> Vec<String> {
        class_name
            .split(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C'))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    fn join_tokens(tokens: &[String]) -> String {
        tokens.join(" ")
    }

    fn validate_token(token: &str) -> Result<(), JsNativeError> {
        if token.is_empty() {
            return Err(JsNativeError::typ().with_message("The token must not be empty"));
        }
        if token.chars().any(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')) {
            return Err(JsNativeError::typ().with_message("The token contains invalid whitespace"));
        }
        Ok(())
    }

    /// Helper: build or return a DOMTokenList object bound to an element
    pub fn create_for_element(element: JsObject, context: &mut Context) -> JsResult<JsObject> {
        let data = DOMTokenListData::new(element.clone());
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().domtokenlist().prototype(),
            data,
        );
        Ok(obj.upcast())
    }

    /* Prototype methods */
    fn contains(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.contains called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.contains called on non-DOMTokenList object")
        })?;

        let token = args.get_or_undefined(0).to_string(context)?;
        let token_std = token.to_std_string_escaped();
        Self::validate_token(&token_std)?;
        if let Some(class_name) = data.class_name(context) {
            let tokens = Self::split_tokens(&class_name);
            return Ok(JsValue::new(tokens.contains(&token_std)));
        }
        Ok(JsValue::new(false))
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.add called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.add called on non-DOMTokenList object")
        })?;

        let mut class = data.class_name(context).unwrap_or_default();
        let mut tokens = Self::split_tokens(&class);
        for arg in args.iter() {
            let token = arg.to_string(context)?;
            let token_std = token.to_std_string_escaped();
            Self::validate_token(&token_std)?;
            if !tokens.contains(&token_std) {
                tokens.push(token_std);
            }
        }
        class = Self::join_tokens(&tokens);
        data.set_class_name(class);
        Ok(JsValue::undefined())
    }

    fn remove(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.remove called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.remove called on non-DOMTokenList object")
        })?;

        let mut class = data.class_name(context).unwrap_or_default();
        let mut tokens = Self::split_tokens(&class);
        for arg in args.iter() {
            let token = arg.to_string(context)?;
            let token_std = token.to_std_string_escaped();
            Self::validate_token(&token_std)?;
            tokens.retain(|x| x != &token_std);
        }
        class = Self::join_tokens(&tokens);
        data.set_class_name(class);
        Ok(JsValue::undefined())
    }

    fn toggle(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.toggle called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.toggle called on non-DOMTokenList object")
        })?;

        let token = args.get_or_undefined(0).to_string(context)?;
        let token_std = token.to_std_string_escaped();
        Self::validate_token(&token_std)?;

        // Support optional 'force' parameter
        let force = if args.len() > 1 && !args[1].is_undefined() {
            Some(args[1].to_boolean())
        } else {
            None
        };

        let class = data.class_name(context).unwrap_or_default();
        let mut tokens = Self::split_tokens(&class);
        let has = tokens.contains(&token_std);

        let result = match force {
            Some(true) => {
                if !has { tokens.push(token_std); }
                true
            }
            Some(false) => {
                tokens.retain(|x| x != &token_std);
                false
            }
            None => {
                if has {
                    tokens.retain(|x| x != &token_std);
                    false
                } else {
                    tokens.push(token_std);
                    true
                }
            }
        };

        data.set_class_name(Self::join_tokens(&tokens));
        Ok(JsValue::new(result))
    }

    fn replace(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.replace called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.replace called on non-DOMTokenList object")
        })?;

        let old_token = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let new_token = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
        Self::validate_token(&old_token)?;
        Self::validate_token(&new_token)?;

        let class = data.class_name(context).unwrap_or_default();
        let mut tokens = Self::split_tokens(&class);
        let mut replaced = false;

        if let Some(pos) = tokens.iter().position(|t| t == &old_token) {
            if !tokens.contains(&new_token) {
                tokens[pos] = new_token;
            } else {
                tokens.remove(pos);
            }
            replaced = true;
        }

        data.set_class_name(Self::join_tokens(&tokens));
        Ok(JsValue::new(replaced))
    }

    fn get_value(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.value called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.value called on non-DOMTokenList object")
        })?;

        let class = data.class_name(context).unwrap_or_default();
        Ok(JsValue::from(JsString::from(class)))
    }

    fn set_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.value setter called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.value setter called on non-DOMTokenList object")
        })?;

        let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        data.set_class_name(value);
        Ok(JsValue::undefined())
    }

    fn to_string_method(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::get_value(this, _args, context)
    }

    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.item called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.item called on non-DOMTokenList object")
        })?;

        let index = args.get_or_undefined(0).to_length(context)? as usize;
        if let Some(class_name) = data.class_name(context) {
            let tokens = Self::split_tokens(&class_name);
            if let Some(t) = tokens.get(index) {
                return Ok(JsValue::from(JsString::from(t.clone())));
            }
        }
        Ok(JsValue::null())
    }

    fn get_length(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.length called on non-object"))?;

        let data = this_obj.downcast_ref::<DOMTokenListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("DOMTokenList.length called on non-DOMTokenList object")
        })?;

        if let Some(class_name) = data.class_name(context) {
            let tokens = Self::split_tokens(&class_name);
            return Ok(JsValue::new(tokens.len() as i32));
        }
        Ok(JsValue::new(0))
    }
}

impl IntrinsicObject for DOMTokenList {
    fn init(realm: &Realm) {
        let contains_func = BuiltInBuilder::callable(realm, Self::contains)
            .name(js_string!("contains"))
            .length(1)
            .build();

        let add_func = BuiltInBuilder::callable(realm, Self::add)
            .name(js_string!("add"))
            .length(1)
            .build();

        let remove_func = BuiltInBuilder::callable(realm, Self::remove)
            .name(js_string!("remove"))
            .length(1)
            .build();

        let toggle_func = BuiltInBuilder::callable(realm, Self::toggle)
            .name(js_string!("toggle"))
            .length(1)
            .build();

        let item_func = BuiltInBuilder::callable(realm, Self::item)
            .name(js_string!("item"))
            .length(1)
            .build();

        let length_getter = BuiltInBuilder::callable(realm, Self::get_length)
            .name(js_string!("get length"))
            .build();

        let value_getter = BuiltInBuilder::callable(realm, Self::get_value)
            .name(js_string!("get value"))
            .build();

        let value_setter = BuiltInBuilder::callable(realm, Self::set_value)
            .name(js_string!("set value"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::contains, js_string!("contains"), 1)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::remove, js_string!("remove"), 1)
            .method(Self::toggle, js_string!("toggle"), 1)
            .method(Self::replace, js_string!("replace"), 2)
            .method(Self::item, js_string!("item"), 1)
            .method(Self::to_string_method, js_string!("toString"), 0)
            .accessor(js_string!("length"), Some(length_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("value"), Some(value_getter), Some(value_setter), Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DOMTokenList {
    const NAME: JsString = js_string!("DOMTokenList");
}

impl BuiltInConstructor for DOMTokenList {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::domtokenlist;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // DOMTokenList is not intended to be directly constructed in most engines; return an empty object
        let obj = JsObject::default(_context.intrinsics());
        Ok(obj.into())
    }
}



#[cfg(test)]
mod tests;
