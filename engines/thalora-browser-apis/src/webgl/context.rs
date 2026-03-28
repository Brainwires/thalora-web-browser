//! WebGLRenderingContext - Core types and context creation
//!
//! Provides the core WebGL 1.0 context types and creation logic.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use boa_engine::{
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::{FunctionObjectBuilder, Object, ObjectInitializer},
    realm::Realm,
};
use boa_gc::{Finalize, Trace};

use super::buffer::{VertexAttribArray, WebGLBuffer, WebGLFramebuffer, WebGLRenderbuffer};
use super::shader::{WebGLProgram, WebGLShader};
use super::state::{WebGLConstants, WebGLState};
use super::texture::WebGLTexture;

/// Maximum number of vertex attributes
pub const MAX_VERTEX_ATTRIBS: usize = 16;

/// WebGL context data stored in JavaScript object
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebGLRenderingContextData {
    /// Canvas width
    #[unsafe_ignore_trace]
    pub width: u32,
    /// Canvas height
    #[unsafe_ignore_trace]
    pub height: u32,
    /// Context state
    #[unsafe_ignore_trace]
    pub state: Arc<Mutex<WebGLState>>,
    /// Shaders
    #[unsafe_ignore_trace]
    pub shaders: Arc<Mutex<HashMap<u32, WebGLShader>>>,
    /// Programs
    #[unsafe_ignore_trace]
    pub programs: Arc<Mutex<HashMap<u32, WebGLProgram>>>,
    /// Buffers
    #[unsafe_ignore_trace]
    pub buffers: Arc<Mutex<HashMap<u32, WebGLBuffer>>>,
    /// Textures
    #[unsafe_ignore_trace]
    pub textures: Arc<Mutex<HashMap<u32, WebGLTexture>>>,
    /// Framebuffers
    #[unsafe_ignore_trace]
    pub framebuffers: Arc<Mutex<HashMap<u32, WebGLFramebuffer>>>,
    /// Renderbuffers
    #[unsafe_ignore_trace]
    pub renderbuffers: Arc<Mutex<HashMap<u32, WebGLRenderbuffer>>>,
    /// Vertex attribute arrays
    #[unsafe_ignore_trace]
    pub vertex_attribs: Arc<Mutex<[VertexAttribArray; MAX_VERTEX_ATTRIBS]>>,
    /// Current error
    #[unsafe_ignore_trace]
    pub error: Arc<Mutex<u32>>,
    /// Render target (pixel buffer for reading back)
    #[unsafe_ignore_trace]
    pub render_target: Arc<Mutex<Vec<u8>>>,
    /// Uniform values cache
    #[unsafe_ignore_trace]
    pub uniform_values: Arc<Mutex<HashMap<u32, Vec<f32>>>>,
    /// Context lost
    #[unsafe_ignore_trace]
    pub context_lost: Arc<Mutex<bool>>,
}

impl WebGLRenderingContextData {
    /// Create new context data
    pub fn new(width: u32, height: u32) -> Self {
        let mut state = WebGLState::default();
        state.viewport = [0, 0, width as i32, height as i32];
        state.scissor = [0, 0, width as i32, height as i32];

        let render_target = vec![0u8; (width * height * 4) as usize];

        Self {
            width,
            height,
            state: Arc::new(Mutex::new(state)),
            shaders: Arc::new(Mutex::new(HashMap::new())),
            programs: Arc::new(Mutex::new(HashMap::new())),
            buffers: Arc::new(Mutex::new(HashMap::new())),
            textures: Arc::new(Mutex::new(HashMap::new())),
            framebuffers: Arc::new(Mutex::new(HashMap::new())),
            renderbuffers: Arc::new(Mutex::new(HashMap::new())),
            vertex_attribs: Arc::new(Mutex::new(std::array::from_fn(|_| {
                VertexAttribArray::default()
            }))),
            error: Arc::new(Mutex::new(WebGLConstants::NO_ERROR)),
            render_target: Arc::new(Mutex::new(render_target)),
            uniform_values: Arc::new(Mutex::new(HashMap::new())),
            context_lost: Arc::new(Mutex::new(false)),
        }
    }

    /// Set error
    pub fn set_error(&self, error: u32) {
        let mut err = self.error.lock().unwrap();
        if *err == WebGLConstants::NO_ERROR {
            *err = error;
        }
    }

    /// Get and clear error
    pub fn get_error(&self) -> u32 {
        let mut err = self.error.lock().unwrap();
        let e = *err;
        *err = WebGLConstants::NO_ERROR;
        e
    }
}

/// Helper macro to get context data from this
/// This must be a macro because we need to keep the JsObject in the caller's scope
/// for the GcRef borrow to be valid
#[macro_export]
macro_rules! with_webgl_context {
    ($this:expr => $obj:ident, $data:ident) => {
        let $obj = $this.as_object().ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("Not a WebGLRenderingContext")
        })?;
        let $data = $obj
            .downcast_ref::<$crate::webgl::context::WebGLRenderingContextData>()
            .ok_or_else(|| {
                boa_engine::JsNativeError::typ().with_message("Not a WebGLRenderingContext")
            })?;
    };
}

/// WebGLRenderingContext JavaScript class
pub struct WebGLRenderingContext;

impl WebGLRenderingContext {
    /// Initialize the WebGLRenderingContext in the realm
    /// WebGL contexts are created dynamically via canvas.getContext("webgl")
    pub fn init(_realm: &Realm) {
        // WebGL contexts are created on-demand, not as global constructors
        // The actual context creation happens in create_context()
    }

    /// Create the global WebGLRenderingContext constructor for global registration
    /// Per Web spec, WebGLRenderingContext IS a global constructor with static constants
    pub fn create_global_constructor(context: &mut Context) -> JsResult<JsObject> {
        // The constructor itself throws when called - contexts are created via getContext()
        fn webgl_constructor(
            _this: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> JsResult<JsValue> {
            Err(JsNativeError::typ()
                .with_message("WebGLRenderingContext cannot be directly constructed; use canvas.getContext('webgl')")
                .into())
        }

        // Create the prototype object with all methods
        let prototype = ObjectInitializer::new(context).build();

        // Add all methods to prototype
        super::methods_shader::add_context_methods(&prototype, context);
        super::methods_shader::add_shader_methods(&prototype, context);
        super::methods_buffer::add_buffer_methods(&prototype, context);
        super::methods_texture::add_texture_methods(&prototype, context);
        super::methods_texture::add_framebuffer_methods(&prototype, context);
        super::methods_uniform::add_uniform_methods(&prototype, context);
        super::methods_uniform::add_vertex_attrib_methods(&prototype, context);
        super::methods_draw::add_draw_methods(&prototype, context);
        super::methods_draw::add_state_methods(&prototype, context);

        // Create constructor function
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(webgl_constructor),
        )
        .name(js_string!("WebGLRenderingContext"))
        .length(0)
        .constructor(true)
        .build();

        // Add all WebGL constants to the constructor (static properties)
        super::constants::add_webgl_constants(&constructor.clone().into(), context);

        // Set prototype
        constructor.set(js_string!("prototype"), prototype, false, context)?;

        Ok(constructor.into())
    }

    /// Create a new WebGL context for a canvas
    pub fn create_context(width: u32, height: u32, context: &mut Context) -> JsResult<JsObject> {
        let data = WebGLRenderingContextData::new(width, height);

        // Create the context object with data
        let obj = JsObject::from_proto_and_data(None, data);

        // Add all WebGL constants to the instance
        super::constants::add_webgl_constants(&obj, context);

        // Add methods
        super::methods_shader::add_context_methods(&obj, context);
        super::methods_shader::add_shader_methods(&obj, context);
        super::methods_buffer::add_buffer_methods(&obj, context);
        super::methods_texture::add_texture_methods(&obj, context);
        super::methods_texture::add_framebuffer_methods(&obj, context);
        super::methods_uniform::add_uniform_methods(&obj, context);
        super::methods_uniform::add_vertex_attrib_methods(&obj, context);
        super::methods_draw::add_draw_methods(&obj, context);
        super::methods_draw::add_state_methods(&obj, context);

        // Add canvas property
        obj.set(js_string!("canvas"), JsValue::null(), false, context)?;

        // Add drawing buffer properties
        obj.set(
            js_string!("drawingBufferWidth"),
            JsValue::from(width),
            false,
            context,
        )?;
        obj.set(
            js_string!("drawingBufferHeight"),
            JsValue::from(height),
            false,
            context,
        )?;

        Ok(obj)
    }
}

/// Helper function to get object ID
pub fn get_object_id(val: &JsValue, ctx: &mut Context) -> JsResult<u32> {
    if val.is_null() || val.is_undefined() {
        return Ok(0);
    }

    if let Some(obj) = val.as_object() {
        if let Ok(id_val) = obj.get(js_string!("_id"), ctx) {
            return id_val.to_u32(ctx);
        }
    }

    Ok(0)
}

/// Helper function to get uniform location ID
pub fn get_location_id(val: &JsValue, ctx: &mut Context) -> JsResult<u32> {
    get_object_id(val, ctx)
}

/// Get parameter value
pub fn get_parameter(
    data: &WebGLRenderingContextData,
    pname: u32,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let state = data.state.lock().unwrap();

    match pname {
        WebGLConstants::VENDOR => Ok(JsValue::from(js_string!("Thalora"))),
        WebGLConstants::RENDERER => Ok(JsValue::from(js_string!("Thalora WebGL"))),
        WebGLConstants::VERSION => Ok(JsValue::from(js_string!("WebGL 1.0 (Thalora)"))),
        WebGLConstants::SHADING_LANGUAGE_VERSION => {
            Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0")))
        }
        WebGLConstants::MAX_TEXTURE_SIZE => Ok(JsValue::from(4096)),
        WebGLConstants::MAX_CUBE_MAP_TEXTURE_SIZE => Ok(JsValue::from(4096)),
        WebGLConstants::MAX_RENDERBUFFER_SIZE => Ok(JsValue::from(4096)),
        WebGLConstants::MAX_VERTEX_ATTRIBS => Ok(JsValue::from(MAX_VERTEX_ATTRIBS as u32)),
        WebGLConstants::MAX_VERTEX_UNIFORM_VECTORS => Ok(JsValue::from(256)),
        WebGLConstants::MAX_VARYING_VECTORS => Ok(JsValue::from(15)),
        WebGLConstants::MAX_FRAGMENT_UNIFORM_VECTORS => Ok(JsValue::from(256)),
        WebGLConstants::MAX_VERTEX_TEXTURE_IMAGE_UNITS => Ok(JsValue::from(4)),
        WebGLConstants::MAX_TEXTURE_IMAGE_UNITS => Ok(JsValue::from(8)),
        WebGLConstants::MAX_COMBINED_TEXTURE_IMAGE_UNITS => Ok(JsValue::from(8)),
        WebGLConstants::VIEWPORT => {
            let arr = boa_engine::object::builtins::JsArray::from_iter(
                state.viewport.iter().map(|&v| JsValue::from(v)),
                ctx,
            );
            Ok(arr.into())
        }
        WebGLConstants::SCISSOR_BOX => {
            let arr = boa_engine::object::builtins::JsArray::from_iter(
                state.scissor.iter().map(|&v| JsValue::from(v)),
                ctx,
            );
            Ok(arr.into())
        }
        WebGLConstants::COLOR_CLEAR_VALUE => {
            let arr = boa_engine::object::builtins::JsArray::from_iter(
                state.clear_color.iter().map(|&v| JsValue::from(v)),
                ctx,
            );
            Ok(arr.into())
        }
        WebGLConstants::DEPTH_CLEAR_VALUE => Ok(JsValue::from(state.clear_depth)),
        WebGLConstants::STENCIL_CLEAR_VALUE => Ok(JsValue::from(state.clear_stencil)),
        WebGLConstants::BLEND => Ok(JsValue::from(state.blend)),
        WebGLConstants::CULL_FACE => Ok(JsValue::from(state.cull_face)),
        WebGLConstants::DEPTH_TEST => Ok(JsValue::from(state.depth_test)),
        WebGLConstants::DITHER => Ok(JsValue::from(state.dither)),
        WebGLConstants::SCISSOR_TEST => Ok(JsValue::from(state.scissor_test)),
        WebGLConstants::STENCIL_TEST => Ok(JsValue::from(state.stencil_test)),
        WebGLConstants::DEPTH_FUNC => Ok(JsValue::from(state.depth_func)),
        WebGLConstants::DEPTH_WRITEMASK => Ok(JsValue::from(state.depth_mask)),
        WebGLConstants::FRONT_FACE => Ok(JsValue::from(state.front_face)),
        WebGLConstants::CULL_FACE_MODE => Ok(JsValue::from(state.cull_face_mode)),
        WebGLConstants::LINE_WIDTH => Ok(JsValue::from(state.line_width)),
        WebGLConstants::BLEND_SRC_RGB => Ok(JsValue::from(state.blend_src_rgb)),
        WebGLConstants::BLEND_DST_RGB => Ok(JsValue::from(state.blend_dst_rgb)),
        WebGLConstants::BLEND_SRC_ALPHA => Ok(JsValue::from(state.blend_src_alpha)),
        WebGLConstants::BLEND_DST_ALPHA => Ok(JsValue::from(state.blend_dst_alpha)),
        WebGLConstants::BLEND_EQUATION_RGB => Ok(JsValue::from(state.blend_equation_rgb)),
        WebGLConstants::BLEND_EQUATION_ALPHA => Ok(JsValue::from(state.blend_equation_alpha)),
        WebGLConstants::UNPACK_ALIGNMENT => Ok(JsValue::from(state.unpack_alignment)),
        WebGLConstants::PACK_ALIGNMENT => Ok(JsValue::from(state.pack_alignment)),
        WebGLConstants::UNPACK_FLIP_Y_WEBGL => Ok(JsValue::from(state.unpack_flip_y)),
        WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
            Ok(JsValue::from(state.unpack_premultiply_alpha))
        }
        _ => Ok(JsValue::null()),
    }
}
