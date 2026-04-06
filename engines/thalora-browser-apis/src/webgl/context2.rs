//! WebGL2 Context Types and Data Structures
//!
//! Core types, context data, and the WebGL2RenderingContext class.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use boa_engine::{
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::{FunctionObjectBuilder, ObjectInitializer},
    realm::Realm,
};
use boa_gc::{Finalize, Trace};

use super::buffer::VertexAttribArray;
use super::context::WebGLRenderingContextData;

/// Maximum number of vertex attributes for WebGL2
pub const MAX_VERTEX_ATTRIBS: usize = 16;

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
#[macro_export]
macro_rules! with_webgl2_context {
    ($this:expr => $obj:ident, $data:ident) => {
        let $obj = $this.as_object().ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("Not a WebGL2RenderingContext")
        })?;
        let $data = $obj
            .downcast_ref::<$crate::webgl::context2::WebGL2RenderingContextData>()
            .ok_or_else(|| {
                boa_engine::JsNativeError::typ().with_message("Not a WebGL2RenderingContext")
            })?;
    };
}

/// Helper function to get object ID
pub fn get_object_id(val: &JsValue, ctx: &mut Context) -> JsResult<u32> {
    if val.is_null() || val.is_undefined() {
        return Ok(0);
    }

    if let Some(obj) = val.as_object()
        && let Ok(id_val) = obj.get(js_string!("_id"), ctx)
    {
        return id_val.to_u32(ctx);
    }

    Ok(0)
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

    /// Create the global WebGL2RenderingContext constructor for global registration
    /// Per Web spec, WebGL2RenderingContext IS a global constructor with static constants
    pub fn create_global_constructor(context: &mut Context) -> JsResult<JsObject> {
        // The constructor itself throws when called - contexts are created via getContext()
        fn webgl2_constructor(
            _this: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> JsResult<JsValue> {
            Err(JsNativeError::typ()
                .with_message("WebGL2RenderingContext cannot be directly constructed; use canvas.getContext('webgl2')")
                .into())
        }

        // Create the prototype object with all methods
        let prototype = ObjectInitializer::new(context).build();

        // Add WebGL2 methods to prototype
        super::methods2_vao::add_vao_methods(&prototype, context);
        super::methods2_vao::add_buffer2_methods(&prototype, context);
        super::methods2_vao::add_texture2_methods(&prototype, context);
        super::methods2_query::add_query_methods(&prototype, context);
        super::methods2_query::add_sampler_methods(&prototype, context);
        super::methods2_query::add_sync_methods(&prototype, context);
        super::methods2_transform::add_transform_feedback_methods(&prototype, context);
        super::methods2_transform::add_uniform_buffer_methods(&prototype, context);

        // Create constructor function
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(webgl2_constructor),
        )
        .name(js_string!("WebGL2RenderingContext"))
        .length(0)
        .constructor(true)
        .build();

        // Add all WebGL2 constants to the constructor (static properties)
        super::constants2::add_webgl2_constants(&constructor.clone().into(), context);

        // Set prototype
        constructor.set(js_string!("prototype"), prototype, false, context)?;

        Ok(constructor.into())
    }

    /// Create a new WebGL2 context for a canvas
    pub fn create_context(width: u32, height: u32, context: &mut Context) -> JsResult<JsObject> {
        let data = WebGL2RenderingContextData::new(width, height);

        // Create the context object with data
        let obj = JsObject::from_proto_and_data(None, data);

        // Add all WebGL2 constants
        super::constants2::add_webgl2_constants(&obj, context);

        // Add WebGL1 methods (inherited)
        super::context::WebGLRenderingContext::init(&context.realm().clone());

        // Add WebGL2 methods
        super::methods2_vao::add_vao_methods(&obj, context);
        super::methods2_vao::add_buffer2_methods(&obj, context);
        super::methods2_vao::add_texture2_methods(&obj, context);
        super::methods2_query::add_query_methods(&obj, context);
        super::methods2_query::add_sampler_methods(&obj, context);
        super::methods2_query::add_sync_methods(&obj, context);
        super::methods2_transform::add_transform_feedback_methods(&obj, context);
        super::methods2_transform::add_uniform_buffer_methods(&obj, context);

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
