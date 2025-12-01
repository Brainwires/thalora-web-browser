//! XMLHttpRequest stub for WASM builds
//!
//! In WASM builds, the browser's native XMLHttpRequest is used directly.

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct XmlHttpRequestData;

#[derive(Debug, Copy, Clone)]
pub struct XmlHttpRequest;

impl IntrinsicObject for XmlHttpRequest {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(js_string!("UNSENT"), 0, Attribute::all())
            .static_property(js_string!("OPENED"), 1, Attribute::all())
            .static_property(js_string!("HEADERS_RECEIVED"), 2, Attribute::all())
            .static_property(js_string!("LOADING"), 3, Attribute::all())
            .static_property(js_string!("DONE"), 4, Attribute::all())
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for XmlHttpRequest {
    const NAME: JsString = js_string!("XMLHttpRequest");
}

impl BuiltInConstructor for XmlHttpRequest {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::xmlhttprequest;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("XMLHttpRequest is not available in WASM. Use the browser's native XMLHttpRequest.")
            .into())
    }
}
