//! Fetch API stub for WASM builds
//!
//! In WASM builds, the browser's native fetch() is used directly.

use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// Fetch intrinsic stub for WASM
#[derive(Debug, Copy, Clone)]
pub struct Fetch;

impl IntrinsicObject for Fetch {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, fetch)
            .name(js_string!("fetch"))
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().fetch().into()
    }
}

impl BuiltInObject for Fetch {
    const NAME: JsString = js_string!("fetch");
}

fn fetch(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Err(JsNativeError::error()
        .with_message("fetch() is not available in WASM. Use the browser's native fetch API.")
        .into())
}

/// Request stub for WASM
#[derive(Debug, Copy, Clone)]
pub struct Request;

impl IntrinsicObject for Request {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Request {
    const NAME: JsString = js_string!("Request");
}

impl BuiltInConstructor for Request {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::request;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("Request is not available in WASM. Use the browser's native Request.")
            .into())
    }
}

/// Response stub for WASM
#[derive(Debug, Copy, Clone)]
pub struct Response;

impl IntrinsicObject for Response {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Response {
    const NAME: JsString = js_string!("Response");
}

impl BuiltInConstructor for Response {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::response;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("Response is not available in WASM. Use the browser's native Response.")
            .into())
    }
}

/// Headers stub for WASM
#[derive(Debug, Copy, Clone)]
pub struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Headers {
    const NAME: JsString = js_string!("Headers");
}

impl BuiltInConstructor for Headers {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::headers;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("Headers is not available in WASM. Use the browser's native Headers.")
            .into())
    }
}
