//! ServiceWorker stub for WASM builds
//!
//! In WASM builds, the browser's native ServiceWorker is used directly.

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

#[derive(Debug, Trace, Finalize, JsData)]
pub struct ServiceWorkerData;

#[derive(Debug, Copy, Clone)]
pub struct ServiceWorker;

impl IntrinsicObject for ServiceWorker {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ServiceWorker {
    const NAME: JsString = StaticJsStrings::SERVICE_WORKER;
}

impl BuiltInConstructor for ServiceWorker {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::service_worker;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message(
                "ServiceWorker is not available in WASM. Use the browser's native ServiceWorker.",
            )
            .into())
    }
}
