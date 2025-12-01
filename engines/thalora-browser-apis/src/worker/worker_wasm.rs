//! Worker stub for WASM builds
//!
//! In WASM builds, the browser's native Worker is used directly.

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
pub struct WorkerData;

#[derive(Debug, Copy, Clone)]
pub struct Worker;

impl IntrinsicObject for Worker {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Worker {
    const NAME: JsString = StaticJsStrings::WORKER;
}

impl BuiltInConstructor for Worker {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::worker;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("Worker is not available in WASM. Use the browser's native Worker API.")
            .into())
    }
}
