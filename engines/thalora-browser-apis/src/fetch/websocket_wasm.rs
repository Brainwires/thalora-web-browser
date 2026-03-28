//! WebSocket Web API stub for WASM builds
//!
//! In WASM builds, the browser's native WebSocket is used directly through JavaScript.
//! This module provides placeholder types for API compatibility.

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `WebSocket` builtin implementation for WASM.
///
/// In WASM builds, WebSocket functionality is delegated to the browser's
/// native WebSocket API via web-sys, so this is a minimal stub.
#[derive(Debug, Copy, Clone)]
pub(crate) struct WebSocket;

impl IntrinsicObject for WebSocket {
    fn init(realm: &Realm) {
        // Minimal WebSocket constructor that throws - actual WebSocket
        // should be used from JavaScript in WASM builds
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                js_string!("CONNECTING"),
                JsValue::from(0),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("OPEN"),
                JsValue::from(1),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("CLOSING"),
                JsValue::from(2),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("CLOSED"),
                JsValue::from(3),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebSocket {
    const NAME: JsString = StaticJsStrings::WEBSOCKET;
}

impl BuiltInConstructor for WebSocket {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::websocket;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // In WASM builds, WebSocket should be used directly from JavaScript
        Err(JsNativeError::error()
            .with_message(
                "WebSocket is not available in WASM. Use the browser's native WebSocket API.",
            )
            .into())
    }
}

/// WebSocket data placeholder for WASM builds
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebSocketData;
