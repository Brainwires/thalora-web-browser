//! EventSource stub for WASM builds
//!
//! In WASM builds, the browser's native EventSource is used directly.

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};

/// EventSource connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closed = 2,
}

impl From<ReadyState> for JsValue {
    fn from(state: ReadyState) -> Self {
        JsValue::from(state as u32)
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
pub struct EventSourceData;

#[derive(Debug, Copy, Clone)]
pub struct EventSource;

impl IntrinsicObject for EventSource {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                js_string!("CONNECTING"),
                ReadyState::Connecting,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("OPEN"),
                ReadyState::Open,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("CLOSED"),
                ReadyState::Closed,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for EventSource {
    const NAME: JsString = StaticJsStrings::EVENT_SOURCE;
}

impl BuiltInConstructor for EventSource {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::event_source;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("EventSource is not available in WASM. Use the browser's native EventSource.")
            .into())
    }
}
