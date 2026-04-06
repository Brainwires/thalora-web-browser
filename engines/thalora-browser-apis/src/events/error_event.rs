//! ErrorEvent implementation for Boa
//!
//! Implements the ErrorEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/webappapis.html#errorevent

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
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

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

        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::error_event, context)?;

        // Get properties from eventInitDict
        let (message, filename, lineno, colno, error) = if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                let message = init_obj
                    .get(js_string!("message"), context)
                    .ok()
                    .and_then(|v| v.to_string(context).ok())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                let filename = init_obj
                    .get(js_string!("filename"), context)
                    .ok()
                    .and_then(|v| v.to_string(context).ok())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                let lineno = init_obj
                    .get(js_string!("lineno"), context)
                    .ok()
                    .and_then(|v| v.to_u32(context).ok())
                    .unwrap_or(0);
                let colno = init_obj
                    .get(js_string!("colno"), context)
                    .ok()
                    .and_then(|v| v.to_u32(context).ok())
                    .unwrap_or(0);
                let error = init_obj
                    .get(js_string!("error"), context)
                    .unwrap_or(JsValue::undefined());
                (message, filename, lineno, colno, error)
            } else {
                (String::new(), String::new(), 0, 0, JsValue::undefined())
            }
        } else {
            (String::new(), String::new(), 0, 0, JsValue::undefined())
        };

        let error_event_data = ErrorEventData::new(
            event_type.to_std_string_escaped(),
            message,
            filename,
            lineno,
            colno,
            error,
        );
        let error_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            error_event_data,
        );

        let error_event_generic = error_event_obj.upcast();

        // Set Event interface properties
        error_event_generic.set(js_string!("type"), event_type, false, context)?;
        error_event_generic.set(js_string!("bubbles"), false, false, context)?;
        error_event_generic.set(js_string!("cancelable"), false, false, context)?;
        error_event_generic.set(js_string!("composed"), false, false, context)?;
        error_event_generic.set(js_string!("defaultPrevented"), false, false, context)?;
        error_event_generic.set(js_string!("eventPhase"), 0, false, context)?;
        error_event_generic.set(js_string!("isTrusted"), false, false, context)?;
        error_event_generic.set(js_string!("target"), JsValue::null(), false, context)?;
        error_event_generic.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        error_event_generic.set(
            js_string!("timeStamp"),
            context.clock().now().millis_since_epoch(),
            false,
            context,
        )?;

        // Parse eventInitDict for Event properties
        if !event_init_dict.is_undefined()
            && let Some(init_obj) = event_init_dict.as_object()
        {
            if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                error_event_generic.set(
                    js_string!("bubbles"),
                    bubbles_val.to_boolean(),
                    false,
                    context,
                )?;
            }
            if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                error_event_generic.set(
                    js_string!("cancelable"),
                    cancelable_val.to_boolean(),
                    false,
                    context,
                )?;
            }
        }

        Ok(error_event_generic.into())
    }
}

/// Internal data for ErrorEvent instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ErrorEventData {
    #[unsafe_ignore_trace]
    event_type: String,
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
    fn new(
        event_type: String,
        message: String,
        filename: String,
        lineno: u32,
        colno: u32,
        error: JsValue,
    ) -> Self {
        Self {
            event_type,
            message,
            filename,
            lineno,
            colno,
            error,
        }
    }
}

fn get_message(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.message called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ErrorEvent.prototype.message called on non-ErrorEvent object")
    })?;

    Ok(js_string!(error_event.message.clone()).into())
}

fn get_filename(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.filename called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ErrorEvent.prototype.filename called on non-ErrorEvent object")
    })?;

    Ok(js_string!(error_event.filename.clone()).into())
}

fn get_lineno(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.lineno called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ErrorEvent.prototype.lineno called on non-ErrorEvent object")
    })?;

    Ok(JsValue::from(error_event.lineno))
}

fn get_colno(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.colno called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ErrorEvent.prototype.colno called on non-ErrorEvent object")
    })?;

    Ok(JsValue::from(error_event.colno))
}

fn get_error(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ErrorEvent.prototype.error called on non-object")
    })?;

    let error_event = this_obj.downcast_ref::<ErrorEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("ErrorEvent.prototype.error called on non-ErrorEvent object")
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
        let result = context
            .eval(Source::from_bytes("typeof ErrorEvent === 'function'"))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_error_event_constructor() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            const event = new ErrorEvent('error', { message: 'Test error', lineno: 10 });
            event.type === 'error' && event.message === 'Test error' && event.lineno === 10;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
