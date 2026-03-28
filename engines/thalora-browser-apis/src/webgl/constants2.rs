//! WebGL2 Constants
//!
//! WebGL 2.0 specific constant values.

use boa_engine::{Context, JsObject, JsValue, js_string};

use super::state::WebGL2Constants;

/// Add WebGL2 constants to a context object
pub fn add_webgl2_constants(obj: &JsObject, context: &mut Context) {
    let constants = [
        // WebGL2 buffer targets
        ("COPY_READ_BUFFER", WebGL2Constants::COPY_READ_BUFFER),
        ("COPY_WRITE_BUFFER", WebGL2Constants::COPY_WRITE_BUFFER),
        (
            "TRANSFORM_FEEDBACK_BUFFER",
            WebGL2Constants::TRANSFORM_FEEDBACK_BUFFER,
        ),
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
        (
            "MAX_COLOR_ATTACHMENTS",
            WebGL2Constants::MAX_COLOR_ATTACHMENTS,
        ),
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
        (
            "SAMPLER_2D_ARRAY_SHADOW",
            WebGL2Constants::SAMPLER_2D_ARRAY_SHADOW,
        ),
        ("SAMPLER_CUBE_SHADOW", WebGL2Constants::SAMPLER_CUBE_SHADOW),
        ("INT_SAMPLER_2D", WebGL2Constants::INT_SAMPLER_2D),
        ("INT_SAMPLER_3D", WebGL2Constants::INT_SAMPLER_3D),
        ("INT_SAMPLER_CUBE", WebGL2Constants::INT_SAMPLER_CUBE),
        (
            "INT_SAMPLER_2D_ARRAY",
            WebGL2Constants::INT_SAMPLER_2D_ARRAY,
        ),
        (
            "UNSIGNED_INT_SAMPLER_2D",
            WebGL2Constants::UNSIGNED_INT_SAMPLER_2D,
        ),
        (
            "UNSIGNED_INT_SAMPLER_3D",
            WebGL2Constants::UNSIGNED_INT_SAMPLER_3D,
        ),
        (
            "UNSIGNED_INT_SAMPLER_CUBE",
            WebGL2Constants::UNSIGNED_INT_SAMPLER_CUBE,
        ),
        (
            "UNSIGNED_INT_SAMPLER_2D_ARRAY",
            WebGL2Constants::UNSIGNED_INT_SAMPLER_2D_ARRAY,
        ),
        // Transform feedback
        ("TRANSFORM_FEEDBACK", WebGL2Constants::TRANSFORM_FEEDBACK),
        (
            "TRANSFORM_FEEDBACK_BINDING",
            WebGL2Constants::TRANSFORM_FEEDBACK_BINDING,
        ),
        (
            "TRANSFORM_FEEDBACK_ACTIVE",
            WebGL2Constants::TRANSFORM_FEEDBACK_ACTIVE,
        ),
        (
            "TRANSFORM_FEEDBACK_PAUSED",
            WebGL2Constants::TRANSFORM_FEEDBACK_PAUSED,
        ),
        // Sync
        (
            "SYNC_GPU_COMMANDS_COMPLETE",
            WebGL2Constants::SYNC_GPU_COMMANDS_COMPLETE,
        ),
        ("ALREADY_SIGNALED", WebGL2Constants::ALREADY_SIGNALED),
        ("TIMEOUT_EXPIRED", WebGL2Constants::TIMEOUT_EXPIRED),
        ("CONDITION_SATISFIED", WebGL2Constants::CONDITION_SATISFIED),
        ("WAIT_FAILED", WebGL2Constants::WAIT_FAILED),
        (
            "SYNC_FLUSH_COMMANDS_BIT",
            WebGL2Constants::SYNC_FLUSH_COMMANDS_BIT,
        ),
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
