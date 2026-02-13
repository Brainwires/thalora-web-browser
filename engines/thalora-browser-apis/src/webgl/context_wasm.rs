//! WebGLRenderingContext stub for WASM builds
//!
//! In WASM builds, WebGL is accessed through the browser's native WebGL API via web-sys.
//! This module provides placeholder types for API compatibility.

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::{FunctionObjectBuilder, JsObject},
    realm::Realm,
    Context, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};

use super::constants::add_webgl_constants;

/// WebGL context data stub for WASM builds
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebGLRenderingContextData;

/// WebGLRenderingContext stub for WASM builds
#[derive(Debug, Copy, Clone)]
pub struct WebGLRenderingContext;

impl WebGLRenderingContext {
    /// Initialize WebGLRenderingContext in the realm
    pub fn init(realm: &Realm) {
        // Minimal initialization - actual WebGL is used via web-sys
    }

    /// Get the WebGLRenderingContext intrinsic object
    pub fn get(intrinsics: &Intrinsics) -> JsObject {
        JsObject::with_null_proto()
    }

    /// Create the global WebGLRenderingContext constructor with constants
    pub fn create_global_constructor(context: &mut Context) -> JsResult<JsObject> {
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            boa_engine::NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Err(JsNativeError::error()
                    .with_message("WebGLRenderingContext is not directly constructible in WASM. Use canvas.getContext('webgl').")
                    .into())
            }),
        )
        .name(js_string!("WebGLRenderingContext"))
        .length(0)
        .build();

        // Add WebGL constants to the constructor
        let constructor_obj: JsObject = constructor.into();
        add_webgl_constants(&constructor_obj, context);

        Ok(constructor_obj)
    }
}
