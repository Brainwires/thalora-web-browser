//! WebGLRenderingContext Implementation
//!
//! Provides the WebGL 1.0 rendering context using wgpu as the backend.
//! https://www.khronos.org/webgl/

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use boa_engine::{
    js_string,
    object::{builtins::JsArrayBuffer, Object},
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};

use super::buffer::{VertexAttribArray, WebGLBuffer, WebGLFramebuffer, WebGLRenderbuffer};
use super::shader::{WebGLProgram, WebGLShader};
use super::state::{WebGLConstants, WebGLState};
use super::texture::WebGLTexture;

/// Maximum number of vertex attributes
const MAX_VERTEX_ATTRIBS: usize = 16;

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
            vertex_attribs: Arc::new(Mutex::new(std::array::from_fn(|_| VertexAttribArray::default()))),
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
macro_rules! with_context_data {
    ($this:expr => $obj:ident, $data:ident) => {
        let $obj = $this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Not a WebGLRenderingContext")
        })?;
        let $data = $obj.downcast_ref::<WebGLRenderingContextData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Not a WebGLRenderingContext")
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

    /// Create a new WebGL context for a canvas
    pub fn create_context(width: u32, height: u32, context: &mut Context) -> JsResult<JsObject> {
        let data = WebGLRenderingContextData::new(width, height);

        // Create the context object with data
        let obj = JsObject::from_proto_and_data(None, data);

        // Add all WebGL constants to the instance
        add_webgl_constants(&obj, context);

        // Add methods
        add_context_methods(&obj, context);
        add_shader_methods(&obj, context);
        add_buffer_methods(&obj, context);
        add_texture_methods(&obj, context);
        add_framebuffer_methods(&obj, context);
        add_uniform_methods(&obj, context);
        add_vertex_attrib_methods(&obj, context);
        add_draw_methods(&obj, context);
        add_state_methods(&obj, context);

        // Add canvas property
        obj.set(js_string!("canvas"), JsValue::null(), false, context)?;

        // Add drawing buffer properties
        obj.set(js_string!("drawingBufferWidth"), JsValue::from(width), false, context)?;
        obj.set(js_string!("drawingBufferHeight"), JsValue::from(height), false, context)?;

        Ok(obj)
    }
}

/// Add WebGL constants to object
fn add_webgl_constants(obj: &JsObject, context: &mut Context) {
    let constants = [
        // Clearing buffers
        ("DEPTH_BUFFER_BIT", WebGLConstants::DEPTH_BUFFER_BIT),
        ("STENCIL_BUFFER_BIT", WebGLConstants::STENCIL_BUFFER_BIT),
        ("COLOR_BUFFER_BIT", WebGLConstants::COLOR_BUFFER_BIT),
        // Rendering primitives
        ("POINTS", WebGLConstants::POINTS),
        ("LINES", WebGLConstants::LINES),
        ("LINE_LOOP", WebGLConstants::LINE_LOOP),
        ("LINE_STRIP", WebGLConstants::LINE_STRIP),
        ("TRIANGLES", WebGLConstants::TRIANGLES),
        ("TRIANGLE_STRIP", WebGLConstants::TRIANGLE_STRIP),
        ("TRIANGLE_FAN", WebGLConstants::TRIANGLE_FAN),
        // Blending
        ("ZERO", WebGLConstants::ZERO),
        ("ONE", WebGLConstants::ONE),
        ("SRC_COLOR", WebGLConstants::SRC_COLOR),
        ("ONE_MINUS_SRC_COLOR", WebGLConstants::ONE_MINUS_SRC_COLOR),
        ("SRC_ALPHA", WebGLConstants::SRC_ALPHA),
        ("ONE_MINUS_SRC_ALPHA", WebGLConstants::ONE_MINUS_SRC_ALPHA),
        ("DST_ALPHA", WebGLConstants::DST_ALPHA),
        ("ONE_MINUS_DST_ALPHA", WebGLConstants::ONE_MINUS_DST_ALPHA),
        ("DST_COLOR", WebGLConstants::DST_COLOR),
        ("ONE_MINUS_DST_COLOR", WebGLConstants::ONE_MINUS_DST_COLOR),
        ("SRC_ALPHA_SATURATE", WebGLConstants::SRC_ALPHA_SATURATE),
        // Blend equations
        ("FUNC_ADD", WebGLConstants::FUNC_ADD),
        ("FUNC_SUBTRACT", WebGLConstants::FUNC_SUBTRACT),
        ("FUNC_REVERSE_SUBTRACT", WebGLConstants::FUNC_REVERSE_SUBTRACT),
        // Data types
        ("BYTE", WebGLConstants::BYTE),
        ("UNSIGNED_BYTE", WebGLConstants::UNSIGNED_BYTE),
        ("SHORT", WebGLConstants::SHORT),
        ("UNSIGNED_SHORT", WebGLConstants::UNSIGNED_SHORT),
        ("INT", WebGLConstants::INT),
        ("UNSIGNED_INT", WebGLConstants::UNSIGNED_INT),
        ("FLOAT", WebGLConstants::FLOAT),
        // Pixel formats
        ("DEPTH_COMPONENT", WebGLConstants::DEPTH_COMPONENT),
        ("ALPHA", WebGLConstants::ALPHA),
        ("RGB", WebGLConstants::RGB),
        ("RGBA", WebGLConstants::RGBA),
        ("LUMINANCE", WebGLConstants::LUMINANCE),
        ("LUMINANCE_ALPHA", WebGLConstants::LUMINANCE_ALPHA),
        // Shaders
        ("FRAGMENT_SHADER", WebGLConstants::FRAGMENT_SHADER),
        ("VERTEX_SHADER", WebGLConstants::VERTEX_SHADER),
        ("COMPILE_STATUS", WebGLConstants::COMPILE_STATUS),
        ("LINK_STATUS", WebGLConstants::LINK_STATUS),
        ("VALIDATE_STATUS", WebGLConstants::VALIDATE_STATUS),
        ("DELETE_STATUS", WebGLConstants::DELETE_STATUS),
        ("SHADER_TYPE", WebGLConstants::SHADER_TYPE),
        ("ATTACHED_SHADERS", WebGLConstants::ATTACHED_SHADERS),
        ("ACTIVE_UNIFORMS", WebGLConstants::ACTIVE_UNIFORMS),
        ("ACTIVE_ATTRIBUTES", WebGLConstants::ACTIVE_ATTRIBUTES),
        // Buffers
        ("ARRAY_BUFFER", WebGLConstants::ARRAY_BUFFER),
        ("ELEMENT_ARRAY_BUFFER", WebGLConstants::ELEMENT_ARRAY_BUFFER),
        ("BUFFER_SIZE", WebGLConstants::BUFFER_SIZE),
        ("BUFFER_USAGE", WebGLConstants::BUFFER_USAGE),
        ("STATIC_DRAW", WebGLConstants::STATIC_DRAW),
        ("STREAM_DRAW", WebGLConstants::STREAM_DRAW),
        ("DYNAMIC_DRAW", WebGLConstants::DYNAMIC_DRAW),
        // Textures
        ("TEXTURE_2D", WebGLConstants::TEXTURE_2D),
        ("TEXTURE_CUBE_MAP", WebGLConstants::TEXTURE_CUBE_MAP),
        ("TEXTURE_MIN_FILTER", WebGLConstants::TEXTURE_MIN_FILTER),
        ("TEXTURE_MAG_FILTER", WebGLConstants::TEXTURE_MAG_FILTER),
        ("TEXTURE_WRAP_S", WebGLConstants::TEXTURE_WRAP_S),
        ("TEXTURE_WRAP_T", WebGLConstants::TEXTURE_WRAP_T),
        ("NEAREST", WebGLConstants::NEAREST),
        ("LINEAR", WebGLConstants::LINEAR),
        ("NEAREST_MIPMAP_NEAREST", WebGLConstants::NEAREST_MIPMAP_NEAREST),
        ("LINEAR_MIPMAP_NEAREST", WebGLConstants::LINEAR_MIPMAP_NEAREST),
        ("NEAREST_MIPMAP_LINEAR", WebGLConstants::NEAREST_MIPMAP_LINEAR),
        ("LINEAR_MIPMAP_LINEAR", WebGLConstants::LINEAR_MIPMAP_LINEAR),
        ("REPEAT", WebGLConstants::REPEAT),
        ("CLAMP_TO_EDGE", WebGLConstants::CLAMP_TO_EDGE),
        ("MIRRORED_REPEAT", WebGLConstants::MIRRORED_REPEAT),
        ("TEXTURE0", WebGLConstants::TEXTURE0),
        ("TEXTURE1", WebGLConstants::TEXTURE1),
        ("TEXTURE2", WebGLConstants::TEXTURE2),
        ("TEXTURE3", WebGLConstants::TEXTURE3),
        ("TEXTURE4", WebGLConstants::TEXTURE4),
        ("TEXTURE5", WebGLConstants::TEXTURE5),
        ("TEXTURE6", WebGLConstants::TEXTURE6),
        ("TEXTURE7", WebGLConstants::TEXTURE7),
        // Culling
        ("CULL_FACE", WebGLConstants::CULL_FACE),
        ("FRONT", WebGLConstants::FRONT),
        ("BACK", WebGLConstants::BACK),
        ("FRONT_AND_BACK", WebGLConstants::FRONT_AND_BACK),
        ("CW", WebGLConstants::CW),
        ("CCW", WebGLConstants::CCW),
        // Depth/stencil
        ("DEPTH_TEST", WebGLConstants::DEPTH_TEST),
        ("STENCIL_TEST", WebGLConstants::STENCIL_TEST),
        ("BLEND", WebGLConstants::BLEND),
        ("DITHER", WebGLConstants::DITHER),
        ("SCISSOR_TEST", WebGLConstants::SCISSOR_TEST),
        ("POLYGON_OFFSET_FILL", WebGLConstants::POLYGON_OFFSET_FILL),
        // Comparison functions
        ("NEVER", WebGLConstants::NEVER),
        ("LESS", WebGLConstants::LESS),
        ("EQUAL", WebGLConstants::EQUAL),
        ("LEQUAL", WebGLConstants::LEQUAL),
        ("GREATER", WebGLConstants::GREATER),
        ("NOTEQUAL", WebGLConstants::NOTEQUAL),
        ("GEQUAL", WebGLConstants::GEQUAL),
        ("ALWAYS", WebGLConstants::ALWAYS),
        // Stencil ops
        ("KEEP", WebGLConstants::KEEP),
        ("REPLACE", WebGLConstants::REPLACE),
        ("INCR", WebGLConstants::INCR),
        ("DECR", WebGLConstants::DECR),
        ("INVERT", WebGLConstants::INVERT),
        ("INCR_WRAP", WebGLConstants::INCR_WRAP),
        ("DECR_WRAP", WebGLConstants::DECR_WRAP),
        // Framebuffers
        ("FRAMEBUFFER", WebGLConstants::FRAMEBUFFER),
        ("RENDERBUFFER", WebGLConstants::RENDERBUFFER),
        ("COLOR_ATTACHMENT0", WebGLConstants::COLOR_ATTACHMENT0),
        ("DEPTH_ATTACHMENT", WebGLConstants::DEPTH_ATTACHMENT),
        ("STENCIL_ATTACHMENT", WebGLConstants::STENCIL_ATTACHMENT),
        ("DEPTH_STENCIL_ATTACHMENT", WebGLConstants::DEPTH_STENCIL_ATTACHMENT),
        ("FRAMEBUFFER_COMPLETE", WebGLConstants::FRAMEBUFFER_COMPLETE),
        // Errors
        ("NO_ERROR", WebGLConstants::NO_ERROR),
        ("INVALID_ENUM", WebGLConstants::INVALID_ENUM),
        ("INVALID_VALUE", WebGLConstants::INVALID_VALUE),
        ("INVALID_OPERATION", WebGLConstants::INVALID_OPERATION),
        ("OUT_OF_MEMORY", WebGLConstants::OUT_OF_MEMORY),
        ("INVALID_FRAMEBUFFER_OPERATION", WebGLConstants::INVALID_FRAMEBUFFER_OPERATION),
        ("CONTEXT_LOST_WEBGL", WebGLConstants::CONTEXT_LOST_WEBGL),
        // Getting info
        ("VENDOR", WebGLConstants::VENDOR),
        ("RENDERER", WebGLConstants::RENDERER),
        ("VERSION", WebGLConstants::VERSION),
        ("SHADING_LANGUAGE_VERSION", WebGLConstants::SHADING_LANGUAGE_VERSION),
        // Pixel storage
        ("UNPACK_FLIP_Y_WEBGL", WebGLConstants::UNPACK_FLIP_Y_WEBGL),
        ("UNPACK_PREMULTIPLY_ALPHA_WEBGL", WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL),
        ("UNPACK_COLORSPACE_CONVERSION_WEBGL", WebGLConstants::UNPACK_COLORSPACE_CONVERSION_WEBGL),
        ("UNPACK_ALIGNMENT", WebGLConstants::UNPACK_ALIGNMENT),
        ("PACK_ALIGNMENT", WebGLConstants::PACK_ALIGNMENT),
    ];

    for (name, value) in constants {
        let _ = obj.define_property_or_throw(
            js_string!(name),
            boa_engine::property::PropertyDescriptor::builder()
                .value(JsValue::from(value))
                .writable(false)
                .enumerable(true)
                .configurable(false),
            context,
        );
    }
}

/// Add context methods
fn add_context_methods(obj: &JsObject, context: &mut Context) {
    // getError
    obj.set(
        js_string!("getError"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context_data!(this => _ctx_obj, data);
            Ok(JsValue::from(data.get_error()))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isContextLost
    obj.set(
        js_string!("isContextLost"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context_data!(this => _ctx_obj, data);
            let lost = *data.context_lost.lock().unwrap();
            Ok(JsValue::from(lost))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getParameter
    obj.set(
        js_string!("getParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let pname = args.get_or_undefined(0).to_u32(ctx)?;
            get_parameter(&data, pname, ctx)
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getExtension
    obj.set(
        js_string!("getExtension"),
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let name = args.get_or_undefined(0).to_string(ctx)?;
            // Return null for now - extensions can be added later
            let _ = name;
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getSupportedExtensions
    obj.set(
        js_string!("getSupportedExtensions"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let arr = boa_engine::object::builtins::JsArray::new(ctx);
            Ok(arr.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getContextAttributes
    obj.set(
        js_string!("getContextAttributes"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let obj = JsObject::with_null_proto();
            obj.set(js_string!("alpha"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("depth"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("stencil"), JsValue::from(false), false, ctx)?;
            obj.set(js_string!("antialias"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("premultipliedAlpha"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("preserveDrawingBuffer"), JsValue::from(false), false, ctx)?;
            obj.set(js_string!("powerPreference"), js_string!("default"), false, ctx)?;
            obj.set(js_string!("failIfMajorPerformanceCaveat"), JsValue::from(false), false, ctx)?;
            Ok(obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add shader methods
fn add_shader_methods(obj: &JsObject, context: &mut Context) {
    // createShader
    obj.set(
        js_string!("createShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_type = args.get_or_undefined(0).to_u32(ctx)?;

            if shader_type != WebGLConstants::VERTEX_SHADER
                && shader_type != WebGLConstants::FRAGMENT_SHADER
            {
                data.set_error(WebGLConstants::INVALID_ENUM);
                return Ok(JsValue::null());
            }

            let shader = WebGLShader::new(shader_type);
            let id = shader.id;
            data.shaders.lock().unwrap().insert(id, shader);

            // Return shader object
            let shader_obj = JsObject::with_null_proto();
            shader_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(shader_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // shaderSource
    obj.set(
        js_string!("shaderSource"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let source = args.get_or_undefined(1).to_string(ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get_mut(&shader_id) {
                shader.set_source(&source.to_std_string_escaped());
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // compileShader
    obj.set(
        js_string!("compileShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get_mut(&shader_id) {
                shader.compile();
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getShaderParameter
    obj.set(
        js_string!("getShaderParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get(&shader_id) {
                if let Some(param) = shader.get_parameter(pname) {
                    return match param {
                        super::shader::ShaderParameter::Int(v) => Ok(JsValue::from(v)),
                        super::shader::ShaderParameter::Bool(v) => Ok(JsValue::from(v)),
                    };
                }
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getShaderInfoLog
    obj.set(
        js_string!("getShaderInfoLog"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get(&shader_id) {
                return Ok(JsValue::from(js_string!(shader.get_info_log())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getShaderSource
    obj.set(
        js_string!("getShaderSource"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get(&shader_id) {
                return Ok(JsValue::from(js_string!(shader.get_source())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteShader
    obj.set(
        js_string!("deleteShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get_mut(&shader_id) {
                shader.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // createProgram
    obj.set(
        js_string!("createProgram"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context_data!(this => _ctx_obj, data);

            let program = WebGLProgram::new();
            let id = program.id;
            data.programs.lock().unwrap().insert(id, program);

            let program_obj = JsObject::with_null_proto();
            program_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(program_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // attachShader
    obj.set(
        js_string!("attachShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let shader_id = get_object_id(args.get_or_undefined(1), ctx)?;

            let shaders = data.shaders.lock().unwrap();
            if let Some(shader) = shaders.get(&shader_id) {
                if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                    program.attach_shader(shader);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // detachShader
    obj.set(
        js_string!("detachShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let shader_id = get_object_id(args.get_or_undefined(1), ctx)?;

            let shaders = data.shaders.lock().unwrap();
            if let Some(shader) = shaders.get(&shader_id) {
                if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                    program.detach_shader(shader);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // linkProgram
    obj.set(
        js_string!("linkProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            let shaders = data.shaders.lock().unwrap();
            if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                program.link(&shaders);
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // useProgram
    obj.set(
        js_string!("useProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = if args.get_or_undefined(0).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(0), ctx)?)
            };

            data.state.lock().unwrap().current_program = program_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getProgramParameter
    obj.set(
        js_string!("getProgramParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                if let Some(param) = program.get_parameter(pname) {
                    return match param {
                        super::shader::ProgramParameter::Int(v) => Ok(JsValue::from(v)),
                        super::shader::ProgramParameter::Bool(v) => Ok(JsValue::from(v)),
                    };
                }
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getProgramInfoLog
    obj.set(
        js_string!("getProgramInfoLog"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                return Ok(JsValue::from(js_string!(program.get_info_log())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // validateProgram
    obj.set(
        js_string!("validateProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                program.validate();
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteProgram
    obj.set(
        js_string!("deleteProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                program.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getUniformLocation
    obj.set(
        js_string!("getUniformLocation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let name = args.get_or_undefined(1).to_string(ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                if let Some(location) = program.get_uniform_location(&name.to_std_string_escaped()) {
                    let loc_obj = JsObject::with_null_proto();
                    loc_obj.set(js_string!("_id"), JsValue::from(location.id), false, ctx)?;
                    loc_obj.set(js_string!("_program"), JsValue::from(program_id), false, ctx)?;
                    return Ok(loc_obj.into());
                }
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getAttribLocation
    obj.set(
        js_string!("getAttribLocation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let name = args.get_or_undefined(1).to_string(ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                if let Some(location) = program.get_attrib_location(&name.to_std_string_escaped()) {
                    return Ok(JsValue::from(location));
                }
            }
            Ok(JsValue::from(-1))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add buffer methods
fn add_buffer_methods(obj: &JsObject, context: &mut Context) {
    // createBuffer
    obj.set(
        js_string!("createBuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context_data!(this => _ctx_obj, data);

            let buffer = WebGLBuffer::new();
            let id = buffer.id;
            data.buffers.lock().unwrap().insert(id, buffer);

            let buffer_obj = JsObject::with_null_proto();
            buffer_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(buffer_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindBuffer
    obj.set(
        js_string!("bindBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            let mut state = data.state.lock().unwrap();
            match target {
                WebGLConstants::ARRAY_BUFFER => {
                    state.bound_array_buffer = buffer_id;
                }
                WebGLConstants::ELEMENT_ARRAY_BUFFER => {
                    state.bound_element_array_buffer = buffer_id;
                }
                _ => {
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bufferData
    obj.set(
        js_string!("bufferData"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let usage = args.get_or_undefined(2).to_u32(ctx)?;

            let buffer_id = {
                let state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::ARRAY_BUFFER => state.bound_array_buffer,
                    WebGLConstants::ELEMENT_ARRAY_BUFFER => state.bound_element_array_buffer,
                    _ => None,
                }
            };

            let Some(buffer_id) = buffer_id else {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            };

            let buf_data_arg = args.get_or_undefined(1);

            // Handle size or data
            if buf_data_arg.is_number() {
                let size = buf_data_arg.to_index(ctx)? as usize;
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.allocate(size, usage);
                }
            } else if let Some(array_buffer) = buf_data_arg.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                let buf = array_buffer.data().expect("ArrayBuffer has no data").as_ref().to_vec();
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.set_data(&buf, usage);
                }
            } else if let Some(obj) = buf_data_arg.as_object() {
                // Try to get typed array data
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                        let byte_offset = obj.get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;
                        let byte_length = obj.get(js_string!("byteLength"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        let full_buf = ab.data().expect("ArrayBuffer has no data").as_ref().to_vec();
                        let slice = &full_buf[byte_offset..byte_offset + byte_length];
                        if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                            buffer.set_data(slice, usage);
                        }
                    }
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bufferSubData
    obj.set(
        js_string!("bufferSubData"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let offset = args.get_or_undefined(1).to_index(ctx)? as usize;

            let buffer_id = {
                let state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::ARRAY_BUFFER => state.bound_array_buffer,
                    WebGLConstants::ELEMENT_ARRAY_BUFFER => state.bound_element_array_buffer,
                    _ => None,
                }
            };

            let Some(buffer_id) = buffer_id else {
                return Ok(JsValue::undefined());
            };

            let buf_data_arg = args.get_or_undefined(2);

            if let Some(array_buffer) = buf_data_arg.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                let buf = array_buffer.data().expect("ArrayBuffer has no data").as_ref().to_vec();
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.set_sub_data(offset, &buf);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteBuffer
    obj.set(
        js_string!("deleteBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let buffer_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                buffer.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isBuffer
    obj.set(
        js_string!("isBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let buffer_id = get_object_id(args.get_or_undefined(0), ctx)?;

            let exists = data.buffers.lock().unwrap().contains_key(&buffer_id);
            Ok(JsValue::from(exists))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add texture methods
fn add_texture_methods(obj: &JsObject, context: &mut Context) {
    // createTexture
    obj.set(
        js_string!("createTexture"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context_data!(this => _ctx_obj, data);

            let texture = WebGLTexture::new();
            let id = texture.id;
            data.textures.lock().unwrap().insert(id, texture);

            let tex_obj = JsObject::with_null_proto();
            tex_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(tex_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindTexture
    obj.set(
        js_string!("bindTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let texture_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            let state = data.state.lock().unwrap();
            let unit = state.active_texture - WebGLConstants::TEXTURE0;
            drop(state);

            if let Some(tex_id) = texture_id {
                let mut state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::TEXTURE_2D => {
                        state.texture_bindings_2d.insert(unit, tex_id);
                    }
                    WebGLConstants::TEXTURE_CUBE_MAP => {
                        state.texture_bindings_cube.insert(unit, tex_id);
                    }
                    _ => {}
                }

                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.bind(target);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // activeTexture
    obj.set(
        js_string!("activeTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let texture = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().active_texture = texture;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // texImage2D
    obj.set(
        js_string!("texImage2D"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let level = args.get_or_undefined(1).to_i32(ctx)?;
            let internal_format = args.get_or_undefined(2).to_u32(ctx)?;
            let width = args.get_or_undefined(3).to_u32(ctx)?;
            let height = args.get_or_undefined(4).to_u32(ctx)?;
            let _border = args.get_or_undefined(5).to_i32(ctx)?;
            let format = args.get_or_undefined(6).to_u32(ctx)?;
            let data_type = args.get_or_undefined(7).to_u32(ctx)?;

            // Get pixel data
            let pixels = args.get_or_undefined(8);
            let pixel_data: Option<Vec<u8>> = if pixels.is_null() || pixels.is_undefined() {
                None
            } else if let Some(array_buffer) = pixels.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                Some(array_buffer.data().expect("ArrayBuffer has no data").as_ref().to_vec())
            } else if let Some(obj) = pixels.as_object() {
                // Try typed array
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                        let byte_offset = obj.get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;
                        let byte_length = obj.get(js_string!("byteLength"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        let full_buf = ab.data().expect("ArrayBuffer has no data").as_ref().to_vec();
                        Some(full_buf[byte_offset..byte_offset + byte_length].to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Get bound texture
            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else if target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                    && target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                {
                    state.texture_bindings_cube.get(&unit).copied()
                } else {
                    None
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    if target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                        && target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                    {
                        texture.tex_image_cube_face(
                            target,
                            level,
                            internal_format,
                            width,
                            height,
                            format,
                            data_type,
                            pixel_data.as_deref(),
                        );
                    } else {
                        texture.tex_image_2d(
                            target,
                            level,
                            internal_format,
                            width,
                            height,
                            format,
                            data_type,
                            pixel_data.as_deref(),
                        );
                    }
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // texParameteri
    obj.set(
        js_string!("texParameteri"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;
            let param = args.get_or_undefined(2).to_u32(ctx)?;

            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else {
                    state.texture_bindings_cube.get(&unit).copied()
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.tex_parameter(pname, param);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // generateMipmap
    obj.set(
        js_string!("generateMipmap"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;

            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else {
                    state.texture_bindings_cube.get(&unit).copied()
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.generate_mipmap();
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteTexture
    obj.set(
        js_string!("deleteTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let texture_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(texture) = data.textures.lock().unwrap().get_mut(&texture_id) {
                texture.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add framebuffer methods
fn add_framebuffer_methods(obj: &JsObject, context: &mut Context) {
    // createFramebuffer
    obj.set(
        js_string!("createFramebuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context_data!(this => _ctx_obj, data);

            let fb = WebGLFramebuffer::new();
            let id = fb.id;
            data.framebuffers.lock().unwrap().insert(id, fb);

            let fb_obj = JsObject::with_null_proto();
            fb_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(fb_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindFramebuffer
    obj.set(
        js_string!("bindFramebuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let fb_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            data.state.lock().unwrap().bound_framebuffer = fb_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // checkFramebufferStatus
    obj.set(
        js_string!("checkFramebufferStatus"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context_data!(this => _ctx_obj, data);

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get(&fb_id) {
                    return Ok(JsValue::from(fb.check_status()));
                }
            }

            // Default framebuffer is always complete
            Ok(JsValue::from(WebGLConstants::FRAMEBUFFER_COMPLETE))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // framebufferTexture2D
    obj.set(
        js_string!("framebufferTexture2D"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let attachment = args.get_or_undefined(1).to_u32(ctx)?;
            let tex_target = args.get_or_undefined(2).to_u32(ctx)?;
            let texture_id = get_object_id(args.get_or_undefined(3), ctx)?;
            let level = args.get_or_undefined(4).to_i32(ctx)?;

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get_mut(&fb_id) {
                    let face = if tex_target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                        && tex_target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                    {
                        Some(tex_target)
                    } else {
                        None
                    };
                    fb.attach_texture(attachment, texture_id, level, face);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // createRenderbuffer
    obj.set(
        js_string!("createRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context_data!(this => _ctx_obj, data);

            let rb = WebGLRenderbuffer::new();
            let id = rb.id;
            data.renderbuffers.lock().unwrap().insert(id, rb);

            let rb_obj = JsObject::with_null_proto();
            rb_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(rb_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindRenderbuffer
    obj.set(
        js_string!("bindRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let rb_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            data.state.lock().unwrap().bound_renderbuffer = rb_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // renderbufferStorage
    obj.set(
        js_string!("renderbufferStorage"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let internal_format = args.get_or_undefined(1).to_u32(ctx)?;
            let width = args.get_or_undefined(2).to_u32(ctx)?;
            let height = args.get_or_undefined(3).to_u32(ctx)?;

            let rb_id = data.state.lock().unwrap().bound_renderbuffer;

            if let Some(rb_id) = rb_id {
                if let Some(rb) = data.renderbuffers.lock().unwrap().get_mut(&rb_id) {
                    rb.storage(internal_format, width, height);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // framebufferRenderbuffer
    obj.set(
        js_string!("framebufferRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let attachment = args.get_or_undefined(1).to_u32(ctx)?;
            let _rb_target = args.get_or_undefined(2).to_u32(ctx)?;
            let rb_id = get_object_id(args.get_or_undefined(3), ctx)?;

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get_mut(&fb_id) {
                    fb.attach_renderbuffer(attachment, rb_id);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add uniform methods
fn add_uniform_methods(obj: &JsObject, context: &mut Context) {
    // uniform1f
    obj.set(
        js_string!("uniform1f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;

            data.uniform_values.lock().unwrap().insert(location_id, vec![x]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniform2f
    obj.set(
        js_string!("uniform2f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;

            data.uniform_values.lock().unwrap().insert(location_id, vec![x, y]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniform3f
    obj.set(
        js_string!("uniform3f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;
            let z = args.get_or_undefined(3).to_number(ctx)? as f32;

            data.uniform_values.lock().unwrap().insert(location_id, vec![x, y, z]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniform4f
    obj.set(
        js_string!("uniform4f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;
            let z = args.get_or_undefined(3).to_number(ctx)? as f32;
            let w = args.get_or_undefined(4).to_number(ctx)? as f32;

            data.uniform_values.lock().unwrap().insert(location_id, vec![x, y, z, w]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniform1i
    obj.set(
        js_string!("uniform1i"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_i32(ctx)? as f32;

            data.uniform_values.lock().unwrap().insert(location_id, vec![x]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniformMatrix4fv
    obj.set(
        js_string!("uniformMatrix4fv"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let _transpose = args.get_or_undefined(1).to_boolean();

            let value_arg = args.get_or_undefined(2);
            let mut values = Vec::with_capacity(16);

            if let Some(obj) = value_arg.as_object() {
                // Get length property
                if let Ok(len) = obj.get(js_string!("length"), ctx) {
                    let len = len.to_index(ctx).unwrap_or(0) as usize;
                    for i in 0..len.min(16) {
                        if let Ok(val) = obj.get(i as u32, ctx) {
                            values.push(val.to_number(ctx).unwrap_or(0.0) as f32);
                        }
                    }
                }
            }

            data.uniform_values.lock().unwrap().insert(location_id, values);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add vertex attribute methods
fn add_vertex_attrib_methods(obj: &JsObject, context: &mut Context) {
    // vertexAttribPointer
    obj.set(
        js_string!("vertexAttribPointer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;
            let size = args.get_or_undefined(1).to_i32(ctx)?;
            let data_type = args.get_or_undefined(2).to_u32(ctx)?;
            let normalized = args.get_or_undefined(3).to_boolean();
            let stride = args.get_or_undefined(4).to_i32(ctx)?;
            let offset = args.get_or_undefined(5).to_i32(ctx)?;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            let buffer_id = data.state.lock().unwrap().bound_array_buffer;

            let mut attribs = data.vertex_attribs.lock().unwrap();
            attribs[index].size = size;
            attribs[index].data_type = data_type;
            attribs[index].normalized = normalized;
            attribs[index].stride = stride;
            attribs[index].offset = offset;
            attribs[index].buffer_id = buffer_id;

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // enableVertexAttribArray
    obj.set(
        js_string!("enableVertexAttribArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            data.vertex_attribs.lock().unwrap()[index].enabled = true;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // disableVertexAttribArray
    obj.set(
        js_string!("disableVertexAttribArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            data.vertex_attribs.lock().unwrap()[index].enabled = false;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add draw methods
fn add_draw_methods(obj: &JsObject, context: &mut Context) {
    // clear
    obj.set(
        js_string!("clear"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let mask = args.get_or_undefined(0).to_u32(ctx)?;

            if mask & WebGLConstants::COLOR_BUFFER_BIT != 0 {
                let state = data.state.lock().unwrap();
                let clear_color = state.clear_color;
                drop(state);

                let mut render_target = data.render_target.lock().unwrap();
                let r = (clear_color[0] * 255.0) as u8;
                let g = (clear_color[1] * 255.0) as u8;
                let b = (clear_color[2] * 255.0) as u8;
                let a = (clear_color[3] * 255.0) as u8;

                for chunk in render_target.chunks_exact_mut(4) {
                    chunk[0] = r;
                    chunk[1] = g;
                    chunk[2] = b;
                    chunk[3] = a;
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // drawArrays
    obj.set(
        js_string!("drawArrays"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _mode = args.get_or_undefined(0).to_u32(ctx)?;
            let _first = args.get_or_undefined(1).to_i32(ctx)?;
            let _count = args.get_or_undefined(2).to_i32(ctx)?;

            // Validate state
            if data.state.lock().unwrap().current_program.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            // Drawing implementation would go here
            // For now, this is a no-op as we need full wgpu pipeline setup

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // drawElements
    obj.set(
        js_string!("drawElements"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let _mode = args.get_or_undefined(0).to_u32(ctx)?;
            let _count = args.get_or_undefined(1).to_i32(ctx)?;
            let _type_ = args.get_or_undefined(2).to_u32(ctx)?;
            let _offset = args.get_or_undefined(3).to_i32(ctx)?;

            // Validate state
            if data.state.lock().unwrap().current_program.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            if data.state.lock().unwrap().bound_element_array_buffer.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            // Drawing implementation would go here
            // For now, this is a no-op as we need full wgpu pipeline setup

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // flush
    obj.set(
        js_string!("flush"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // finish
    obj.set(
        js_string!("finish"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // readPixels
    obj.set(
        js_string!("readPixels"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;
            let _format = args.get_or_undefined(4).to_u32(ctx)?;
            let _type = args.get_or_undefined(5).to_u32(ctx)?;
            let pixels_arg = args.get_or_undefined(6);

            // Read from render target
            let render_target = data.render_target.lock().unwrap();
            let target_width = data.width as i32;
            let target_height = data.height as i32;

            if let Some(obj) = pixels_arg.as_object() {
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                        let byte_offset = obj.get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        // Copy pixel data
                        let data_slice = ab.data();
                        if let Some(data_guard) = data_slice {
                            let mut dst = data_guard.as_ref().to_vec();
                            for row in 0..height {
                                let src_y = target_height - 1 - (y + row); // Flip Y
                                if src_y < 0 || src_y >= target_height {
                                    continue;
                                }
                                for col in 0..width {
                                    let src_x = x + col;
                                    if src_x < 0 || src_x >= target_width {
                                        continue;
                                    }
                                    let src_idx = ((src_y * target_width + src_x) * 4) as usize;
                                    let dst_idx = byte_offset + ((row * width + col) * 4) as usize;
                                    if src_idx + 4 <= render_target.len() && dst_idx + 4 <= dst.len() {
                                        dst[dst_idx..dst_idx + 4].copy_from_slice(&render_target[src_idx..src_idx + 4]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add state methods
fn add_state_methods(obj: &JsObject, context: &mut Context) {
    // clearColor
    obj.set(
        js_string!("clearColor"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let r = args.get_or_undefined(0).to_number(ctx)? as f32;
            let g = args.get_or_undefined(1).to_number(ctx)? as f32;
            let b = args.get_or_undefined(2).to_number(ctx)? as f32;
            let a = args.get_or_undefined(3).to_number(ctx)? as f32;

            data.state.lock().unwrap().clear_color = [r, g, b, a];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // clearDepth
    obj.set(
        js_string!("clearDepth"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let depth = args.get_or_undefined(0).to_number(ctx)? as f32;

            data.state.lock().unwrap().clear_depth = depth;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // clearStencil
    obj.set(
        js_string!("clearStencil"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let stencil = args.get_or_undefined(0).to_i32(ctx)?;

            data.state.lock().unwrap().clear_stencil = stencil;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // enable
    obj.set(
        js_string!("enable"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            match cap {
                WebGLConstants::BLEND => state.blend = true,
                WebGLConstants::CULL_FACE => state.cull_face = true,
                WebGLConstants::DEPTH_TEST => state.depth_test = true,
                WebGLConstants::DITHER => state.dither = true,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill = true,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage = true,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage = true,
                WebGLConstants::SCISSOR_TEST => state.scissor_test = true,
                WebGLConstants::STENCIL_TEST => state.stencil_test = true,
                _ => {
                    drop(state);
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // disable
    obj.set(
        js_string!("disable"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            match cap {
                WebGLConstants::BLEND => state.blend = false,
                WebGLConstants::CULL_FACE => state.cull_face = false,
                WebGLConstants::DEPTH_TEST => state.depth_test = false,
                WebGLConstants::DITHER => state.dither = false,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill = false,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage = false,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage = false,
                WebGLConstants::SCISSOR_TEST => state.scissor_test = false,
                WebGLConstants::STENCIL_TEST => state.stencil_test = false,
                _ => {
                    drop(state);
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isEnabled
    obj.set(
        js_string!("isEnabled"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let state = data.state.lock().unwrap();
            let enabled = match cap {
                WebGLConstants::BLEND => state.blend,
                WebGLConstants::CULL_FACE => state.cull_face,
                WebGLConstants::DEPTH_TEST => state.depth_test,
                WebGLConstants::DITHER => state.dither,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage,
                WebGLConstants::SCISSOR_TEST => state.scissor_test,
                WebGLConstants::STENCIL_TEST => state.stencil_test,
                _ => false,
            };
            Ok(JsValue::from(enabled))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // viewport
    obj.set(
        js_string!("viewport"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;

            data.state.lock().unwrap().viewport = [x, y, width, height];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // scissor
    obj.set(
        js_string!("scissor"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;

            data.state.lock().unwrap().scissor = [x, y, width, height];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendFunc
    obj.set(
        js_string!("blendFunc"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let sfactor = args.get_or_undefined(0).to_u32(ctx)?;
            let dfactor = args.get_or_undefined(1).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_src_rgb = sfactor;
            state.blend_src_alpha = sfactor;
            state.blend_dst_rgb = dfactor;
            state.blend_dst_alpha = dfactor;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendFuncSeparate
    obj.set(
        js_string!("blendFuncSeparate"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let src_rgb = args.get_or_undefined(0).to_u32(ctx)?;
            let dst_rgb = args.get_or_undefined(1).to_u32(ctx)?;
            let src_alpha = args.get_or_undefined(2).to_u32(ctx)?;
            let dst_alpha = args.get_or_undefined(3).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_src_rgb = src_rgb;
            state.blend_dst_rgb = dst_rgb;
            state.blend_src_alpha = src_alpha;
            state.blend_dst_alpha = dst_alpha;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendEquation
    obj.set(
        js_string!("blendEquation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_equation_rgb = mode;
            state.blend_equation_alpha = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // depthFunc
    obj.set(
        js_string!("depthFunc"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let func = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().depth_func = func;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // depthMask
    obj.set(
        js_string!("depthMask"),
        NativeFunction::from_fn_ptr(|this, args, _ctx| {
            with_context_data!(this => _ctx_obj, data);
            let flag = args.get_or_undefined(0).to_boolean();

            data.state.lock().unwrap().depth_mask = flag;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // colorMask
    obj.set(
        js_string!("colorMask"),
        NativeFunction::from_fn_ptr(|this, args, _ctx| {
            with_context_data!(this => _ctx_obj, data);
            let r = args.get_or_undefined(0).to_boolean();
            let g = args.get_or_undefined(1).to_boolean();
            let b = args.get_or_undefined(2).to_boolean();
            let a = args.get_or_undefined(3).to_boolean();

            data.state.lock().unwrap().color_mask = [r, g, b, a];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // cullFace
    obj.set(
        js_string!("cullFace"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().cull_face_mode = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // frontFace
    obj.set(
        js_string!("frontFace"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().front_face = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // lineWidth
    obj.set(
        js_string!("lineWidth"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let width = args.get_or_undefined(0).to_number(ctx)? as f32;

            data.state.lock().unwrap().line_width = width;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // pixelStorei
    obj.set(
        js_string!("pixelStorei"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context_data!(this => _ctx_obj, data);
            let pname = args.get_or_undefined(0).to_u32(ctx)?;
            let param = args.get_or_undefined(1);

            let mut state = data.state.lock().unwrap();
            match pname {
                WebGLConstants::UNPACK_FLIP_Y_WEBGL => {
                    state.unpack_flip_y = param.to_boolean();
                }
                WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                    state.unpack_premultiply_alpha = param.to_boolean();
                }
                WebGLConstants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                    state.unpack_colorspace_conversion = param.to_u32(ctx).unwrap_or(0);
                }
                WebGLConstants::UNPACK_ALIGNMENT => {
                    state.unpack_alignment = param.to_u32(ctx).unwrap_or(4);
                }
                WebGLConstants::PACK_ALIGNMENT => {
                    state.pack_alignment = param.to_u32(ctx).unwrap_or(4);
                }
                _ => {}
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Helper function to get object ID
fn get_object_id(val: &JsValue, ctx: &mut Context) -> JsResult<u32> {
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
fn get_location_id(val: &JsValue, ctx: &mut Context) -> JsResult<u32> {
    get_object_id(val, ctx)
}

/// Get parameter value
fn get_parameter(data: &WebGLRenderingContextData, pname: u32, ctx: &mut Context) -> JsResult<JsValue> {
    let state = data.state.lock().unwrap();

    match pname {
        WebGLConstants::VENDOR => Ok(JsValue::from(js_string!("Thalora"))),
        WebGLConstants::RENDERER => Ok(JsValue::from(js_string!("Thalora WebGL"))),
        WebGLConstants::VERSION => Ok(JsValue::from(js_string!("WebGL 1.0 (Thalora)"))),
        WebGLConstants::SHADING_LANGUAGE_VERSION => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0"))),
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
        WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => Ok(JsValue::from(state.unpack_premultiply_alpha)),
        _ => Ok(JsValue::null()),
    }
}
