//! WebGL Constants
//!
//! Adds WebGL 1.0 constants to context objects.

use boa_engine::{Context, JsObject, JsValue, js_string};

use super::state::WebGLConstants;

/// Add WebGL constants to object
pub fn add_webgl_constants(obj: &JsObject, context: &mut Context) {
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
        (
            "FUNC_REVERSE_SUBTRACT",
            WebGLConstants::FUNC_REVERSE_SUBTRACT,
        ),
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
        (
            "NEAREST_MIPMAP_NEAREST",
            WebGLConstants::NEAREST_MIPMAP_NEAREST,
        ),
        (
            "LINEAR_MIPMAP_NEAREST",
            WebGLConstants::LINEAR_MIPMAP_NEAREST,
        ),
        (
            "NEAREST_MIPMAP_LINEAR",
            WebGLConstants::NEAREST_MIPMAP_LINEAR,
        ),
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
        (
            "DEPTH_STENCIL_ATTACHMENT",
            WebGLConstants::DEPTH_STENCIL_ATTACHMENT,
        ),
        ("FRAMEBUFFER_COMPLETE", WebGLConstants::FRAMEBUFFER_COMPLETE),
        // Errors
        ("NO_ERROR", WebGLConstants::NO_ERROR),
        ("INVALID_ENUM", WebGLConstants::INVALID_ENUM),
        ("INVALID_VALUE", WebGLConstants::INVALID_VALUE),
        ("INVALID_OPERATION", WebGLConstants::INVALID_OPERATION),
        ("OUT_OF_MEMORY", WebGLConstants::OUT_OF_MEMORY),
        (
            "INVALID_FRAMEBUFFER_OPERATION",
            WebGLConstants::INVALID_FRAMEBUFFER_OPERATION,
        ),
        ("CONTEXT_LOST_WEBGL", WebGLConstants::CONTEXT_LOST_WEBGL),
        // Getting info
        ("VENDOR", WebGLConstants::VENDOR),
        ("RENDERER", WebGLConstants::RENDERER),
        ("VERSION", WebGLConstants::VERSION),
        (
            "SHADING_LANGUAGE_VERSION",
            WebGLConstants::SHADING_LANGUAGE_VERSION,
        ),
        // Pixel storage
        ("UNPACK_FLIP_Y_WEBGL", WebGLConstants::UNPACK_FLIP_Y_WEBGL),
        (
            "UNPACK_PREMULTIPLY_ALPHA_WEBGL",
            WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
        ),
        (
            "UNPACK_COLORSPACE_CONVERSION_WEBGL",
            WebGLConstants::UNPACK_COLORSPACE_CONVERSION_WEBGL,
        ),
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
