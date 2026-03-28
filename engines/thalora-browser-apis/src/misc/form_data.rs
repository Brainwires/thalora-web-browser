//! FormData implementation for Boa
//!
//! Implements the FormData interface as defined in:
//! https://xhr.spec.whatwg.org/#interface-formdata

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `FormData` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct FormData;

impl IntrinsicObject for FormData {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(append, js_string!("append"), 2)
            .method(delete, js_string!("delete"), 1)
            .method(get, js_string!("get"), 1)
            .method(get_all, js_string!("getAll"), 1)
            .method(has, js_string!("has"), 1)
            .method(set, js_string!("set"), 2)
            .method(keys, js_string!("keys"), 0)
            .method(values, js_string!("values"), 0)
            .method(entries, js_string!("entries"), 0)
            .method(for_each, js_string!("forEach"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for FormData {
    const NAME: JsString = StaticJsStrings::FORM_DATA;
}

impl BuiltInConstructor for FormData {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::form_data;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("FormData constructor requires 'new'")
                .into());
        }

        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::form_data, context)?;
        let form_data = FormDataData::new();
        let form_data_obj =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), proto, form_data);

        Ok(form_data_obj.upcast().into())
    }
}

/// Internal data for FormData instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FormDataData {
    #[unsafe_ignore_trace]
    entries: HashMap<String, Vec<String>>,
}

impl FormDataData {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

fn append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.append called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let value = args
        .get_or_undefined(1)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<FormDataData>() {
        data.entries
            .entry(name)
            .or_insert_with(Vec::new)
            .push(value);
    }

    Ok(JsValue::undefined())
}

fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.delete called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<FormDataData>() {
        data.entries.remove(&name);
    }

    Ok(JsValue::undefined())
}

fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.get called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        if let Some(values) = data.entries.get(&name) {
            if let Some(first) = values.first() {
                return Ok(js_string!(first.clone()).into());
            }
        }
    }

    Ok(JsValue::null())
}

fn get_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.getAll called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        if let Some(values) = data.entries.get(&name) {
            let array = boa_engine::object::builtins::JsArray::new(context)?;
            for value in values {
                array.push(js_string!(value.clone()), context)?;
            }
            return Ok(array.into());
        }
    }

    Ok(boa_engine::object::builtins::JsArray::new(context)?.into())
}

fn has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.has called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        return Ok(JsValue::from(data.entries.contains_key(&name)));
    }

    Ok(JsValue::from(false))
}

fn set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.set called on non-object")
    })?;

    let name = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let value = args
        .get_or_undefined(1)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<FormDataData>() {
        data.entries.insert(name, vec![value]);
    }

    Ok(JsValue::undefined())
}

fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.keys called on non-object")
    })?;

    let array = boa_engine::object::builtins::JsArray::new(context)?;

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        for key in data.entries.keys() {
            array.push(js_string!(key.clone()), context)?;
        }
    }

    Ok(array.into())
}

fn values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.values called on non-object")
    })?;

    let array = boa_engine::object::builtins::JsArray::new(context)?;

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        for values in data.entries.values() {
            for value in values {
                array.push(js_string!(value.clone()), context)?;
            }
        }
    }

    Ok(array.into())
}

fn entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.entries called on non-object")
    })?;

    let array = boa_engine::object::builtins::JsArray::new(context)?;

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        for (key, values) in &data.entries {
            for value in values {
                let entry = boa_engine::object::builtins::JsArray::new(context)?;
                entry.push(js_string!(key.clone()), context)?;
                entry.push(js_string!(value.clone()), context)?;
                array.push(entry, context)?;
            }
        }
    }

    Ok(array.into())
}

fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("FormData.prototype.forEach called on non-object")
    })?;

    let callback = args.get_or_undefined(0);
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("FormData.prototype.forEach callback must be a function")
            .into());
    }

    if let Some(data) = this_obj.downcast_ref::<FormDataData>() {
        if let Some(func) = callback.as_callable() {
            for (key, values) in &data.entries {
                for value in values {
                    func.call(
                        this,
                        &[
                            js_string!(value.clone()).into(),
                            js_string!(key.clone()).into(),
                            this.clone(),
                        ],
                        context,
                    )?;
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_form_data_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof FormData === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_form_data_append_get() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const fd = new FormData();
            fd.append('key', 'value');
            fd.get('key') === 'value';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_form_data_has() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const fd = new FormData();
            fd.append('key', 'value');
            fd.has('key') === true && fd.has('missing') === false;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
