//! RTCIceCandidate stub for WASM builds

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
pub struct RTCIceCandidateData;

#[derive(Debug, Copy, Clone)]
pub struct RTCIceCandidate;

impl IntrinsicObject for RTCIceCandidate {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RTCIceCandidate {
    const NAME: JsString = StaticJsStrings::RTC_ICE_CANDIDATE;
}

impl BuiltInConstructor for RTCIceCandidate {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::rtc_ice_candidate;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("RTCIceCandidate is not available in WASM.")
            .into())
    }
}
