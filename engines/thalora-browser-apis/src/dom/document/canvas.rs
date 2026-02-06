//! Canvas 2D and WebGL context methods
//!
//! canvas_get_context, canvas_to_data_url, canvas_2d_* methods, create_webgl_context

use boa_engine::{
    builtins::BuiltInBuilder,
    NativeFunction,
    object::{JsObject, builtins::JsArray},
    value::JsValue,
    Context, JsArgs, JsResult, js_string,
    JsString, property::PropertyDescriptorBuilder
};

/// Canvas `getContext(contextType)` method implementation
pub(super) fn canvas_get_context(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let context_type = args.get_or_undefined(0).to_string(context)?;
    let context_type_str = context_type.to_std_string_escaped();

    match context_type_str.as_str() {
        "2d" => {
            // Create a Canvas 2D rendering context object
            let context_2d = JsObject::default(context.intrinsics());

            // Add Canvas 2D methods
            // Drawing rectangles
            let fill_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_rect)
                .name(js_string!("fillRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_rect_func)
                    .build(),
                context,
            )?;

            let stroke_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_rect)
                .name(js_string!("strokeRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_rect_func)
                    .build(),
                context,
            )?;

            let clear_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_clear_rect)
                .name(js_string!("clearRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("clearRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(clear_rect_func)
                    .build(),
                context,
            )?;

            // Text rendering
            let fill_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_text)
                .name(js_string!("fillText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_text_func)
                    .build(),
                context,
            )?;

            let stroke_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_text)
                .name(js_string!("strokeText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_text_func)
                    .build(),
                context,
            )?;

            let measure_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_measure_text)
                .name(js_string!("measureText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("measureText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(measure_text_func)
                    .build(),
                context,
            )?;

            // Path methods
            let begin_path_func = BuiltInBuilder::callable(context.realm(), canvas_2d_begin_path)
                .name(js_string!("beginPath"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("beginPath"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(begin_path_func)
                    .build(),
                context,
            )?;

            let move_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_move_to)
                .name(js_string!("moveTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("moveTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(move_to_func)
                    .build(),
                context,
            )?;

            let line_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_line_to)
                .name(js_string!("lineTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("lineTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(line_to_func)
                    .build(),
                context,
            )?;

            let stroke_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke)
                .name(js_string!("stroke"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("stroke"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_func)
                    .build(),
                context,
            )?;

            let fill_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill)
                .name(js_string!("fill"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fill"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_func)
                    .build(),
                context,
            )?;

            // Style properties
            context_2d.define_property_or_throw(
                js_string!("fillStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("strokeStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("lineWidth"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(1.0)
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("font"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("10px sans-serif"))
                    .build(),
                context,
            )?;

            Ok(context_2d.into())
        }
        "webgl" | "experimental-webgl" => {
            create_webgl_context(context, false)
        }
        "webgl2" | "experimental-webgl2" => {
            create_webgl_context(context, true)
        }
        _ => Ok(JsValue::null())
    }
}

/// Canvas `toDataURL(type, quality)` method implementation
pub(super) fn canvas_to_data_url(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _mime_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let _quality = args.get_or_undefined(1).to_number(context)?;

    // For now, return a minimal empty PNG data URL
    // TODO: Implement actual image generation
    Ok(js_string!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==").into())
}

// Canvas 2D context method implementations
fn canvas_2d_fill_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle drawing
    eprintln!("Canvas fillRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle outlining
    eprintln!("Canvas strokeRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_clear_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle clearing
    eprintln!("Canvas clearRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_fill_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text rendering
    eprintln!("Canvas fillText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text stroking
    eprintln!("Canvas strokeText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_measure_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;

    // Create TextMetrics object
    let metrics = JsObject::default(context.intrinsics());

    // Calculate approximate width (very basic implementation)
    let text_width = text.to_std_string_escaped().len() as f64 * 6.0; // Rough estimate

    metrics.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(text_width)
            .build(),
        context,
    )?;

    // TODO: Add other TextMetrics properties (actualBoundingBoxLeft, etc.)

    Ok(metrics.into())
}

fn canvas_2d_begin_path(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path state management
    eprintln!("Canvas beginPath()");
    Ok(JsValue::undefined())
}

fn canvas_2d_move_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement path cursor movement
    eprintln!("Canvas moveTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_line_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement line drawing to path
    eprintln!("Canvas lineTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path stroking
    eprintln!("Canvas stroke()");
    Ok(JsValue::undefined())
}

fn canvas_2d_fill(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path filling
    eprintln!("Canvas fill()");
    Ok(JsValue::undefined())
}

/// Create WebGL context with comprehensive method support
fn create_webgl_context(context: &mut Context, is_webgl2: bool) -> JsResult<JsValue> {
    let gl_context = JsObject::default(context.intrinsics());

    // WebGL constants (subset of most commonly used)
    gl_context.set(js_string!("VERTEX_SHADER"), JsValue::from(35633), false, context)?;
    gl_context.set(js_string!("FRAGMENT_SHADER"), JsValue::from(35632), false, context)?;
    gl_context.set(js_string!("ARRAY_BUFFER"), JsValue::from(34962), false, context)?;
    gl_context.set(js_string!("STATIC_DRAW"), JsValue::from(35044), false, context)?;
    gl_context.set(js_string!("COLOR_BUFFER_BIT"), JsValue::from(16384), false, context)?;
    gl_context.set(js_string!("TRIANGLES"), JsValue::from(4), false, context)?;
    gl_context.set(js_string!("FLOAT"), JsValue::from(5126), false, context)?;

    // Core WebGL methods
    let create_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let shader_obj = JsObject::default(_context.intrinsics());
        Ok(JsValue::from(shader_obj))
    }) };
    gl_context.set(js_string!("createShader"), JsValue::from(create_shader_fn.to_js_function(context.realm())), false, context)?;

    let create_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let program_obj = JsObject::default(_context.intrinsics());
        Ok(JsValue::from(program_obj))
    }) };
    gl_context.set(js_string!("createProgram"), JsValue::from(create_program_fn.to_js_function(context.realm())), false, context)?;

    let create_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let buffer_obj = JsObject::default(_context.intrinsics());
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
            34921 => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"))), // GL_SHADING_LANGUAGE_VERSION
            34930 => Ok(JsValue::from(16)), // GL_MAX_TEXTURE_SIZE
            3379 => Ok(JsValue::from(16384)), // GL_MAX_VIEWPORT_DIMS
            _ => Ok(JsValue::from(0))
        }
    }) };
    gl_context.set(js_string!("getParameter"), JsValue::from(get_parameter_fn.to_js_function(context.realm())), false, context)?;

    // Extensions support
    let get_extension_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
        if args.is_empty() {
            return Ok(JsValue::null());
        }

        let ext_name = args[0].to_string(context)?.to_std_string_escaped();
        match ext_name.as_str() {
            "WEBKIT_EXT_texture_filter_anisotropic" |
            "EXT_texture_filter_anisotropic" |
            "OES_element_index_uint" |
            "OES_standard_derivatives" => {
                let ext_obj = JsObject::default(context.intrinsics());
                Ok(JsValue::from(ext_obj))
            },
            _ => Ok(JsValue::null())
        }
    }) };
    gl_context.set(js_string!("getExtension"), JsValue::from(get_extension_fn.to_js_function(context.realm())), false, context)?;

    let get_supported_extensions_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
        let extensions = vec![
            "WEBKIT_EXT_texture_filter_anisotropic",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "WEBGL_debug_renderer_info"
        ];

        let js_array = JsArray::new(context);
        for (i, ext) in extensions.iter().enumerate() {
            js_array.set(i, js_string!(*ext), true, context).ok();
        }
        Ok(JsValue::from(js_array))
    }) };
    gl_context.set(js_string!("getSupportedExtensions"), JsValue::from(get_supported_extensions_fn.to_js_function(context.realm())), false, context)?;

    // WebGL2 specific methods
    if is_webgl2 {
        let create_vertex_array_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            let vao_obj = JsObject::default(_context.intrinsics());
            Ok(JsValue::from(vao_obj))
        }) };
        gl_context.set(js_string!("createVertexArray"), JsValue::from(create_vertex_array_fn.to_js_function(context.realm())), false, context)?;
    }

    Ok(JsValue::from(gl_context))
}
