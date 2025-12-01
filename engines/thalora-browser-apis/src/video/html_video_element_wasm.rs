//! HTMLVideoElement stub for WASM builds
//!
//! In WASM builds, the browser's native HTMLVideoElement is used directly.

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLVideoElementData;

#[derive(Debug, Copy, Clone)]
pub struct HTMLVideoElement;

impl IntrinsicObject for HTMLVideoElement {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLVideoElement {
    const NAME: JsString = StaticJsStrings::HTML_VIDEO_ELEMENT;
}

impl BuiltInConstructor for HTMLVideoElement {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_video_element;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("HTMLVideoElement is not available in WASM. Use the browser's native HTMLVideoElement.")
            .into())
    }
}
