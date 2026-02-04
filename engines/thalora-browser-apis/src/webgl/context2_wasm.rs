//! WebGL2RenderingContext stub for WASM builds
//!
//! In WASM builds, WebGL2 is accessed through the browser's native WebGL2 API via web-sys.
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
use super::constants2::add_webgl2_constants;

/// WebGL2 context data stub for WASM builds
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebGL2RenderingContextData;

/// WebGL2RenderingContext stub for WASM builds
#[derive(Debug, Copy, Clone)]
pub struct WebGL2RenderingContext;

impl WebGL2RenderingContext {
    /// Initialize WebGL2RenderingContext in the realm
    pub fn init(realm: &Realm) {
        // Minimal initialization - actual WebGL2 is used via web-sys
    }

    /// Get the WebGL2RenderingContext intrinsic object
    pub fn get(intrinsics: &Intrinsics) -> JsObject {
        JsObject::with_null_proto()
    }

    /// Create the global WebGL2RenderingContext constructor with constants
    pub fn create_global_constructor(context: &mut Context) -> JsResult<JsObject> {
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            boa_engine::NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
                Err(JsNativeError::error()
                    .with_message("WebGL2RenderingContext is not directly constructible in WASM. Use canvas.getContext('webgl2').")
                    .into())
            }),
        )
        .name(js_string!("WebGL2RenderingContext"))
        .length(0)
        .build();

        // Add WebGL and WebGL2 constants to the constructor
        let constructor_obj: JsObject = constructor.into();
        add_webgl_constants(&constructor_obj, context);
        add_webgl2_constants(&constructor_obj, context);

        Ok(constructor_obj)
    }

    /// Create a new WebGL2 context for a canvas (WASM stub)
    ///
    /// In WASM builds, this returns a stub object. The actual WebGL2 context
    /// is obtained via web-sys in JavaScript land.
    pub fn create_context(_width: u32, _height: u32, context: &mut Context) -> JsResult<JsObject> {
        // Create a stub object with WebGL and WebGL2 constants for API compatibility
        let obj = JsObject::with_null_proto();
        add_webgl_constants(&obj, context);
        add_webgl2_constants(&obj, context);

        // Add drawing buffer properties
        obj.set(js_string!("drawingBufferWidth"), JsValue::from(_width), false, context)?;
        obj.set(js_string!("drawingBufferHeight"), JsValue::from(_height), false, context)?;
        obj.set(js_string!("canvas"), JsValue::null(), false, context)?;

        Ok(obj)
    }
}
