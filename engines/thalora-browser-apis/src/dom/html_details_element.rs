//! HTMLDetailsElement implementation for Boa
//!
//! Implements the HTMLDetailsElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element

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

/// JavaScript `HTMLDetailsElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLDetailsElement;

impl IntrinsicObject for HTMLDetailsElement {
    fn init(realm: &Realm) {
        let open_getter = BuiltInBuilder::callable(realm, get_open)
            .name(js_string!("get open"))
            .build();
        let open_setter = BuiltInBuilder::callable(realm, set_open)
            .name(js_string!("set open"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("open"),
                Some(open_getter),
                Some(open_setter),
                Attribute::CONFIGURABLE,
            )
            .method(toggle, js_string!("toggle"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLDetailsElement {
    const NAME: JsString = StaticJsStrings::HTML_DETAILS_ELEMENT;
}

impl BuiltInConstructor for HTMLDetailsElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_details_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLDetailsElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_details_element,
            context,
        )?;
        let details_data = HTMLDetailsElementData::new();
        let details_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            details_data,
        );

        let details_generic = details_obj.upcast();

        // Set Element interface properties
        details_generic.set(js_string!("tagName"), js_string!("DETAILS"), false, context)?;
        details_generic.set(
            js_string!("nodeName"),
            js_string!("DETAILS"),
            false,
            context,
        )?;
        details_generic.set(js_string!("nodeType"), 1, false, context)?; // ELEMENT_NODE

        Ok(details_generic.into())
    }
}

/// Internal data for HTMLDetailsElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLDetailsElementData {
    #[unsafe_ignore_trace]
    open: bool,
}

impl HTMLDetailsElementData {
    pub fn new() -> Self {
        Self { open: false }
    }
}

fn get_open(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDetailsElement.prototype.open called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLDetailsElementData>() {
        Ok(JsValue::from(data.open))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_open(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDetailsElement.prototype.open called on non-object")
    })?;

    let open = args.get_or_undefined(0).to_boolean();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDetailsElementData>() {
        data.open = open;
    }

    Ok(JsValue::undefined())
}

fn toggle(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("HTMLDetailsElement.prototype.toggle called on non-object")
    })?;

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDetailsElementData>() {
        data.open = !data.open;
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
    fn test_html_details_element_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                "typeof HTMLDetailsElement === 'function'",
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_details_element_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const details = new HTMLDetailsElement();
            details.tagName === 'DETAILS';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_details_element_open() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const details = new HTMLDetailsElement();
            details.open === false && (details.open = true, details.open === true);
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_details_element_toggle() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const details = new HTMLDetailsElement();
            details.toggle();
            const afterFirst = details.open;
            details.toggle();
            const afterSecond = details.open;
            afterFirst === true && afterSecond === false;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
