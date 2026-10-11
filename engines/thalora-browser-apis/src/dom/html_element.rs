//! HTMLElement implementation for Boa
//!
//! Implements the HTMLElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/dom.html#htmlelement

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};

use crate::dom::element::ElementData;

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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // HTMLElement.prototype -> Element.prototype, as in browsers
            .inherits(Some(
                realm.intrinsics().constructors().element().prototype(),
            ))
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
            // style/click/focus/blur come from Element.prototype
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

        // super() during a custom element upgrade: hand back the element
        // being upgraded
        if let Some(element) =
            crate::web_components::custom_element_registry::take_constructing_element()
        {
            return Ok(element.into());
        }

        // `new MyElement()`: the element is named after its definition
        let name = new_target.as_object().and_then(|ctor| {
            crate::web_components::custom_element_registry::name_for_constructor(&ctor, context)
        });
        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_element,
            context,
        )?;
        let tag = name.clone().unwrap_or_else(|| "div".to_string());
        // A real element, so everything on Element.prototype works on it
        // (and on custom elements whose class extends HTMLElement)
        let element: JsValue = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ElementData::with_tag_name(tag.to_uppercase()),
        )
        .upcast()
        .into();
        // Tree-backed (detached) when the page has a DOM tree
        let document = context
            .global_object()
            .get(js_string!("document"), context)?;
        if let Some((document, tree)) = crate::dom::binding::document_tree(&document) {
            crate::dom::binding::bind_new_element(&document, &tree, &element, &tag);
        }
        Ok(element)
    }
}

fn element_data(this: &JsValue, what: &str) -> JsResult<JsObject> {
    this.as_object()
        .filter(|obj| obj.downcast_ref::<ElementData>().is_some())
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message(format!(
                    "HTMLElement.prototype.{what} called on non-element"
                ))
                .into()
        })
}

/// `HTMLElement.prototype.innerText` getter (approximated by textContent)
fn get_inner_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = element_data(this, "innerText")?;
    let text = obj
        .downcast_ref::<ElementData>()
        .map(|e| e.get_text_content())
        .unwrap_or_default();
    Ok(js_string!(text).into())
}

/// `HTMLElement.prototype.innerText` setter
fn set_inner_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = element_data(this, "innerText")?;
    let text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    if let Some(element) = obj.downcast_ref::<ElementData>() {
        element.set_text_content(text);
    }
    Ok(JsValue::undefined())
}

/// `HTMLElement.prototype.hidden` getter (reflects the `hidden` attribute)
fn get_hidden(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = element_data(this, "hidden")?;
    let hidden = obj
        .downcast_ref::<ElementData>()
        .is_some_and(|e| e.has_attribute("hidden"));
    Ok(hidden.into())
}

/// `HTMLElement.prototype.hidden` setter
fn set_hidden(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = element_data(this, "hidden")?;
    let hidden = args.get_or_undefined(0).to_boolean();
    if let Some(element) = obj.downcast_ref::<ElementData>() {
        if hidden {
            element.set_attribute("hidden".to_string(), String::new());
        } else {
            element.remove_attribute("hidden");
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
