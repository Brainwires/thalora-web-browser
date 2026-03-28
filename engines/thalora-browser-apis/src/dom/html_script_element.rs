//! HTMLScriptElement implementation for Boa
//!
//! Implements the HTMLScriptElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/scripting.html#the-script-element

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `HTMLScriptElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLScriptElement;

impl IntrinsicObject for HTMLScriptElement {
    fn init(realm: &Realm) {
        let src_getter = BuiltInBuilder::callable(realm, get_src)
            .name(js_string!("get src"))
            .build();
        let src_setter = BuiltInBuilder::callable(realm, set_src)
            .name(js_string!("set src"))
            .build();

        let type_getter = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();
        let type_setter = BuiltInBuilder::callable(realm, set_type)
            .name(js_string!("set type"))
            .build();

        let async_getter = BuiltInBuilder::callable(realm, get_async)
            .name(js_string!("get async"))
            .build();
        let async_setter = BuiltInBuilder::callable(realm, set_async)
            .name(js_string!("set async"))
            .build();

        let defer_getter = BuiltInBuilder::callable(realm, get_defer)
            .name(js_string!("get defer"))
            .build();
        let defer_setter = BuiltInBuilder::callable(realm, set_defer)
            .name(js_string!("set defer"))
            .build();

        let text_getter = BuiltInBuilder::callable(realm, get_text)
            .name(js_string!("get text"))
            .build();
        let text_setter = BuiltInBuilder::callable(realm, set_text)
            .name(js_string!("set text"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("src"),
                Some(src_getter),
                Some(src_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_getter),
                Some(type_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("async"),
                Some(async_getter),
                Some(async_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("defer"),
                Some(defer_getter),
                Some(defer_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("text"),
                Some(text_getter),
                Some(text_setter),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLScriptElement {
    const NAME: JsString = StaticJsStrings::HTML_SCRIPT_ELEMENT;
}

impl BuiltInConstructor for HTMLScriptElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_script_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLScriptElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_script_element,
            context,
        )?;
        let script_data = HTMLScriptElementData::new();
        let script_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            script_data,
        );

        let script_generic = script_obj.upcast();

        // Set Element interface properties
        script_generic.set(js_string!("tagName"), js_string!("SCRIPT"), false, context)?;
        script_generic.set(js_string!("nodeName"), js_string!("SCRIPT"), false, context)?;
        script_generic.set(js_string!("nodeType"), 1, false, context)?;

        Ok(script_generic.into())
    }
}

/// Internal data for HTMLScriptElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLScriptElementData {
    #[unsafe_ignore_trace]
    src: String,
    #[unsafe_ignore_trace]
    type_: String,
    #[unsafe_ignore_trace]
    async_: bool,
    #[unsafe_ignore_trace]
    defer: bool,
    #[unsafe_ignore_trace]
    text: String,
}

impl HTMLScriptElementData {
    pub fn new() -> Self {
        Self {
            src: String::new(),
            type_: String::new(),
            async_: false,
            defer: false,
            text: String::new(),
        }
    }
}

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.src called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.src.clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.src called on non-object")
    })?;

    let src = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLScriptElementData>() {
        data.src = src;
    }

    Ok(JsValue::undefined())
}

fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.type called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.type_.clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_type(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.type called on non-object")
    })?;

    let type_ = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLScriptElementData>() {
        data.type_ = type_;
    }

    Ok(JsValue::undefined())
}

fn get_async(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.async called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(JsValue::from(data.async_))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_async(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.async called on non-object")
    })?;

    let async_ = args.get_or_undefined(0).to_boolean();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLScriptElementData>() {
        data.async_ = async_;
    }

    Ok(JsValue::undefined())
}

fn get_defer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.defer called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(JsValue::from(data.defer))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_defer(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.defer called on non-object")
    })?;

    let defer = args.get_or_undefined(0).to_boolean();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLScriptElementData>() {
        data.defer = defer;
    }

    Ok(JsValue::undefined())
}

fn get_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.text called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(js_string!(data.text.clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLScriptElement.prototype.text called on non-object")
    })?;

    let text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLScriptElementData>() {
        data.text = text;
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
    fn test_html_script_element_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                "typeof HTMLScriptElement === 'function'",
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_script_element_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const script = new HTMLScriptElement();
            script.tagName === 'SCRIPT';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
