//! WebGL State Management
//!
//! Manages WebGL state including blend modes, depth test, stencil, etc.

use std::collections::HashMap;

/// WebGL context state
#[derive(Debug, Clone)]
pub struct WebGLState {
    /// Clear color (RGBA)
    pub clear_color: [f32; 4],
    /// Clear depth value
    pub clear_depth: f32,
    /// Clear stencil value
    pub clear_stencil: i32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Blend enabled
    pub blend: bool,
    /// Cull face enabled
    pub cull_face: bool,
    /// Scissor test enabled
    pub scissor_test: bool,
    /// Stencil test enabled
    pub stencil_test: bool,
    /// Dither enabled
    pub dither: bool,
    /// Polygon offset fill enabled
    pub polygon_offset_fill: bool,
    /// Sample alpha to coverage
    pub sample_alpha_to_coverage: bool,
    /// Sample coverage
    pub sample_coverage: bool,
    /// Current blend function source RGB
    pub blend_src_rgb: u32,
    /// Current blend function dest RGB
    pub blend_dst_rgb: u32,
    /// Current blend function source alpha
    pub blend_src_alpha: u32,
    /// Current blend function dest alpha
    pub blend_dst_alpha: u32,
    /// Blend equation RGB
    pub blend_equation_rgb: u32,
    /// Blend equation alpha
    pub blend_equation_alpha: u32,
    /// Depth function
    pub depth_func: u32,
    /// Depth write mask
    pub depth_mask: bool,
    /// Front face winding
    pub front_face: u32,
    /// Cull face mode
    pub cull_face_mode: u32,
    /// Line width
    pub line_width: f32,
    /// Viewport
    pub viewport: [i32; 4],
    /// Scissor box
    pub scissor: [i32; 4],
    /// Color mask
    pub color_mask: [bool; 4],
    /// Stencil mask front
    pub stencil_mask_front: u32,
    /// Stencil mask back
    pub stencil_mask_back: u32,
    /// Active texture unit
    pub active_texture: u32,
    /// Currently bound array buffer
    pub bound_array_buffer: Option<u32>,
    /// Currently bound element array buffer
    pub bound_element_array_buffer: Option<u32>,
    /// Currently bound framebuffer
    pub bound_framebuffer: Option<u32>,
    /// Currently bound renderbuffer
    pub bound_renderbuffer: Option<u32>,
    /// Currently used program
    pub current_program: Option<u32>,
    /// Texture unit bindings (unit -> texture id)
    pub texture_bindings_2d: HashMap<u32, u32>,
    /// Cube map texture bindings
    pub texture_bindings_cube: HashMap<u32, u32>,
    /// Polygon offset factor
    pub polygon_offset_factor: f32,
    /// Polygon offset units
    pub polygon_offset_units: f32,
    /// Sample coverage value
    pub sample_coverage_value: f32,
    /// Sample coverage invert
    pub sample_coverage_invert: bool,
    /// Unpack flip Y
    pub unpack_flip_y: bool,
    /// Unpack premultiply alpha
    pub unpack_premultiply_alpha: bool,
    /// Unpack colorspace conversion
    pub unpack_colorspace_conversion: u32,
    /// Unpack alignment
    pub unpack_alignment: u32,
    /// Pack alignment
    pub pack_alignment: u32,
}

impl Default for WebGLState {
    fn default() -> Self {
        Self {
            clear_color: [0.0, 0.0, 0.0, 0.0],
            clear_depth: 1.0,
            clear_stencil: 0,
            depth_test: false,
            blend: false,
            cull_face: false,
            scissor_test: false,
            stencil_test: false,
            dither: true,
            polygon_offset_fill: false,
            sample_alpha_to_coverage: false,
            sample_coverage: false,
            blend_src_rgb: WebGLConstants::ONE,
            blend_dst_rgb: WebGLConstants::ZERO,
            blend_src_alpha: WebGLConstants::ONE,
            blend_dst_alpha: WebGLConstants::ZERO,
            blend_equation_rgb: WebGLConstants::FUNC_ADD,
            blend_equation_alpha: WebGLConstants::FUNC_ADD,
            depth_func: WebGLConstants::LESS,
            depth_mask: true,
            front_face: WebGLConstants::CCW,
            cull_face_mode: WebGLConstants::BACK,
            line_width: 1.0,
            viewport: [0, 0, 300, 150],
            scissor: [0, 0, 300, 150],
            color_mask: [true, true, true, true],
            stencil_mask_front: 0xFFFFFFFF,
            stencil_mask_back: 0xFFFFFFFF,
            active_texture: WebGLConstants::TEXTURE0,
            bound_array_buffer: None,
            bound_element_array_buffer: None,
            bound_framebuffer: None,
            bound_renderbuffer: None,
            current_program: None,
            texture_bindings_2d: HashMap::new(),
            texture_bindings_cube: HashMap::new(),
            polygon_offset_factor: 0.0,
            polygon_offset_units: 0.0,
            sample_coverage_value: 1.0,
            sample_coverage_invert: false,
            unpack_flip_y: false,
            unpack_premultiply_alpha: false,
            unpack_colorspace_conversion: WebGLConstants::BROWSER_DEFAULT_WEBGL,
            unpack_alignment: 4,
            pack_alignment: 4,
        }
    }
}

/// WebGL constants matching the WebGL specification
pub struct WebGLConstants;

impl WebGLConstants {
    // Clearing buffers
    pub const DEPTH_BUFFER_BIT: u32 = 0x00000100;
    pub const STENCIL_BUFFER_BIT: u32 = 0x00000400;
    pub const COLOR_BUFFER_BIT: u32 = 0x00004000;

    // Rendering primitives
    pub const POINTS: u32 = 0x0000;
    pub const LINES: u32 = 0x0001;
    pub const LINE_LOOP: u32 = 0x0002;
    pub const LINE_STRIP: u32 = 0x0003;
    pub const TRIANGLES: u32 = 0x0004;
    pub const TRIANGLE_STRIP: u32 = 0x0005;
    pub const TRIANGLE_FAN: u32 = 0x0006;

    // Blending modes
    pub const ZERO: u32 = 0;
    pub const ONE: u32 = 1;
    pub const SRC_COLOR: u32 = 0x0300;
    pub const ONE_MINUS_SRC_COLOR: u32 = 0x0301;
    pub const SRC_ALPHA: u32 = 0x0302;
    pub const ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
    pub const DST_ALPHA: u32 = 0x0304;
    pub const ONE_MINUS_DST_ALPHA: u32 = 0x0305;
    pub const DST_COLOR: u32 = 0x0306;
    pub const ONE_MINUS_DST_COLOR: u32 = 0x0307;
    pub const SRC_ALPHA_SATURATE: u32 = 0x0308;

    // Blend equations
    pub const FUNC_ADD: u32 = 0x8006;
    pub const FUNC_SUBTRACT: u32 = 0x800A;
    pub const FUNC_REVERSE_SUBTRACT: u32 = 0x800B;

    // Getting GL parameter information
    pub const BLEND_EQUATION: u32 = 0x8009;
    pub const BLEND_EQUATION_RGB: u32 = 0x8009;
    pub const BLEND_EQUATION_ALPHA: u32 = 0x883D;
    pub const BLEND_DST_RGB: u32 = 0x80C8;
    pub const BLEND_SRC_RGB: u32 = 0x80C9;
    pub const BLEND_DST_ALPHA: u32 = 0x80CA;
    pub const BLEND_SRC_ALPHA: u32 = 0x80CB;
    pub const BLEND_COLOR: u32 = 0x8005;
    pub const ARRAY_BUFFER_BINDING: u32 = 0x8894;
    pub const ELEMENT_ARRAY_BUFFER_BINDING: u32 = 0x8895;
    pub const LINE_WIDTH: u32 = 0x0B21;
    pub const ALIASED_POINT_SIZE_RANGE: u32 = 0x846D;
    pub const ALIASED_LINE_WIDTH_RANGE: u32 = 0x846E;
    pub const CULL_FACE_MODE: u32 = 0x0B45;
    pub const FRONT_FACE: u32 = 0x0B46;
    pub const DEPTH_RANGE: u32 = 0x0B70;
    pub const DEPTH_WRITEMASK: u32 = 0x0B72;
    pub const DEPTH_CLEAR_VALUE: u32 = 0x0B73;
    pub const DEPTH_FUNC: u32 = 0x0B74;
    pub const STENCIL_CLEAR_VALUE: u32 = 0x0B91;
    pub const STENCIL_FUNC: u32 = 0x0B92;
    pub const STENCIL_FAIL: u32 = 0x0B94;
    pub const STENCIL_PASS_DEPTH_FAIL: u32 = 0x0B95;
    pub const STENCIL_PASS_DEPTH_PASS: u32 = 0x0B96;
    pub const STENCIL_REF: u32 = 0x0B97;
    pub const STENCIL_VALUE_MASK: u32 = 0x0B93;
    pub const STENCIL_WRITEMASK: u32 = 0x0B98;
    pub const STENCIL_BACK_FUNC: u32 = 0x8800;
    pub const STENCIL_BACK_FAIL: u32 = 0x8801;
    pub const STENCIL_BACK_PASS_DEPTH_FAIL: u32 = 0x8802;
    pub const STENCIL_BACK_PASS_DEPTH_PASS: u32 = 0x8803;
    pub const STENCIL_BACK_REF: u32 = 0x8CA3;
    pub const STENCIL_BACK_VALUE_MASK: u32 = 0x8CA4;
    pub const STENCIL_BACK_WRITEMASK: u32 = 0x8CA5;
    pub const VIEWPORT: u32 = 0x0BA2;
    pub const SCISSOR_BOX: u32 = 0x0C10;
    pub const COLOR_CLEAR_VALUE: u32 = 0x0C22;
    pub const COLOR_WRITEMASK: u32 = 0x0C23;
    pub const UNPACK_ALIGNMENT: u32 = 0x0CF5;
    pub const PACK_ALIGNMENT: u32 = 0x0D05;
    pub const MAX_TEXTURE_SIZE: u32 = 0x0D33;
    pub const MAX_VIEWPORT_DIMS: u32 = 0x0D3A;
    pub const SUBPIXEL_BITS: u32 = 0x0D50;
    pub const RED_BITS: u32 = 0x0D52;
    pub const GREEN_BITS: u32 = 0x0D53;
    pub const BLUE_BITS: u32 = 0x0D54;
    pub const ALPHA_BITS: u32 = 0x0D55;
    pub const DEPTH_BITS: u32 = 0x0D56;
    pub const STENCIL_BITS: u32 = 0x0D57;
    pub const POLYGON_OFFSET_UNITS: u32 = 0x2A00;
    pub const POLYGON_OFFSET_FACTOR: u32 = 0x8038;
    pub const TEXTURE_BINDING_2D: u32 = 0x8069;
    pub const SAMPLE_BUFFERS: u32 = 0x80A8;
    pub const SAMPLES: u32 = 0x80A9;
    pub const SAMPLE_COVERAGE_VALUE: u32 = 0x80AA;
    pub const SAMPLE_COVERAGE_INVERT: u32 = 0x80AB;

    // Data types
    pub const BYTE: u32 = 0x1400;
    pub const UNSIGNED_BYTE: u32 = 0x1401;
    pub const SHORT: u32 = 0x1402;
    pub const UNSIGNED_SHORT: u32 = 0x1403;
    pub const INT: u32 = 0x1404;
    pub const UNSIGNED_INT: u32 = 0x1405;
    pub const FLOAT: u32 = 0x1406;

    // Pixel formats
    pub const DEPTH_COMPONENT: u32 = 0x1902;
    pub const ALPHA: u32 = 0x1906;
    pub const RGB: u32 = 0x1907;
    pub const RGBA: u32 = 0x1908;
    pub const LUMINANCE: u32 = 0x1909;
    pub const LUMINANCE_ALPHA: u32 = 0x190A;

    // Pixel types
    pub const UNSIGNED_SHORT_4_4_4_4: u32 = 0x8033;
    pub const UNSIGNED_SHORT_5_5_5_1: u32 = 0x8034;
    pub const UNSIGNED_SHORT_5_6_5: u32 = 0x8363;

    // Shaders
    pub const FRAGMENT_SHADER: u32 = 0x8B30;
    pub const VERTEX_SHADER: u32 = 0x8B31;
    pub const MAX_VERTEX_ATTRIBS: u32 = 0x8869;
    pub const MAX_VERTEX_UNIFORM_VECTORS: u32 = 0x8DFB;
    pub const MAX_VARYING_VECTORS: u32 = 0x8DFC;
    pub const MAX_COMBINED_TEXTURE_IMAGE_UNITS: u32 = 0x8B4D;
    pub const MAX_VERTEX_TEXTURE_IMAGE_UNITS: u32 = 0x8B4C;
    pub const MAX_TEXTURE_IMAGE_UNITS: u32 = 0x8872;
    pub const MAX_FRAGMENT_UNIFORM_VECTORS: u32 = 0x8DFD;
    pub const SHADER_TYPE: u32 = 0x8B4F;
    pub const DELETE_STATUS: u32 = 0x8B80;
    pub const LINK_STATUS: u32 = 0x8B82;
    pub const VALIDATE_STATUS: u32 = 0x8B83;
    pub const ATTACHED_SHADERS: u32 = 0x8B85;
    pub const ACTIVE_UNIFORMS: u32 = 0x8B86;
    pub const ACTIVE_UNIFORM_MAX_LENGTH: u32 = 0x8B87;
    pub const ACTIVE_ATTRIBUTES: u32 = 0x8B89;
    pub const ACTIVE_ATTRIBUTE_MAX_LENGTH: u32 = 0x8B8A;
    pub const SHADING_LANGUAGE_VERSION: u32 = 0x8B8C;
    pub const CURRENT_PROGRAM: u32 = 0x8B8D;

    // Stencil operations
    pub const NEVER: u32 = 0x0200;
    pub const LESS: u32 = 0x0201;
    pub const EQUAL: u32 = 0x0202;
    pub const LEQUAL: u32 = 0x0203;
    pub const GREATER: u32 = 0x0204;
    pub const NOTEQUAL: u32 = 0x0205;
    pub const GEQUAL: u32 = 0x0206;
    pub const ALWAYS: u32 = 0x0207;
    pub const KEEP: u32 = 0x1E00;
    pub const REPLACE: u32 = 0x1E01;
    pub const INCR: u32 = 0x1E02;
    pub const DECR: u32 = 0x1E03;
    pub const INVERT: u32 = 0x150A;
    pub const INCR_WRAP: u32 = 0x8507;
    pub const DECR_WRAP: u32 = 0x8508;

    // Textures
    pub const NEAREST: u32 = 0x2600;
    pub const LINEAR: u32 = 0x2601;
    pub const NEAREST_MIPMAP_NEAREST: u32 = 0x2700;
    pub const LINEAR_MIPMAP_NEAREST: u32 = 0x2701;
    pub const NEAREST_MIPMAP_LINEAR: u32 = 0x2702;
    pub const LINEAR_MIPMAP_LINEAR: u32 = 0x2703;
    pub const TEXTURE_MAG_FILTER: u32 = 0x2800;
    pub const TEXTURE_MIN_FILTER: u32 = 0x2801;
    pub const TEXTURE_WRAP_S: u32 = 0x2802;
    pub const TEXTURE_WRAP_T: u32 = 0x2803;
    pub const TEXTURE_2D: u32 = 0x0DE1;
    pub const TEXTURE: u32 = 0x1702;
    pub const TEXTURE_CUBE_MAP: u32 = 0x8513;
    pub const TEXTURE_BINDING_CUBE_MAP: u32 = 0x8514;
    pub const TEXTURE_CUBE_MAP_POSITIVE_X: u32 = 0x8515;
    pub const TEXTURE_CUBE_MAP_NEGATIVE_X: u32 = 0x8516;
    pub const TEXTURE_CUBE_MAP_POSITIVE_Y: u32 = 0x8517;
    pub const TEXTURE_CUBE_MAP_NEGATIVE_Y: u32 = 0x8518;
    pub const TEXTURE_CUBE_MAP_POSITIVE_Z: u32 = 0x8519;
    pub const TEXTURE_CUBE_MAP_NEGATIVE_Z: u32 = 0x851A;
    pub const MAX_CUBE_MAP_TEXTURE_SIZE: u32 = 0x851C;
    pub const TEXTURE0: u32 = 0x84C0;
    pub const TEXTURE1: u32 = 0x84C1;
    pub const TEXTURE2: u32 = 0x84C2;
    pub const TEXTURE3: u32 = 0x84C3;
    pub const TEXTURE4: u32 = 0x84C4;
    pub const TEXTURE5: u32 = 0x84C5;
    pub const TEXTURE6: u32 = 0x84C6;
    pub const TEXTURE7: u32 = 0x84C7;
    pub const TEXTURE8: u32 = 0x84C8;
    pub const TEXTURE9: u32 = 0x84C9;
    pub const TEXTURE10: u32 = 0x84CA;
    pub const TEXTURE11: u32 = 0x84CB;
    pub const TEXTURE12: u32 = 0x84CC;
    pub const TEXTURE13: u32 = 0x84CD;
    pub const TEXTURE14: u32 = 0x84CE;
    pub const TEXTURE15: u32 = 0x84CF;
    pub const TEXTURE16: u32 = 0x84D0;
    pub const TEXTURE17: u32 = 0x84D1;
    pub const TEXTURE18: u32 = 0x84D2;
    pub const TEXTURE19: u32 = 0x84D3;
    pub const TEXTURE20: u32 = 0x84D4;
    pub const TEXTURE21: u32 = 0x84D5;
    pub const TEXTURE22: u32 = 0x84D6;
    pub const TEXTURE23: u32 = 0x84D7;
    pub const TEXTURE24: u32 = 0x84D8;
    pub const TEXTURE25: u32 = 0x84D9;
    pub const TEXTURE26: u32 = 0x84DA;
    pub const TEXTURE27: u32 = 0x84DB;
    pub const TEXTURE28: u32 = 0x84DC;
    pub const TEXTURE29: u32 = 0x84DD;
    pub const TEXTURE30: u32 = 0x84DE;
    pub const TEXTURE31: u32 = 0x84DF;
    pub const REPEAT: u32 = 0x2901;
    pub const CLAMP_TO_EDGE: u32 = 0x812F;
    pub const MIRRORED_REPEAT: u32 = 0x8370;

    // Uniform types
    pub const FLOAT_VEC2: u32 = 0x8B50;
    pub const FLOAT_VEC3: u32 = 0x8B51;
    pub const FLOAT_VEC4: u32 = 0x8B52;
    pub const INT_VEC2: u32 = 0x8B53;
    pub const INT_VEC3: u32 = 0x8B54;
    pub const INT_VEC4: u32 = 0x8B55;
    pub const BOOL: u32 = 0x8B56;
    pub const BOOL_VEC2: u32 = 0x8B57;
    pub const BOOL_VEC3: u32 = 0x8B58;
    pub const BOOL_VEC4: u32 = 0x8B59;
    pub const FLOAT_MAT2: u32 = 0x8B5A;
    pub const FLOAT_MAT3: u32 = 0x8B5B;
    pub const FLOAT_MAT4: u32 = 0x8B5C;
    pub const SAMPLER_2D: u32 = 0x8B5E;
    pub const SAMPLER_CUBE: u32 = 0x8B60;

    // Shader source
    pub const COMPILE_STATUS: u32 = 0x8B81;
    pub const INFO_LOG_LENGTH: u32 = 0x8B84;
    pub const SHADER_SOURCE_LENGTH: u32 = 0x8B88;

    // Buffer objects
    pub const ARRAY_BUFFER: u32 = 0x8892;
    pub const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
    pub const BUFFER_SIZE: u32 = 0x8764;
    pub const BUFFER_USAGE: u32 = 0x8765;
    pub const STREAM_DRAW: u32 = 0x88E0;
    pub const STATIC_DRAW: u32 = 0x88E4;
    pub const DYNAMIC_DRAW: u32 = 0x88E8;

    // Culling
    pub const FRONT: u32 = 0x0404;
    pub const BACK: u32 = 0x0405;
    pub const FRONT_AND_BACK: u32 = 0x0408;

    // Enabling and disabling
    pub const BLEND: u32 = 0x0BE2;
    pub const DEPTH_TEST: u32 = 0x0B71;
    pub const DITHER: u32 = 0x0BD0;
    pub const POLYGON_OFFSET_FILL: u32 = 0x8037;
    pub const SAMPLE_ALPHA_TO_COVERAGE: u32 = 0x809E;
    pub const SAMPLE_COVERAGE: u32 = 0x80A0;
    pub const SCISSOR_TEST: u32 = 0x0C11;
    pub const STENCIL_TEST: u32 = 0x0B90;
    pub const CULL_FACE: u32 = 0x0B44;

    // Errors
    pub const NO_ERROR: u32 = 0;
    pub const INVALID_ENUM: u32 = 0x0500;
    pub const INVALID_VALUE: u32 = 0x0501;
    pub const INVALID_OPERATION: u32 = 0x0502;
    pub const OUT_OF_MEMORY: u32 = 0x0505;
    pub const INVALID_FRAMEBUFFER_OPERATION: u32 = 0x0506;
    pub const CONTEXT_LOST_WEBGL: u32 = 0x9242;

    // Front face directions
    pub const CW: u32 = 0x0900;
    pub const CCW: u32 = 0x0901;

    // Hints
    pub const DONT_CARE: u32 = 0x1100;
    pub const FASTEST: u32 = 0x1101;
    pub const NICEST: u32 = 0x1102;
    pub const GENERATE_MIPMAP_HINT: u32 = 0x8192;

    // Pixel storage
    pub const UNPACK_FLIP_Y_WEBGL: u32 = 0x9240;
    pub const UNPACK_PREMULTIPLY_ALPHA_WEBGL: u32 = 0x9241;
    pub const UNPACK_COLORSPACE_CONVERSION_WEBGL: u32 = 0x9243;
    pub const BROWSER_DEFAULT_WEBGL: u32 = 0x9244;

    // Framebuffers and renderbuffers
    pub const FRAMEBUFFER: u32 = 0x8D40;
    pub const RENDERBUFFER: u32 = 0x8D41;
    pub const RGBA4: u32 = 0x8056;
    pub const RGB5_A1: u32 = 0x8057;
    pub const RGB565: u32 = 0x8D62;
    pub const DEPTH_COMPONENT16: u32 = 0x81A5;
    pub const STENCIL_INDEX8: u32 = 0x8D48;
    pub const DEPTH_STENCIL: u32 = 0x84F9;
    pub const RENDERBUFFER_WIDTH: u32 = 0x8D42;
    pub const RENDERBUFFER_HEIGHT: u32 = 0x8D43;
    pub const RENDERBUFFER_INTERNAL_FORMAT: u32 = 0x8D44;
    pub const RENDERBUFFER_RED_SIZE: u32 = 0x8D50;
    pub const RENDERBUFFER_GREEN_SIZE: u32 = 0x8D51;
    pub const RENDERBUFFER_BLUE_SIZE: u32 = 0x8D52;
    pub const RENDERBUFFER_ALPHA_SIZE: u32 = 0x8D53;
    pub const RENDERBUFFER_DEPTH_SIZE: u32 = 0x8D54;
    pub const RENDERBUFFER_STENCIL_SIZE: u32 = 0x8D55;
    pub const FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: u32 = 0x8CD0;
    pub const FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: u32 = 0x8CD1;
    pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: u32 = 0x8CD2;
    pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: u32 = 0x8CD3;
    pub const COLOR_ATTACHMENT0: u32 = 0x8CE0;
    pub const DEPTH_ATTACHMENT: u32 = 0x8D00;
    pub const STENCIL_ATTACHMENT: u32 = 0x8D20;
    pub const DEPTH_STENCIL_ATTACHMENT: u32 = 0x821A;
    pub const NONE: u32 = 0;
    pub const FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
    pub const FRAMEBUFFER_INCOMPLETE_ATTACHMENT: u32 = 0x8CD6;
    pub const FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: u32 = 0x8CD7;
    pub const FRAMEBUFFER_INCOMPLETE_DIMENSIONS: u32 = 0x8CD9;
    pub const FRAMEBUFFER_UNSUPPORTED: u32 = 0x8CDD;
    pub const FRAMEBUFFER_BINDING: u32 = 0x8CA6;
    pub const RENDERBUFFER_BINDING: u32 = 0x8CA7;
    pub const MAX_RENDERBUFFER_SIZE: u32 = 0x84E8;

    // Vertex attributes
    pub const VERTEX_ATTRIB_ARRAY_ENABLED: u32 = 0x8622;
    pub const VERTEX_ATTRIB_ARRAY_SIZE: u32 = 0x8623;
    pub const VERTEX_ATTRIB_ARRAY_STRIDE: u32 = 0x8624;
    pub const VERTEX_ATTRIB_ARRAY_TYPE: u32 = 0x8625;
    pub const VERTEX_ATTRIB_ARRAY_NORMALIZED: u32 = 0x886A;
    pub const VERTEX_ATTRIB_ARRAY_POINTER: u32 = 0x8645;
    pub const VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: u32 = 0x889F;
    pub const CURRENT_VERTEX_ATTRIB: u32 = 0x8626;

    // Getting information
    pub const VENDOR: u32 = 0x1F00;
    pub const RENDERER: u32 = 0x1F01;
    pub const VERSION: u32 = 0x1F02;

    // High/low precision
    pub const LOW_FLOAT: u32 = 0x8DF0;
    pub const MEDIUM_FLOAT: u32 = 0x8DF1;
    pub const HIGH_FLOAT: u32 = 0x8DF2;
    pub const LOW_INT: u32 = 0x8DF3;
    pub const MEDIUM_INT: u32 = 0x8DF4;
    pub const HIGH_INT: u32 = 0x8DF5;

    // Implementation color read
    pub const IMPLEMENTATION_COLOR_READ_TYPE: u32 = 0x8B9A;
    pub const IMPLEMENTATION_COLOR_READ_FORMAT: u32 = 0x8B9B;
}

/// WebGL2 additional constants
pub struct WebGL2Constants;

impl WebGL2Constants {
    // WebGL2 specific buffer targets
    pub const COPY_READ_BUFFER: u32 = 0x8F36;
    pub const COPY_WRITE_BUFFER: u32 = 0x8F37;
    pub const TRANSFORM_FEEDBACK_BUFFER: u32 = 0x8C8E;
    pub const UNIFORM_BUFFER: u32 = 0x8A11;
    pub const PIXEL_PACK_BUFFER: u32 = 0x88EB;
    pub const PIXEL_UNPACK_BUFFER: u32 = 0x88EC;

    // WebGL2 buffer usage
    pub const STREAM_READ: u32 = 0x88E1;
    pub const STREAM_COPY: u32 = 0x88E2;
    pub const STATIC_READ: u32 = 0x88E5;
    pub const STATIC_COPY: u32 = 0x88E6;
    pub const DYNAMIC_READ: u32 = 0x88E9;
    pub const DYNAMIC_COPY: u32 = 0x88EA;

    // WebGL2 draw buffers
    pub const DRAW_BUFFER0: u32 = 0x8825;
    pub const DRAW_BUFFER1: u32 = 0x8826;
    pub const DRAW_BUFFER2: u32 = 0x8827;
    pub const DRAW_BUFFER3: u32 = 0x8828;
    pub const DRAW_BUFFER4: u32 = 0x8829;
    pub const DRAW_BUFFER5: u32 = 0x882A;
    pub const DRAW_BUFFER6: u32 = 0x882B;
    pub const DRAW_BUFFER7: u32 = 0x882C;
    pub const DRAW_BUFFER8: u32 = 0x882D;
    pub const DRAW_BUFFER9: u32 = 0x882E;
    pub const DRAW_BUFFER10: u32 = 0x882F;
    pub const DRAW_BUFFER11: u32 = 0x8830;
    pub const DRAW_BUFFER12: u32 = 0x8831;
    pub const DRAW_BUFFER13: u32 = 0x8832;
    pub const DRAW_BUFFER14: u32 = 0x8833;
    pub const DRAW_BUFFER15: u32 = 0x8834;
    pub const MAX_DRAW_BUFFERS: u32 = 0x8824;
    pub const MAX_COLOR_ATTACHMENTS: u32 = 0x8CDF;

    // WebGL2 internal formats
    pub const R8: u32 = 0x8229;
    pub const RG8: u32 = 0x822B;
    pub const R16F: u32 = 0x822D;
    pub const R32F: u32 = 0x822E;
    pub const RG16F: u32 = 0x822F;
    pub const RG32F: u32 = 0x8230;
    pub const RGBA32F: u32 = 0x8814;
    pub const RGB32F: u32 = 0x8815;
    pub const RGBA16F: u32 = 0x881A;
    pub const RGB16F: u32 = 0x881B;
    pub const DEPTH_COMPONENT24: u32 = 0x81A6;
    pub const DEPTH_COMPONENT32F: u32 = 0x8CAC;
    pub const DEPTH24_STENCIL8: u32 = 0x88F0;
    pub const DEPTH32F_STENCIL8: u32 = 0x8CAD;

    // WebGL2 pixel formats
    pub const RED: u32 = 0x1903;
    pub const RG: u32 = 0x8227;
    pub const RED_INTEGER: u32 = 0x8D94;
    pub const RG_INTEGER: u32 = 0x8228;
    pub const RGB_INTEGER: u32 = 0x8D98;
    pub const RGBA_INTEGER: u32 = 0x8D99;

    // WebGL2 samplers
    pub const SAMPLER_3D: u32 = 0x8B5F;
    pub const SAMPLER_2D_SHADOW: u32 = 0x8B62;
    pub const SAMPLER_2D_ARRAY: u32 = 0x8DC1;
    pub const SAMPLER_2D_ARRAY_SHADOW: u32 = 0x8DC4;
    pub const SAMPLER_CUBE_SHADOW: u32 = 0x8DC5;
    pub const INT_SAMPLER_2D: u32 = 0x8DCA;
    pub const INT_SAMPLER_3D: u32 = 0x8DCB;
    pub const INT_SAMPLER_CUBE: u32 = 0x8DCC;
    pub const INT_SAMPLER_2D_ARRAY: u32 = 0x8DCF;
    pub const UNSIGNED_INT_SAMPLER_2D: u32 = 0x8DD2;
    pub const UNSIGNED_INT_SAMPLER_3D: u32 = 0x8DD3;
    pub const UNSIGNED_INT_SAMPLER_CUBE: u32 = 0x8DD4;
    pub const UNSIGNED_INT_SAMPLER_2D_ARRAY: u32 = 0x8DD7;

    // WebGL2 texture 3D
    pub const TEXTURE_3D: u32 = 0x806F;
    pub const TEXTURE_2D_ARRAY: u32 = 0x8C1A;
    pub const TEXTURE_WRAP_R: u32 = 0x8072;

    // WebGL2 transform feedback
    pub const TRANSFORM_FEEDBACK: u32 = 0x8E22;
    pub const TRANSFORM_FEEDBACK_BINDING: u32 = 0x8E25;
    pub const TRANSFORM_FEEDBACK_ACTIVE: u32 = 0x8E24;
    pub const TRANSFORM_FEEDBACK_PAUSED: u32 = 0x8E23;

    // WebGL2 sync objects
    pub const SYNC_GPU_COMMANDS_COMPLETE: u32 = 0x9117;
    pub const ALREADY_SIGNALED: u32 = 0x911A;
    pub const TIMEOUT_EXPIRED: u32 = 0x911B;
    pub const CONDITION_SATISFIED: u32 = 0x911C;
    pub const WAIT_FAILED: u32 = 0x911D;
    pub const SYNC_FLUSH_COMMANDS_BIT: u32 = 0x00000001;
}
