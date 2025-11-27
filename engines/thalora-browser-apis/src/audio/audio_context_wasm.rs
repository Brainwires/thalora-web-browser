//! AudioContext stub for WASM builds
//!
//! In WASM builds, Web Audio API is handled by the browser's native AudioContext.
//! This module provides placeholder types for API compatibility.

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};

/// AudioContext data stub for WASM builds
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AudioContextData;

/// AudioContext stub for WASM builds
#[derive(Debug, Copy, Clone)]
pub struct AudioContext;

impl IntrinsicObject for AudioContext {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AudioContext {
    const NAME: JsString = StaticJsStrings::AUDIO_CONTEXT;
}

impl BuiltInConstructor for AudioContext {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::audio_context;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // In WASM builds, AudioContext should be used directly from the browser
        Err(JsNativeError::error()
            .with_message("AudioContext is not available in WASM. Use the browser's native Web Audio API.")
            .into())
    }
}
