use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string};
use std::collections::HashMap;
use std::hash::Hasher;

/// WebGL implementation for realistic canvas fingerprinting and 3D graphics support
pub struct WebGLManager {
    contexts: HashMap<String, WebGLContext>,
}

#[derive(Debug, Clone)]
pub struct WebGLContext {
    pub version: String,
    pub renderer: String,
    pub vendor: String,
    pub extensions: Vec<String>,
    pub parameters: HashMap<String, JsValue>,
}

impl WebGLManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Setup WebGL context with realistic browser fingerprints
    pub fn setup_webgl_context(&self, context: &mut Context, canvas_element: &JsObject) -> Result<(), boa_engine::JsError> {
        // Enhanced getContext method that supports WebGL
        let get_context_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let context_type = args[0].to_string(context)?.to_std_string_escaped();

            match context_type.as_str() {
                "2d" => {
                    Self::create_2d_context(context)
                },
                "webgl" | "experimental-webgl" => {
                    Self::create_webgl_context(context, false)
                },
                "webgl2" => {
                    Self::create_webgl_context(context, true)
                },
                _ => Ok(JsValue::null())
            }
        }) };

        canvas_element.set(js_string!("getContext"), JsValue::from(get_context_fn.to_js_function(context.realm())), false, context)?;
        Ok(())
    }

    fn create_2d_context(context: &mut Context) -> Result<JsValue, boa_engine::JsError> {
        let ctx_2d = JsObject::default();

        // Basic 2D context methods
        ctx_2d.set(js_string!("fillStyle"), JsValue::from(js_string!("#000000")), true, context)?;
        ctx_2d.set(js_string!("strokeStyle"), JsValue::from(js_string!("#000000")), true, context)?;
        ctx_2d.set(js_string!("lineWidth"), JsValue::from(1.0), true, context)?;
        ctx_2d.set(js_string!("font"), JsValue::from(js_string!("10px sans-serif")), true, context)?;

        // Drawing methods
        let fill_rect_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        ctx_2d.set(js_string!("fillRect"), JsValue::from(fill_rect_fn.to_js_function(context.realm())), false, context)?;

        let fill_text_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        ctx_2d.set(js_string!("fillText"), JsValue::from(fill_text_fn.to_js_function(context.realm())), false, context)?;

        Ok(JsValue::from(ctx_2d))
    }

    fn create_webgl_context(context: &mut Context, is_webgl2: bool) -> Result<JsValue, boa_engine::JsError> {
        let gl_context = JsObject::default();

        // WebGL constants (subset of most commonly used)
        gl_context.set(js_string!("VERTEX_SHADER"), JsValue::from(35633), false, context)?;
        gl_context.set(js_string!("FRAGMENT_SHADER"), JsValue::from(35632), false, context)?;
        gl_context.set(js_string!("ARRAY_BUFFER"), JsValue::from(34962), false, context)?;
        gl_context.set(js_string!("STATIC_DRAW"), JsValue::from(35044), false, context)?;
        gl_context.set(js_string!("COLOR_BUFFER_BIT"), JsValue::from(16384), false, context)?;
        gl_context.set(js_string!("TRIANGLES"), JsValue::from(4), false, context)?;
        gl_context.set(js_string!("FLOAT"), JsValue::from(5126), false, context)?;

        // Core WebGL methods
        let create_shader_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            let shader_obj = JsObject::default();
            Ok(JsValue::from(shader_obj))
        }) };
        gl_context.set(js_string!("createShader"), JsValue::from(create_shader_fn.to_js_function(context.realm())), false, context)?;

        let create_program_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            let program_obj = JsObject::default();
            Ok(JsValue::from(program_obj))
        }) };
        gl_context.set(js_string!("createProgram"), JsValue::from(create_program_fn.to_js_function(context.realm())), false, context)?;

        let create_buffer_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            let buffer_obj = JsObject::default();
            Ok(JsValue::from(buffer_obj))
        }) };
        gl_context.set(js_string!("createBuffer"), JsValue::from(create_buffer_fn.to_js_function(context.realm())), false, context)?;

        // Shader operations
        let shader_source_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("shaderSource"), JsValue::from(shader_source_fn.to_js_function(context.realm())), false, context)?;

        let compile_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("compileShader"), JsValue::from(compile_shader_fn.to_js_function(context.realm())), false, context)?;

        // Program operations
        let attach_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("attachShader"), JsValue::from(attach_shader_fn.to_js_function(context.realm())), false, context)?;

        let link_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("linkProgram"), JsValue::from(link_program_fn.to_js_function(context.realm())), false, context)?;

        let use_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("useProgram"), JsValue::from(use_program_fn.to_js_function(context.realm())), false, context)?;

        // Buffer operations
        let bind_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("bindBuffer"), JsValue::from(bind_buffer_fn.to_js_function(context.realm())), false, context)?;

        let buffer_data_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("bufferData"), JsValue::from(buffer_data_fn.to_js_function(context.realm())), false, context)?;

        // Rendering operations
        let viewport_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("viewport"), JsValue::from(viewport_fn.to_js_function(context.realm())), false, context)?;

        let clear_color_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("clearColor"), JsValue::from(clear_color_fn.to_js_function(context.realm())), false, context)?;

        let clear_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("clear"), JsValue::from(clear_fn.to_js_function(context.realm())), false, context)?;

        let draw_arrays_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
        gl_context.set(js_string!("drawArrays"), JsValue::from(draw_arrays_fn.to_js_function(context.realm())), false, context)?;

        // Critical fingerprinting methods
        let get_parameter_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let param = args[0].to_i32(context)?;

            match param {
                7936 => Ok(JsValue::from(js_string!("WebKit"))), // GL_VENDOR
                7937 => Ok(JsValue::from(js_string!("WebKit WebGL"))), // GL_RENDERER
                7938 => Ok(JsValue::from(js_string!("WebGL 1.0 (OpenGL ES 2.0 Chromium)"))), // GL_VERSION
                35724 => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"))), // GL_SHADING_LANGUAGE_VERSION
                3379 => Ok(JsValue::from(16384)), // GL_MAX_TEXTURE_SIZE
                3386 => Ok(JsValue::from(30)), // GL_MAX_VIEWPORT_DIMS
                34024 => Ok(JsValue::from(16)), // GL_MAX_TEXTURE_IMAGE_UNITS
                _ => Ok(JsValue::null())
            }
        }) };
        gl_context.set(js_string!("getParameter"), JsValue::from(get_parameter_fn.to_js_function(context.realm())), false, context)?;

        let get_supported_extensions_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            let extensions = vec![
                "ANGLE_instanced_arrays",
                "EXT_blend_minmax",
                "EXT_color_buffer_half_float",
                "WEBGL_debug_renderer_info",
                "WEBGL_lose_context"
            ];

            let array = JsObject::default();
            array.set(js_string!("length"), JsValue::from(extensions.len()), false, context)?;
            for (i, ext) in extensions.iter().enumerate() {
                array.set(i, JsValue::from(js_string!(*ext)), false, context)?;
            }

            Ok(JsValue::from(array))
        }) };
        gl_context.set(js_string!("getSupportedExtensions"), JsValue::from(get_supported_extensions_fn.to_js_function(context.realm())), false, context)?;

        let get_extension_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let ext_name = args[0].to_string(context)?.to_std_string_escaped();

            // MOCK: Returns fake extension objects - no real GPU hardware access
            match ext_name.as_str() {
                "WEBGL_debug_renderer_info" => {
                    // MOCK: Hardcoded GL constants for debugging extension
                    let ext_obj = JsObject::default();
                    ext_obj.set(js_string!("UNMASKED_VENDOR_WEBGL"), JsValue::from(37445), false, context)?; // MOCK GL constant
                    ext_obj.set(js_string!("UNMASKED_RENDERER_WEBGL"), JsValue::from(37446), false, context)?; // MOCK GL constant
                    Ok(JsValue::from(ext_obj))
                },
                "WEBGL_lose_context" => {
                    // MOCK: Fake context loss extension - no real GPU interaction
                    let ext_obj = JsObject::default();
                    let lose_context_fn = NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())); // MOCK: No-op function
                    ext_obj.set(js_string!("loseContext"), JsValue::from(lose_context_fn.to_js_function(context.realm())), false, context)?;
                    Ok(JsValue::from(ext_obj))
                },
                _ => Ok(JsValue::null()) // MOCK: All other extensions return null
            }
        }) };
        gl_context.set(js_string!("getExtension"), JsValue::from(get_extension_fn.to_js_function(context.realm())), false, context)?;

        // Texture methods
        let create_texture_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
            let texture_obj = JsObject::default();
            Ok(JsValue::from(texture_obj))
        }) };
        gl_context.set(js_string!("createTexture"), JsValue::from(create_texture_fn.to_js_function(context.realm())), false, context)?;

        // Additional WebGL2 methods if requested
        if is_webgl2 {
            let create_vertex_array_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                let vao_obj = JsObject::default();
                Ok(JsValue::from(vao_obj))
            }) };
            gl_context.set(js_string!("createVertexArray"), JsValue::from(create_vertex_array_fn.to_js_function(context.realm())), false, context)?;

            let bind_vertex_array_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
            gl_context.set(js_string!("bindVertexArray"), JsValue::from(bind_vertex_array_fn.to_js_function(context.realm())), false, context)?;
        }

        Ok(JsValue::from(gl_context))
    }

    /// Get realistic WebGL renderer info for fingerprinting
    pub fn get_webgl_renderer_info() -> (String, String) {
        // Realistic renderer strings that match actual Chrome browsers
        let renderers = vec![
            ("ANGLE (Apple, Apple M1, OpenGL 4.1)", "Google Inc. (Apple)"),
            ("ANGLE (Intel, Intel(R) UHD Graphics 630, OpenGL 4.1)", "Google Inc. (Intel)"),
            ("ANGLE (NVIDIA, NVIDIA GeForce RTX 3080, OpenGL 4.1)", "Google Inc. (NVIDIA)"),
            ("ANGLE (AMD, AMD Radeon Pro 5500M, OpenGL 4.1)", "Google Inc. (AMD)"),
        ];

        let idx = std::collections::hash_map::DefaultHasher::new().finish() as usize % renderers.len();
        let (renderer, vendor) = renderers[idx];
        (renderer.to_string(), vendor.to_string())
    }
}