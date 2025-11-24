//! WebGL2RenderingContext Implementation
//!
//! Provides the WebGL 2.0 rendering context, extending WebGL 1.0 with additional features.
//! https://www.khronos.org/webgl/

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use boa_engine::{
    js_string,
    object::Object,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};

use super::buffer::{VertexAttribArray, WebGLBuffer, WebGLFramebuffer, WebGLRenderbuffer};
use super::shader::{WebGLProgram, WebGLShader};
use super::state::{WebGL2Constants, WebGLConstants, WebGLState};
use super::texture::WebGLTexture;
use super::webgl_rendering_context::WebGLRenderingContextData;

/// Maximum number of vertex attributes for WebGL2
const MAX_VERTEX_ATTRIBS: usize = 16;

/// WebGL2 Sync object
#[derive(Debug, Clone)]
pub struct WebGLSync {
    pub id: u32,
    pub signaled: bool,
}

/// WebGL2 Query object
#[derive(Debug, Clone)]
pub struct WebGLQuery {
    pub id: u32,
    pub target: u32,
    pub result: Option<u64>,
}

/// WebGL2 Sampler object
#[derive(Debug, Clone)]
pub struct WebGLSampler {
    pub id: u32,
    pub min_filter: u32,
    pub mag_filter: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub wrap_r: u32,
    pub min_lod: f32,
    pub max_lod: f32,
    pub compare_mode: u32,
    pub compare_func: u32,
}

/// WebGL2 Transform Feedback object
#[derive(Debug, Clone)]
pub struct WebGLTransformFeedback {
    pub id: u32,
    pub active: bool,
    pub paused: bool,
}

/// WebGL2 Vertex Array Object
#[derive(Debug, Clone)]
pub struct WebGLVertexArrayObject {
    pub id: u32,
    pub attribs: [VertexAttribArray; MAX_VERTEX_ATTRIBS],
    pub element_array_buffer: Option<u32>,
}

/// WebGL2 context data (extends WebGL1)
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebGL2RenderingContextData {
    /// Base WebGL1 context data
    #[unsafe_ignore_trace]
    pub base: WebGLRenderingContextData,
    /// Sync objects
    #[unsafe_ignore_trace]
    pub syncs: Arc<Mutex<HashMap<u32, WebGLSync>>>,
    /// Query objects
    #[unsafe_ignore_trace]
    pub queries: Arc<Mutex<HashMap<u32, WebGLQuery>>>,
    /// Sampler objects
    #[unsafe_ignore_trace]
    pub samplers: Arc<Mutex<HashMap<u32, WebGLSampler>>>,
    /// Transform feedback objects
    #[unsafe_ignore_trace]
    pub transform_feedbacks: Arc<Mutex<HashMap<u32, WebGLTransformFeedback>>>,
    /// Vertex Array Objects
    #[unsafe_ignore_trace]
    pub vertex_array_objects: Arc<Mutex<HashMap<u32, WebGLVertexArrayObject>>>,
    /// Current bound VAO
    #[unsafe_ignore_trace]
    pub current_vao: Arc<Mutex<Option<u32>>>,
    /// Current transform feedback
    #[unsafe_ignore_trace]
    pub current_transform_feedback: Arc<Mutex<Option<u32>>>,
    /// Uniform buffer bindings
    #[unsafe_ignore_trace]
    pub uniform_buffer_bindings: Arc<Mutex<HashMap<u32, Option<u32>>>>,
}

impl WebGL2RenderingContextData {
    /// Create new WebGL2 context data
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            base: WebGLRenderingContextData::new(width, height),
            syncs: Arc::new(Mutex::new(HashMap::new())),
            queries: Arc::new(Mutex::new(HashMap::new())),
            samplers: Arc::new(Mutex::new(HashMap::new())),
            transform_feedbacks: Arc::new(Mutex::new(HashMap::new())),
            vertex_array_objects: Arc::new(Mutex::new(HashMap::new())),
            current_vao: Arc::new(Mutex::new(None)),
            current_transform_feedback: Arc::new(Mutex::new(None)),
            uniform_buffer_bindings: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Helper macro to get WebGL2 context data from this
/// This must be a macro because we need to keep the JsObject in the caller's scope
/// for the GcRef borrow to be valid
macro_rules! with_context2_data {
    ($this:expr => $obj:ident, $data:ident) => {
        let $obj = $this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Not a WebGL2RenderingContext")
        })?;
        let $data = $obj.downcast_ref::<WebGL2RenderingContextData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Not a WebGL2RenderingContext")
        })?;
    };
}

/// WebGL2RenderingContext JavaScript class
pub struct WebGL2RenderingContext;

impl WebGL2RenderingContext {
    /// Initialize the WebGL2RenderingContext in the realm
    /// WebGL2 contexts are created dynamically via canvas.getContext("webgl2")
    pub fn init(_realm: &Realm) {
        // WebGL2 contexts are created on-demand, not as global constructors
        // The actual context creation happens in create_context()
    }

    /// Create a new WebGL2 context for a canvas
    pub fn create_context(width: u32, height: u32, context: &mut Context) -> JsResult<JsObject> {
        let data = WebGL2RenderingContextData::new(width, height);

        // Create the context object with data
        let obj = JsObject::from_proto_and_data(None, data);

        // Add all WebGL2 constants
        add_webgl2_constants(&obj, context);

        // Add WebGL1 methods (inherited)
        super::webgl_rendering_context::WebGLRenderingContext::init(&context.realm().clone());

        // Add WebGL2 methods
        add_vao_methods(&obj, context);
        add_buffer2_methods(&obj, context);
        add_texture2_methods(&obj, context);
        add_query_methods(&obj, context);
        add_sampler_methods(&obj, context);
        add_sync_methods(&obj, context);
        add_transform_feedback_methods(&obj, context);
        add_uniform_buffer_methods(&obj, context);

        // Add canvas property
        obj.set(js_string!("canvas"), JsValue::null(), false, context)?;

        // Add drawing buffer properties
        obj.set(js_string!("drawingBufferWidth"), JsValue::from(width), false, context)?;
        obj.set(js_string!("drawingBufferHeight"), JsValue::from(height), false, context)?;

        Ok(obj)
    }
}

/// Add WebGL2 constants
fn add_webgl2_constants(obj: &JsObject, context: &mut Context) {
    let constants = [
        // WebGL2 buffer targets
        ("COPY_READ_BUFFER", WebGL2Constants::COPY_READ_BUFFER),
        ("COPY_WRITE_BUFFER", WebGL2Constants::COPY_WRITE_BUFFER),
        ("TRANSFORM_FEEDBACK_BUFFER", WebGL2Constants::TRANSFORM_FEEDBACK_BUFFER),
        ("UNIFORM_BUFFER", WebGL2Constants::UNIFORM_BUFFER),
        ("PIXEL_PACK_BUFFER", WebGL2Constants::PIXEL_PACK_BUFFER),
        ("PIXEL_UNPACK_BUFFER", WebGL2Constants::PIXEL_UNPACK_BUFFER),
        // Buffer usage
        ("STREAM_READ", WebGL2Constants::STREAM_READ),
        ("STREAM_COPY", WebGL2Constants::STREAM_COPY),
        ("STATIC_READ", WebGL2Constants::STATIC_READ),
        ("STATIC_COPY", WebGL2Constants::STATIC_COPY),
        ("DYNAMIC_READ", WebGL2Constants::DYNAMIC_READ),
        ("DYNAMIC_COPY", WebGL2Constants::DYNAMIC_COPY),
        // Draw buffers
        ("DRAW_BUFFER0", WebGL2Constants::DRAW_BUFFER0),
        ("DRAW_BUFFER1", WebGL2Constants::DRAW_BUFFER1),
        ("DRAW_BUFFER2", WebGL2Constants::DRAW_BUFFER2),
        ("DRAW_BUFFER3", WebGL2Constants::DRAW_BUFFER3),
        ("MAX_DRAW_BUFFERS", WebGL2Constants::MAX_DRAW_BUFFERS),
        ("MAX_COLOR_ATTACHMENTS", WebGL2Constants::MAX_COLOR_ATTACHMENTS),
        // Internal formats
        ("R8", WebGL2Constants::R8),
        ("RG8", WebGL2Constants::RG8),
        ("R16F", WebGL2Constants::R16F),
        ("R32F", WebGL2Constants::R32F),
        ("RG16F", WebGL2Constants::RG16F),
        ("RG32F", WebGL2Constants::RG32F),
        ("RGBA32F", WebGL2Constants::RGBA32F),
        ("RGB32F", WebGL2Constants::RGB32F),
        ("RGBA16F", WebGL2Constants::RGBA16F),
        ("RGB16F", WebGL2Constants::RGB16F),
        ("DEPTH_COMPONENT24", WebGL2Constants::DEPTH_COMPONENT24),
        ("DEPTH_COMPONENT32F", WebGL2Constants::DEPTH_COMPONENT32F),
        ("DEPTH24_STENCIL8", WebGL2Constants::DEPTH24_STENCIL8),
        ("DEPTH32F_STENCIL8", WebGL2Constants::DEPTH32F_STENCIL8),
        // Pixel formats
        ("RED", WebGL2Constants::RED),
        ("RG", WebGL2Constants::RG),
        ("RED_INTEGER", WebGL2Constants::RED_INTEGER),
        ("RG_INTEGER", WebGL2Constants::RG_INTEGER),
        ("RGB_INTEGER", WebGL2Constants::RGB_INTEGER),
        ("RGBA_INTEGER", WebGL2Constants::RGBA_INTEGER),
        // Textures
        ("TEXTURE_3D", WebGL2Constants::TEXTURE_3D),
        ("TEXTURE_2D_ARRAY", WebGL2Constants::TEXTURE_2D_ARRAY),
        ("TEXTURE_WRAP_R", WebGL2Constants::TEXTURE_WRAP_R),
        // Samplers
        ("SAMPLER_3D", WebGL2Constants::SAMPLER_3D),
        ("SAMPLER_2D_SHADOW", WebGL2Constants::SAMPLER_2D_SHADOW),
        ("SAMPLER_2D_ARRAY", WebGL2Constants::SAMPLER_2D_ARRAY),
        ("SAMPLER_2D_ARRAY_SHADOW", WebGL2Constants::SAMPLER_2D_ARRAY_SHADOW),
        ("SAMPLER_CUBE_SHADOW", WebGL2Constants::SAMPLER_CUBE_SHADOW),
        ("INT_SAMPLER_2D", WebGL2Constants::INT_SAMPLER_2D),
        ("INT_SAMPLER_3D", WebGL2Constants::INT_SAMPLER_3D),
        ("INT_SAMPLER_CUBE", WebGL2Constants::INT_SAMPLER_CUBE),
        ("INT_SAMPLER_2D_ARRAY", WebGL2Constants::INT_SAMPLER_2D_ARRAY),
        ("UNSIGNED_INT_SAMPLER_2D", WebGL2Constants::UNSIGNED_INT_SAMPLER_2D),
        ("UNSIGNED_INT_SAMPLER_3D", WebGL2Constants::UNSIGNED_INT_SAMPLER_3D),
        ("UNSIGNED_INT_SAMPLER_CUBE", WebGL2Constants::UNSIGNED_INT_SAMPLER_CUBE),
        ("UNSIGNED_INT_SAMPLER_2D_ARRAY", WebGL2Constants::UNSIGNED_INT_SAMPLER_2D_ARRAY),
        // Transform feedback
        ("TRANSFORM_FEEDBACK", WebGL2Constants::TRANSFORM_FEEDBACK),
        ("TRANSFORM_FEEDBACK_BINDING", WebGL2Constants::TRANSFORM_FEEDBACK_BINDING),
        ("TRANSFORM_FEEDBACK_ACTIVE", WebGL2Constants::TRANSFORM_FEEDBACK_ACTIVE),
        ("TRANSFORM_FEEDBACK_PAUSED", WebGL2Constants::TRANSFORM_FEEDBACK_PAUSED),
        // Sync
        ("SYNC_GPU_COMMANDS_COMPLETE", WebGL2Constants::SYNC_GPU_COMMANDS_COMPLETE),
        ("ALREADY_SIGNALED", WebGL2Constants::ALREADY_SIGNALED),
        ("TIMEOUT_EXPIRED", WebGL2Constants::TIMEOUT_EXPIRED),
        ("CONDITION_SATISFIED", WebGL2Constants::CONDITION_SATISFIED),
        ("WAIT_FAILED", WebGL2Constants::WAIT_FAILED),
        ("SYNC_FLUSH_COMMANDS_BIT", WebGL2Constants::SYNC_FLUSH_COMMANDS_BIT),
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

/// Add VAO methods
fn add_vao_methods(obj: &JsObject, context: &mut Context) {
    static VAO_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createVertexArray
    obj.set(
        js_string!("createVertexArray"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let vao = WebGLVertexArrayObject {
                id: VAO_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                attribs: std::array::from_fn(|_| VertexAttribArray::default()),
                element_array_buffer: None,
            };
            let id = vao.id;
            data.vertex_array_objects.lock().unwrap().insert(id, vao);

            let vao_obj = JsObject::with_null_proto();
            vao_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(vao_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindVertexArray
    obj.set(
        js_string!("bindVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let vao_id = if args.get_or_undefined(0).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(0), ctx)?)
            };

            *data.current_vao.lock().unwrap() = vao_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteVertexArray
    obj.set(
        js_string!("deleteVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let vao_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.vertex_array_objects.lock().unwrap().remove(&vao_id);

            // Unbind if currently bound
            let mut current = data.current_vao.lock().unwrap();
            if *current == Some(vao_id) {
                *current = None;
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isVertexArray
    obj.set(
        js_string!("isVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let vao_id = get_object_id(args.get_or_undefined(0), ctx)?;

            let exists = data.vertex_array_objects.lock().unwrap().contains_key(&vao_id);
            Ok(JsValue::from(exists))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add WebGL2 buffer methods
fn add_buffer2_methods(obj: &JsObject, context: &mut Context) {
    // copyBufferSubData
    obj.set(
        js_string!("copyBufferSubData"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Implementation would copy data between buffers
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getBufferSubData
    obj.set(
        js_string!("getBufferSubData"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Implementation would read buffer data back
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add WebGL2 texture methods
fn add_texture2_methods(obj: &JsObject, context: &mut Context) {
    // texImage3D
    obj.set(
        js_string!("texImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // 3D texture implementation
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // texSubImage3D
    obj.set(
        js_string!("texSubImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // compressedTexImage3D
    obj.set(
        js_string!("compressedTexImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // texStorage2D
    obj.set(
        js_string!("texStorage2D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // texStorage3D
    obj.set(
        js_string!("texStorage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add query methods
fn add_query_methods(obj: &JsObject, context: &mut Context) {
    static QUERY_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createQuery
    obj.set(
        js_string!("createQuery"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let query = WebGLQuery {
                id: QUERY_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                target: 0,
                result: None,
            };
            let id = query.id;
            data.queries.lock().unwrap().insert(id, query);

            let query_obj = JsObject::with_null_proto();
            query_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(query_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteQuery
    obj.set(
        js_string!("deleteQuery"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let query_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.queries.lock().unwrap().remove(&query_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // beginQuery
    obj.set(
        js_string!("beginQuery"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let query_id = get_object_id(args.get_or_undefined(1), ctx)?;

            if let Some(query) = data.queries.lock().unwrap().get_mut(&query_id) {
                query.target = target;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // endQuery
    obj.set(
        js_string!("endQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getQueryParameter
    obj.set(
        js_string!("getQueryParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let query_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(query) = data.queries.lock().unwrap().get(&query_id) {
                if let Some(result) = query.result {
                    return Ok(JsValue::from(result as f64));
                }
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add sampler methods
fn add_sampler_methods(obj: &JsObject, context: &mut Context) {
    static SAMPLER_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createSampler
    obj.set(
        js_string!("createSampler"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let sampler = WebGLSampler {
                id: SAMPLER_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                min_filter: WebGLConstants::NEAREST_MIPMAP_LINEAR,
                mag_filter: WebGLConstants::LINEAR,
                wrap_s: WebGLConstants::REPEAT,
                wrap_t: WebGLConstants::REPEAT,
                wrap_r: WebGLConstants::REPEAT,
                min_lod: -1000.0,
                max_lod: 1000.0,
                compare_mode: 0,
                compare_func: WebGLConstants::LEQUAL,
            };
            let id = sampler.id;
            data.samplers.lock().unwrap().insert(id, sampler);

            let sampler_obj = JsObject::with_null_proto();
            sampler_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(sampler_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteSampler
    obj.set(
        js_string!("deleteSampler"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let sampler_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.samplers.lock().unwrap().remove(&sampler_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindSampler
    obj.set(
        js_string!("bindSampler"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // samplerParameteri
    obj.set(
        js_string!("samplerParameteri"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let sampler_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;
            let param = args.get_or_undefined(2).to_u32(ctx)?;

            if let Some(sampler) = data.samplers.lock().unwrap().get_mut(&sampler_id) {
                match pname {
                    WebGLConstants::TEXTURE_MIN_FILTER => sampler.min_filter = param,
                    WebGLConstants::TEXTURE_MAG_FILTER => sampler.mag_filter = param,
                    WebGLConstants::TEXTURE_WRAP_S => sampler.wrap_s = param,
                    WebGLConstants::TEXTURE_WRAP_T => sampler.wrap_t = param,
                    WebGL2Constants::TEXTURE_WRAP_R => sampler.wrap_r = param,
                    _ => {}
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add sync methods
fn add_sync_methods(obj: &JsObject, context: &mut Context) {
    static SYNC_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // fenceSync
    obj.set(
        js_string!("fenceSync"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let sync = WebGLSync {
                id: SYNC_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                signaled: false,
            };
            let id = sync.id;
            data.syncs.lock().unwrap().insert(id, sync);

            let sync_obj = JsObject::with_null_proto();
            sync_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(sync_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteSync
    obj.set(
        js_string!("deleteSync"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.syncs.lock().unwrap().remove(&sync_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // clientWaitSync
    obj.set(
        js_string!("clientWaitSync"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _flags = args.get_or_undefined(1).to_u32(ctx)?;
            let _timeout = args.get_or_undefined(2).to_number(ctx)?;

            if let Some(sync) = data.syncs.lock().unwrap().get(&sync_id) {
                if sync.signaled {
                    return Ok(JsValue::from(WebGL2Constants::ALREADY_SIGNALED));
                }
            }
            // For now, always return condition satisfied
            Ok(JsValue::from(WebGL2Constants::CONDITION_SATISFIED))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // waitSync
    obj.set(
        js_string!("waitSync"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // getSyncParameter
    obj.set(
        js_string!("getSyncParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(sync) = data.syncs.lock().unwrap().get(&sync_id) {
                if sync.signaled {
                    return Ok(JsValue::from(1));
                }
            }
            Ok(JsValue::from(0))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add transform feedback methods
fn add_transform_feedback_methods(obj: &JsObject, context: &mut Context) {
    static TF_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createTransformFeedback
    obj.set(
        js_string!("createTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let tf = WebGLTransformFeedback {
                id: TF_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                active: false,
                paused: false,
            };
            let id = tf.id;
            data.transform_feedbacks.lock().unwrap().insert(id, tf);

            let tf_obj = JsObject::with_null_proto();
            tf_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(tf_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteTransformFeedback
    obj.set(
        js_string!("deleteTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let tf_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.transform_feedbacks.lock().unwrap().remove(&tf_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindTransformFeedback
    obj.set(
        js_string!("bindTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let tf_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            *data.current_transform_feedback.lock().unwrap() = tf_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // beginTransformFeedback
    obj.set(
        js_string!("beginTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.active = true;
                    tf.paused = false;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // endTransformFeedback
    obj.set(
        js_string!("endTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.active = false;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // pauseTransformFeedback
    obj.set(
        js_string!("pauseTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.paused = true;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // resumeTransformFeedback
    obj.set(
        js_string!("resumeTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_context2_data!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.paused = false;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add uniform buffer methods
fn add_uniform_buffer_methods(obj: &JsObject, context: &mut Context) {
    // getUniformBlockIndex
    obj.set(
        js_string!("getUniformBlockIndex"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Would return the index of a uniform block
            Ok(JsValue::from(0))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // uniformBlockBinding
    obj.set(
        js_string!("uniformBlockBinding"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindBufferBase
    obj.set(
        js_string!("bindBufferBase"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let index = args.get_or_undefined(1).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(2).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(2), ctx)?)
            };

            data.uniform_buffer_bindings.lock().unwrap().insert(index, buffer_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindBufferRange
    obj.set(
        js_string!("bindBufferRange"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_context2_data!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let index = args.get_or_undefined(1).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(2).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(2), ctx)?)
            };

            data.uniform_buffer_bindings.lock().unwrap().insert(index, buffer_id);
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
