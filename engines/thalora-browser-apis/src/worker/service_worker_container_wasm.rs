//! ServiceWorkerContainer stub for WASM builds
//!
//! In WASM builds, the browser's native ServiceWorkerContainer is used directly.

use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct ServiceWorkerContainerData;

#[derive(Debug, Copy, Clone)]
pub struct ServiceWorkerContainer;

impl ServiceWorkerContainer {
    pub fn create(_context: &mut Context) -> JsResult<JsObject> {
        Err(JsNativeError::error()
            .with_message("ServiceWorkerContainer is not available in WASM. Use the browser's native navigator.serviceWorker.")
            .into())
    }
}

impl IntrinsicObject for ServiceWorkerContainer {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                js_string!("ready"),
                JsValue::undefined(),
                Attribute::READONLY,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ServiceWorkerContainer {
    const NAME: JsString = js_string!("ServiceWorkerContainer");
}

impl BuiltInConstructor for ServiceWorkerContainer {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::service_worker_container;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("ServiceWorkerContainer is not available in WASM. Use the browser's native navigator.serviceWorker.")
            .into())
    }
}
