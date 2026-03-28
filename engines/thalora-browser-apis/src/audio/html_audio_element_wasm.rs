//! HTMLAudioElement stub for WASM builds
//!
//! In WASM builds, audio playback is handled by the browser's native HTMLAudioElement.
//! This module provides placeholder types for API compatibility.

use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};

/// HTMLAudioElement data stub for WASM builds
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLAudioElementData;

/// HTMLAudioElement stub for WASM builds
#[derive(Debug, Copy, Clone)]
pub struct HTMLAudioElement;

impl IntrinsicObject for HTMLAudioElement {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLAudioElement {
    const NAME: JsString = StaticJsStrings::HTML_AUDIO_ELEMENT;
}

impl BuiltInConstructor for HTMLAudioElement {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_audio_element;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // In WASM builds, HTMLAudioElement should be used directly from the DOM
        Err(JsNativeError::error()
            .with_message(
                "HTMLAudioElement is not available in WASM. Use the browser's native Audio API.",
            )
            .into())
    }
}
