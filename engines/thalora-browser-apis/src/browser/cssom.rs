//! CSS Object Model (CSSOM) APIs for Boa
//!
//! Implements CSSStyleDeclaration and getComputedStyle

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, FunctionObjectBuilder},
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string, NativeFunction,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `CSSStyleDeclaration` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct CSSStyleDeclaration;

impl IntrinsicObject for CSSStyleDeclaration {
    fn init(realm: &Realm) {
        let length_getter = BuiltInBuilder::callable(realm, get_length)
            .name(js_string!("get length"))
            .build();

        let css_text_getter = BuiltInBuilder::callable(realm, get_css_text)
            .name(js_string!("get cssText"))
            .build();
        let css_text_setter = BuiltInBuilder::callable(realm, set_css_text)
            .name(js_string!("set cssText"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cssText"),
                Some(css_text_getter),
                Some(css_text_setter),
                boa_engine::property::Attribute::CONFIGURABLE,
            )
            .method(get_property_value, js_string!("getPropertyValue"), 1)
            .method(set_property, js_string!("setProperty"), 2)
            .method(remove_property, js_string!("removeProperty"), 1)
            .method(get_property_priority, js_string!("getPropertyPriority"), 1)
            .method(item, js_string!("item"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CSSStyleDeclaration {
    const NAME: JsString = StaticJsStrings::CSS_STYLE_DECLARATION;
}

impl BuiltInConstructor for CSSStyleDeclaration {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::css_style_declaration;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CSSStyleDeclaration constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::css_style_declaration, context)?;
        let style_data = CSSStyleDeclarationData::new();
        let style_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            style_data,
        );

        Ok(style_obj.upcast().into())
    }
}

/// Internal data for CSSStyleDeclaration instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CSSStyleDeclarationData {
    #[unsafe_ignore_trace]
    properties: HashMap<String, (String, bool)>, // (value, important)
}

impl CSSStyleDeclarationData {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
}

fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.length called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        return Ok(JsValue::from(data.properties.len() as u32));
    }

    Ok(JsValue::from(0))
}

fn get_css_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.cssText called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        let css_text: Vec<String> = data.properties.iter()
            .map(|(name, (value, important))| {
                if *important {
                    format!("{}: {} !important", name, value)
                } else {
                    format!("{}: {}", name, value)
                }
            })
            .collect();
        return Ok(js_string!(css_text.join("; ")).into());
    }

    Ok(js_string!("").into())
}

fn set_css_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.cssText called on non-object")
    })?;

    let css_text = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>() {
        data.properties.clear();
        // Parse simple CSS text (property: value; ...)
        for declaration in css_text.split(';') {
            let parts: Vec<&str> = declaration.splitn(2, ':').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let mut value = parts[1].trim().to_string();
                let important = value.ends_with("!important");
                if important {
                    value = value.replace("!important", "").trim().to_string();
                }
                data.properties.insert(name, (value, important));
            }
        }
    }

    Ok(JsValue::undefined())
}

fn get_property_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.getPropertyValue called on non-object")
    })?;

    let property = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        if let Some((value, _)) = data.properties.get(&property) {
            return Ok(js_string!(value.clone()).into());
        }
    }

    Ok(js_string!("").into())
}

fn set_property(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.setProperty called on non-object")
    })?;

    let property = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
    let priority = args.get(2)
        .map(|p| p.to_string(context).ok())
        .flatten()
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();

    let important = priority.eq_ignore_ascii_case("important");

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>() {
        if value.is_empty() {
            data.properties.remove(&property);
        } else {
            data.properties.insert(property, (value, important));
        }
    }

    Ok(JsValue::undefined())
}

fn remove_property(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.removeProperty called on non-object")
    })?;

    let property = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<CSSStyleDeclarationData>() {
        if let Some((old_value, _)) = data.properties.remove(&property) {
            return Ok(js_string!(old_value).into());
        }
    }

    Ok(js_string!("").into())
}

fn get_property_priority(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.getPropertyPriority called on non-object")
    })?;

    let property = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        if let Some((_, important)) = data.properties.get(&property) {
            if *important {
                return Ok(js_string!("important").into());
            }
        }
    }

    Ok(js_string!("").into())
}

fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CSSStyleDeclaration.prototype.item called on non-object")
    })?;

    let index = args.get_or_undefined(0).to_u32(context)? as usize;

    if let Some(data) = this_obj.downcast_ref::<CSSStyleDeclarationData>() {
        if let Some(name) = data.properties.keys().nth(index) {
            return Ok(js_string!(name.clone()).into());
        }
    }

    Ok(js_string!("").into())
}

/// Create the getComputedStyle global function
pub fn create_get_computed_style_function(context: &mut Context) -> JsResult<JsValue> {
    let func = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(get_computed_style),
    )
    .name(js_string!("getComputedStyle"))
    .length(1)
    .build();

    Ok(func.into())
}

/// `getComputedStyle(element, pseudoElt)`
fn get_computed_style(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a new CSSStyleDeclaration
    let style_constructor = context.intrinsics().constructors().css_style_declaration().constructor();
    CSSStyleDeclaration::constructor(
        &style_constructor.clone().into(),
        &[],
        context,
    )
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
    fn test_css_style_declaration_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof CSSStyleDeclaration === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_get_computed_style_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof getComputedStyle === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_css_style_declaration_set_property() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const style = new CSSStyleDeclaration();
            style.setProperty('color', 'red');
            style.getPropertyValue('color') === 'red';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
