//! ErrorEvent implementation for Boa
//!
//! Implements the ErrorEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/webappapis.html#errorevent

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};

use super::event::EventData;

/// JavaScript `ErrorEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ErrorEvent;

impl IntrinsicObject for ErrorEvent {
    fn init(realm: &Realm) {
        let message_getter = BuiltInBuilder::callable(realm, get_message)
            .name(js_string!("get message"))
            .build();

        let filename_getter = BuiltInBuilder::callable(realm, get_filename)
            .name(js_string!("get filename"))
            .build();

        let lineno_getter = BuiltInBuilder::callable(realm, get_lineno)
            .name(js_string!("get lineno"))
            .build();

        let colno_getter = BuiltInBuilder::callable(realm, get_colno)
            .name(js_string!("get colno"))
            .build();

        let error_getter = BuiltInBuilder::callable(realm, get_error)
            .name(js_string!("get error"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("message"),
                Some(message_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("filename"),
                Some(filename_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lineno"),
                Some(lineno_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("colno"),
                Some(colno_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("error"),
                Some(error_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ErrorEvent {
    const NAME: JsString = StaticJsStrings::ERROR_EVENT;
}

impl BuiltInConstructor for ErrorEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 10;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::error_event;

    /// `new ErrorEvent(type, eventInitDict)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ErrorEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::error_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut message = String::new();
        let mut filename = String::new();
        let mut lineno = 0u32;
        let mut colno = 0u32;
        let mut error = JsValue::undefined();

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("message"), context) {
                message = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("filename"), context) {
                filename = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("lineno"), context) {
                lineno = v.to_u32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("colno"), context) {
                colno = v.to_u32(context)?;
            }
            if let Ok(v) = init_obj.get(js_string!("error"), context) {
                error = v;
            }
        }

        let event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        let error_event_data = ErrorEventData::new(event_data, message, filename, lineno, colno, error);

        let error_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            error_event_data,
        );

        Ok(error_event_obj.into())
    }
}

/// Internal data for ErrorEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ErrorEventData {
    /// Base event data
    pub event: EventData,
    #[unsafe_ignore_trace]
    message: String,
    #[unsafe_ignore_trace]
    filename: String,
    #[unsafe_ignore_trace]
    lineno: u32,
    #[unsafe_ignore_trace]
    colno: u32,
    error: JsValue,
}

impl ErrorEventData {
    pub fn new(event: EventData, message: String, filename: String, lineno: u32, colno: u32, error: JsValue) -> Self {
        Self { event, message, filename, lineno, colno, error }
    }
}

fn get_message(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.message called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.message called on non-ErrorEvent object")
    })?;

    Ok(js_string!(error_event.message.clone()).into())
}

fn get_filename(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.filename called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.filename called on non-ErrorEvent object")
    })?;

    Ok(js_string!(error_event.filename.clone()).into())
}

fn get_lineno(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.lineno called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.lineno called on non-ErrorEvent object")
    })?;

    Ok(JsValue::from(error_event.lineno))
}

fn get_colno(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.colno called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.colno called on non-ErrorEvent object")
    })?;

    Ok(JsValue::from(error_event.colno))
}

fn get_error(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.error called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.error called on non-ErrorEvent object")
    })?;

    Ok(error_event.error.clone())
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
    fn test_error_event_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof ErrorEvent === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_error_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new ErrorEvent('error', { message: 'Test error', lineno: 10 });
            event.type === 'error' && event.message === 'Test error' && event.lineno === 10;
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
