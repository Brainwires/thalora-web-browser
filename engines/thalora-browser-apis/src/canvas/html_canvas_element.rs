//! HTMLCanvasElement Web API implementation
//!
//! The HTMLCanvasElement is the JavaScript interface for the <canvas> element.
//! https://html.spec.whatwg.org/multipage/canvas.html#htmlcanvaselement

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute, Source,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

use super::canvas_state::CanvasState;
use super::rendering_context_2d::CanvasRenderingContext2DData;

/// Internal data for HTMLCanvasElement
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLCanvasElementData {
    /// Canvas width
    #[unsafe_ignore_trace]
    width: Arc<Mutex<u32>>,
    /// Canvas height
    #[unsafe_ignore_trace]
    height: Arc<Mutex<u32>>,
    /// The canvas state (shared with 2D context)
    #[unsafe_ignore_trace]
    state: Arc<Mutex<Option<CanvasState>>>,
    /// The 2D rendering context (cached)
    #[unsafe_ignore_trace]
    context_2d: Arc<Mutex<Option<JsObject>>>,
}

impl HTMLCanvasElementData {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: Arc::new(Mutex::new(width)),
            height: Arc::new(Mutex::new(height)),
            state: Arc::new(Mutex::new(CanvasState::new(width, height))),
            context_2d: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_width(&self) -> u32 {
        *self.width.lock().unwrap()
    }

    pub fn get_height(&self) -> u32 {
        *self.height.lock().unwrap()
    }

    pub fn set_width(&self, width: u32) {
        *self.width.lock().unwrap() = width;
        // Recreate the canvas state with new dimensions
        *self.state.lock().unwrap() = CanvasState::new(width, self.get_height());
        // Clear the cached context
        *self.context_2d.lock().unwrap() = None;
    }

    pub fn set_height(&self, height: u32) {
        *self.height.lock().unwrap() = height;
        // Recreate the canvas state with new dimensions
        *self.state.lock().unwrap() = CanvasState::new(self.get_width(), height);
        // Clear the cached context
        *self.context_2d.lock().unwrap() = None;
    }

    pub fn get_state(&self) -> Arc<Mutex<Option<CanvasState>>> {
        self.state.clone()
    }

    pub fn set_context_2d(&self, ctx: JsObject) {
        *self.context_2d.lock().unwrap() = Some(ctx);
    }

    pub fn get_context_2d(&self) -> Option<JsObject> {
        self.context_2d.lock().unwrap().clone()
    }

    /// Get the pixel data as PNG
    pub fn to_png(&self) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();
        state.as_ref().and_then(|s| s.to_png())
    }

    /// Get the pixel data as a data URL
    pub fn to_data_url(&self, mime_type: &str) -> String {
        match mime_type {
            "image/png" | "" => {
                if let Some(png_data) = self.to_png() {
                    let base64 = base64_encode(&png_data);
                    format!("data:image/png;base64,{}", base64)
                } else {
                    "data:,".to_string()
                }
            }
            "image/jpeg" => {
                // For now, just return PNG even for JPEG requests
                // A proper implementation would encode as JPEG
                if let Some(png_data) = self.to_png() {
                    let base64 = base64_encode(&png_data);
                    format!("data:image/png;base64,{}", base64)
                } else {
                    "data:,".to_string()
                }
            }
            _ => "data:,".to_string(),
        }
    }
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// JavaScript `HTMLCanvasElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLCanvasElement;

impl IntrinsicObject for HTMLCanvasElement {
    fn init(realm: &Realm) {
        let width_getter = BuiltInBuilder::callable(realm, get_width)
            .name(js_string!("get width"))
            .build();

        let width_setter = BuiltInBuilder::callable(realm, set_width)
            .name(js_string!("set width"))
            .build();

        let height_getter = BuiltInBuilder::callable(realm, get_height)
            .name(js_string!("get height"))
            .build();

        let height_setter = BuiltInBuilder::callable(realm, set_height)
            .name(js_string!("set height"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().html_element().prototype()))
            .accessor(
                js_string!("width"),
                Some(width_getter),
                Some(width_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("height"),
                Some(height_getter),
                Some(height_setter),
                Attribute::CONFIGURABLE,
            )
            .method(get_context, js_string!("getContext"), 1)
            .method(to_data_url, js_string!("toDataURL"), 0)
            .method(to_blob, js_string!("toBlob"), 1)
            .method(transfer_control_to_offscreen, js_string!("transferControlToOffscreen"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLCanvasElement {
    const NAME: JsString = StaticJsStrings::HTML_CANVAS_ELEMENT;
}

impl BuiltInConstructor for HTMLCanvasElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 20;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 20;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_canvas_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_canvas_element,
            context,
        )?;

        // Default dimensions
        let width = args
            .get(0)
            .and_then(|v| v.to_number(context).ok())
            .map(|n| n as u32)
            .unwrap_or(300);

        let height = args
            .get(1)
            .and_then(|v| v.to_number(context).ok())
            .map(|n| n as u32)
            .unwrap_or(150);

        let canvas_data = HTMLCanvasElementData::new(width, height);

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            canvas_data,
        );

        Ok(obj.into())
    }
}

// ============== Property Accessors ==============

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLCanvasElementData>() {
            return Ok(JsValue::from(data.get_width()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLCanvasElement")
        .into())
}

fn set_width(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLCanvasElementData>() {
            let width = args.get_or_undefined(0).to_number(context)? as u32;
            data.set_width(width);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLCanvasElement")
        .into())
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLCanvasElementData>() {
            return Ok(JsValue::from(data.get_height()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLCanvasElement")
        .into())
}

fn set_height(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLCanvasElementData>() {
            let height = args.get_or_undefined(0).to_number(context)? as u32;
            data.set_height(height);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLCanvasElement")
        .into())
}

// ============== Methods ==============

fn get_context(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let canvas_data = this_obj.downcast_ref::<HTMLCanvasElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLCanvasElement")
    })?;

    let context_type = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    match context_type.as_str() {
        "2d" => {
            // Return cached context if available
            if let Some(ctx) = canvas_data.get_context_2d() {
                return Ok(ctx.into());
            }

            // Create new 2D context
            let ctx_data = CanvasRenderingContext2DData::new(canvas_data.get_state());

            let prototype = context
                .intrinsics()
                .constructors()
                .canvas_rendering_context_2d()
                .prototype();

            let ctx_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                ctx_data,
            );

            // Set the canvas back-reference on the context
            let ctx_generic = ctx_obj.upcast();
            ctx_generic.set(js_string!("canvas"), this.clone(), false, context)?;

            // Cache the context
            canvas_data.set_context_2d(ctx_generic.clone());

            Ok(ctx_generic.into())
        }
        "webgl" | "experimental-webgl" => {
            #[cfg(feature = "native")]
            {
                // Create WebGL 1.0 context using the existing implementation
                let width = canvas_data.get_width();
                let height = canvas_data.get_height();
                let webgl_ctx = crate::webgl::WebGLRenderingContext::create_context(width, height, context)?;

                // Set the canvas back-reference on the context
                webgl_ctx.set(js_string!("canvas"), this.clone(), false, context)?;

                Ok(webgl_ctx.into())
            }
            #[cfg(not(feature = "native"))]
            Ok(JsValue::null())
        }
        "webgl2" | "experimental-webgl2" => {
            #[cfg(feature = "native")]
            {
                // Create WebGL 2.0 context using the existing implementation
                let width = canvas_data.get_width();
                let height = canvas_data.get_height();
                let webgl2_ctx = crate::webgl::WebGL2RenderingContext::create_context(width, height, context)?;

                // Set the canvas back-reference on the context
                webgl2_ctx.set(js_string!("canvas"), this.clone(), false, context)?;

                Ok(webgl2_ctx.into())
            }
            #[cfg(not(feature = "native"))]
            Ok(JsValue::null())
        }
        "bitmaprenderer" => {
            // ImageBitmapRenderingContext - return null for now
            Ok(JsValue::null())
        }
        _ => {
            // Unknown context type
            Ok(JsValue::null())
        }
    }
}

fn to_data_url(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let canvas_data = this_obj.downcast_ref::<HTMLCanvasElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLCanvasElement")
    })?;

    let mime_type = args
        .get(0)
        .and_then(|v| v.to_string(context).ok())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_else(|| "image/png".to_string());

    let data_url = canvas_data.to_data_url(&mime_type);
    Ok(JsValue::from(js_string!(data_url)))
}

fn to_blob(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let canvas_data = this_obj.downcast_ref::<HTMLCanvasElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLCanvasElement")
    })?;

    // Get callback function
    let callback = args.get_or_undefined(0);
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("Callback must be a function")
            .into());
    }

    // Get PNG data
    if let Some(png_data) = canvas_data.to_png() {
        // Create a Blob-like object with the PNG data
        // For now, we'll create a simple object with the data
        // A proper Blob implementation would be needed for full compatibility

        // Create a Uint8Array from the PNG data
        let typed_array = context.eval(Source::from_bytes(&format!(
            "new Uint8Array([{}])",
            png_data.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",")
        )))?;

        // Call the callback with the "blob" (typed array for now)
        if let Some(callable) = callback.as_callable() {
            callable.call(&JsValue::undefined(), &[typed_array], context)?;
        }
    } else {
        // Call callback with null if no data
        if let Some(callable) = callback.as_callable() {
            callable.call(&JsValue::undefined(), &[JsValue::null()], context)?;
        }
    }

    Ok(JsValue::undefined())
}

fn transfer_control_to_offscreen(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    // OffscreenCanvas will be implemented in Phase 3.4
    // For now, throw an error
    Err(JsNativeError::typ()
        .with_message("transferControlToOffscreen not yet implemented")
        .into())
}

/// Helper function to create a canvas element programmatically
pub fn create_canvas_element(
    width: u32,
    height: u32,
    context: &mut Context,
) -> JsResult<JsObject> {
    let prototype = context
        .intrinsics()
        .constructors()
        .html_canvas_element()
        .prototype();

    let canvas_data = HTMLCanvasElementData::new(width, height);

    let obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        canvas_data,
    );

    Ok(obj.upcast())
}
