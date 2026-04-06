//! HTMLElement implementation for Boa
//!
//! Implements the HTMLElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/dom.html#htmlelement

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

/// JavaScript `HTMLElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLElement;

impl IntrinsicObject for HTMLElement {
    fn init(realm: &Realm) {
        let inner_text_getter = BuiltInBuilder::callable(realm, get_inner_text)
            .name(js_string!("get innerText"))
            .build();
        let inner_text_setter = BuiltInBuilder::callable(realm, set_inner_text)
            .name(js_string!("set innerText"))
            .build();

        let hidden_getter = BuiltInBuilder::callable(realm, get_hidden)
            .name(js_string!("get hidden"))
            .build();
        let hidden_setter = BuiltInBuilder::callable(realm, set_hidden)
            .name(js_string!("set hidden"))
            .build();

        let style_getter = BuiltInBuilder::callable(realm, get_style)
            .name(js_string!("get style"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("innerText"),
                Some(inner_text_getter),
                Some(inner_text_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hidden"),
                Some(hidden_getter),
                Some(hidden_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("style"),
                Some(style_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(click, js_string!("click"), 0)
            .method(focus, js_string!("focus"), 0)
            .method(blur, js_string!("blur"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLElement {
    const NAME: JsString = StaticJsStrings::HTML_ELEMENT;
}

impl BuiltInConstructor for HTMLElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_element,
            context,
        )?;
        let html_element_data = HTMLElementData::new();
        let html_element_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            html_element_data,
        );

        let html_element_generic = html_element_obj.upcast();

        // Set Element interface properties
        html_element_generic.set(js_string!("tagName"), js_string!("DIV"), false, context)?;
        html_element_generic.set(js_string!("nodeName"), js_string!("DIV"), false, context)?;
        html_element_generic.set(js_string!("nodeType"), 1, false, context)?; // ELEMENT_NODE

        Ok(html_element_generic.into())
    }
}

/// Internal data for HTMLElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLElementData {
    #[unsafe_ignore_trace]
    inner_text: String,
    #[unsafe_ignore_trace]
    hidden: bool,
}

impl Default for HTMLElementData {
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLElementData {
    pub fn new() -> Self {
        Self {
            inner_text: String::new(),
            hidden: false,
        }
    }
}

fn get_inner_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLElement.prototype.innerText called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLElementData>() {
        Ok(js_string!(data.inner_text.clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_inner_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLElement.prototype.innerText called on non-object")
    })?;

    let text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLElementData>() {
        data.inner_text = text;
    }

    Ok(JsValue::undefined())
}

fn get_hidden(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLElement.prototype.hidden called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLElementData>() {
        Ok(JsValue::from(data.hidden))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_hidden(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLElement.prototype.hidden called on non-object")
    })?;

    let hidden = args.get_or_undefined(0).to_boolean();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLElementData>() {
        data.hidden = hidden;
    }

    Ok(JsValue::undefined())
}

fn get_style(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a CSSStyleDeclaration object
    let style_constructor = context
        .intrinsics()
        .constructors()
        .css_style_declaration()
        .constructor();
    crate::browser::cssom::CSSStyleDeclaration::constructor(
        &style_constructor.clone().into(),
        &[],
        context,
    )
}

fn click(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, click is a no-op but valid
    Ok(JsValue::undefined())
}

fn focus(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, focus is a no-op but valid
    Ok(JsValue::undefined())
}

fn blur(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, blur is a no-op but valid
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
    fn test_html_element_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof HTMLElement === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_element_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const el = new HTMLElement();
            el.nodeType === 1;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
