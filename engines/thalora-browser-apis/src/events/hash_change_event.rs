//! HashChangeEvent implementation for Boa
//!
//! Implements the HashChangeEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/browsing-the-web.html#hashchangeevent

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

/// JavaScript `HashChangeEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct HashChangeEvent;

impl IntrinsicObject for HashChangeEvent {
    fn init(realm: &Realm) {
        let old_url_getter = BuiltInBuilder::callable(realm, get_old_url)
            .name(js_string!("get oldURL"))
            .build();

        let new_url_getter = BuiltInBuilder::callable(realm, get_new_url)
            .name(js_string!("get newURL"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("oldURL"),
                Some(old_url_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("newURL"),
                Some(new_url_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HashChangeEvent {
    const NAME: JsString = StaticJsStrings::HASH_CHANGE_EVENT;
}

impl BuiltInConstructor for HashChangeEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::hash_change_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("HashChangeEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        let event_type = type_arg.to_string(context)?;

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::hash_change_event, context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut old_url = String::new();
        let mut new_url = String::new();

        if let Some(init_obj) = event_init_dict.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("oldURL"), context) {
                old_url = v.to_string(context)?.to_std_string_escaped();
            }
            if let Ok(v) = init_obj.get(js_string!("newURL"), context) {
                new_url = v.to_string(context)?.to_std_string_escaped();
            }
        }

        let event_data = EventData::new(event_type.to_std_string_escaped(), bubbles, cancelable);
        let hash_change_event_data = HashChangeEventData::new(event_data, old_url, new_url);

        let hash_change_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            hash_change_event_data,
        );

        Ok(hash_change_event_obj.into())
    }
}

/// Internal data for HashChangeEvent instances - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HashChangeEventData {
    /// Base event data
    pub event: EventData,
    #[unsafe_ignore_trace]
    old_url: String,
    #[unsafe_ignore_trace]
    new_url: String,
}

impl HashChangeEventData {
    pub fn new(event: EventData, old_url: String, new_url: String) -> Self {
        Self { event, old_url, new_url }
    }
}

fn get_old_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HashChangeEvent.prototype.oldURL called on non-object")
    })?;

    let hash_change_event = this_obj.downcast_ref::<HashChangeEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("HashChangeEvent.prototype.oldURL called on non-HashChangeEvent object")
    })?;

    Ok(js_string!(hash_change_event.old_url.clone()).into())
}

fn get_new_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("HashChangeEvent.prototype.newURL called on non-object")
    })?;

    let hash_change_event = this_obj.downcast_ref::<HashChangeEventData>().ok_or_else(|| {
        JsNativeError::typ().with_message("HashChangeEvent.prototype.newURL called on non-HashChangeEvent object")
    })?;

    Ok(js_string!(hash_change_event.new_url.clone()).into())
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
    fn test_hash_change_event_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof HashChangeEvent === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_hash_change_event_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const event = new HashChangeEvent('hashchange', { oldURL: 'http://example.com#old', newURL: 'http://example.com#new' });
            event.type === 'hashchange' && event.oldURL === 'http://example.com#old' && event.newURL === 'http://example.com#new';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
