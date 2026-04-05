//! HTMLDialogElement implementation for Boa
//!
//! Implements the HTMLDialogElement interface as defined in:
//! https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element

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

/// JavaScript `HTMLDialogElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLDialogElement;

impl IntrinsicObject for HTMLDialogElement {
    fn init(realm: &Realm) {
        let open_getter = BuiltInBuilder::callable(realm, get_open)
            .name(js_string!("get open"))
            .build();
        let open_setter = BuiltInBuilder::callable(realm, set_open)
            .name(js_string!("set open"))
            .build();

        let return_value_getter = BuiltInBuilder::callable(realm, get_return_value)
            .name(js_string!("get returnValue"))
            .build();
        let return_value_setter = BuiltInBuilder::callable(realm, set_return_value)
            .name(js_string!("set returnValue"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("open"),
                Some(open_getter),
                Some(open_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("returnValue"),
                Some(return_value_getter),
                Some(return_value_setter),
                Attribute::CONFIGURABLE,
            )
            .method(show, js_string!("show"), 0)
            .method(show_modal, js_string!("showModal"), 0)
            .method(close, js_string!("close"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLDialogElement {
    const NAME: JsString = StaticJsStrings::HTML_DIALOG_ELEMENT;
}

impl BuiltInConstructor for HTMLDialogElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_dialog_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HTMLDialogElement constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_dialog_element,
            context,
        )?;
        let dialog_data = HTMLDialogElementData::new();
        let dialog_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            dialog_data,
        );

        let dialog_generic = dialog_obj.upcast();

        // Set Element interface properties
        dialog_generic.set(js_string!("tagName"), js_string!("DIALOG"), false, context)?;
        dialog_generic.set(js_string!("nodeName"), js_string!("DIALOG"), false, context)?;
        dialog_generic.set(js_string!("nodeType"), 1, false, context)?; // ELEMENT_NODE

        Ok(dialog_generic.into())
    }
}

/// Internal data for HTMLDialogElement instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLDialogElementData {
    #[unsafe_ignore_trace]
    open: bool,
    #[unsafe_ignore_trace]
    is_modal: bool,
    #[unsafe_ignore_trace]
    return_value: String,
}

impl HTMLDialogElementData {
    pub fn new() -> Self {
        Self {
            open: false,
            is_modal: false,
            return_value: String::new(),
        }
    }
}

fn get_open(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDialogElement.prototype.open called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLDialogElementData>() {
        Ok(JsValue::from(data.open))
    } else {
        Ok(JsValue::from(false))
    }
}

fn set_open(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDialogElement.prototype.open called on non-object")
    })?;

    let open = args.get_or_undefined(0).to_boolean();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDialogElementData>() {
        data.open = open;
    }

    Ok(JsValue::undefined())
}

fn get_return_value(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("HTMLDialogElement.prototype.returnValue called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<HTMLDialogElementData>() {
        Ok(js_string!(data.return_value.clone()).into())
    } else {
        Ok(js_string!("").into())
    }
}

fn set_return_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("HTMLDialogElement.prototype.returnValue called on non-object")
    })?;

    let return_value = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDialogElementData>() {
        data.return_value = return_value;
    }

    Ok(JsValue::undefined())
}

fn show(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDialogElement.prototype.show called on non-object")
    })?;

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDialogElementData>() {
        data.open = true;
        data.is_modal = false;
    }

    Ok(JsValue::undefined())
}

fn show_modal(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("HTMLDialogElement.prototype.showModal called on non-object")
    })?;

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDialogElementData>() {
        data.open = true;
        data.is_modal = true;
    }

    Ok(JsValue::undefined())
}

fn close(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HTMLDialogElement.prototype.close called on non-object")
    })?;

    if let Some(mut data) = this_obj.downcast_mut::<HTMLDialogElementData>() {
        data.open = false;

        // Update return value if provided
        let arg = args.get_or_undefined(0);
        if !arg.is_undefined() {
            data.return_value = arg.to_string(context)?.to_std_string_escaped();
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
    fn test_html_dialog_element_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                "typeof HTMLDialogElement === 'function'",
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_dialog_element_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const dialog = new HTMLDialogElement();
            dialog.tagName === 'DIALOG';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_dialog_element_show() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const dialog = new HTMLDialogElement();
            dialog.open === false && (dialog.show(), dialog.open === true);
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_dialog_element_show_modal() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const dialog = new HTMLDialogElement();
            dialog.showModal();
            dialog.open === true;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_dialog_element_close() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const dialog = new HTMLDialogElement();
            dialog.show();
            dialog.close('result');
            dialog.open === false && dialog.returnValue === 'result';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_html_dialog_element_return_value() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const dialog = new HTMLDialogElement();
            dialog.returnValue = 'hello';
            dialog.returnValue === 'hello';
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
