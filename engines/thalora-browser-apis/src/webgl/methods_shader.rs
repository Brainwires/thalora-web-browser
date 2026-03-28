//! WebGL Shader and Context Methods
//!
//! Context initialization methods and shader/program operations.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
};

use super::context::{WebGLRenderingContextData, get_object_id, get_parameter};
use super::shader::{WebGLProgram, WebGLShader};
use super::state::WebGLConstants;
use crate::with_webgl_context;

/// Add context methods
pub fn add_context_methods(obj: &JsObject, context: &mut Context) {
    // getError
    obj.set(
        js_string!("getError"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            Ok(JsValue::from(data.get_error()))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // isContextLost
    obj.set(
        js_string!("isContextLost"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let lost = *data.context_lost.lock().unwrap();
            Ok(JsValue::from(lost))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getParameter
    obj.set(
        js_string!("getParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let pname = args.get_or_undefined(0).to_u32(ctx)?;
            get_parameter(&data, pname, ctx)
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getExtension
    obj.set(
        js_string!("getExtension"),
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let name = args.get_or_undefined(0).to_string(ctx)?;
            let _ = name;
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getSupportedExtensions
    obj.set(
        js_string!("getSupportedExtensions"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let arr = boa_engine::object::builtins::JsArray::new(ctx)?;
            Ok(arr.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getContextAttributes
    obj.set(
        js_string!("getContextAttributes"),
        NativeFunction::from_fn_ptr(|_this, _args, ctx| {
            let obj = JsObject::with_null_proto();
            obj.set(js_string!("alpha"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("depth"), JsValue::from(true), false, ctx)?;
            obj.set(js_string!("stencil"), JsValue::from(false), false, ctx)?;
            obj.set(js_string!("antialias"), JsValue::from(true), false, ctx)?;
            obj.set(
                js_string!("premultipliedAlpha"),
                JsValue::from(true),
                false,
                ctx,
            )?;
            obj.set(
                js_string!("preserveDrawingBuffer"),
                JsValue::from(false),
                false,
                ctx,
            )?;
            obj.set(
                js_string!("powerPreference"),
                js_string!("default"),
                false,
                ctx,
            )?;
            obj.set(
                js_string!("failIfMajorPerformanceCaveat"),
                JsValue::from(false),
                false,
                ctx,
            )?;
            Ok(obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add shader methods
pub fn add_shader_methods(obj: &JsObject, context: &mut Context) {
    // createShader
    obj.set(
        js_string!("createShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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

            let shader_obj = JsObject::with_null_proto();
            shader_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(shader_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // shaderSource
    obj.set(
        js_string!("shaderSource"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // compileShader
    obj.set(
        js_string!("compileShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get_mut(&shader_id) {
                shader.compile();
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getShaderParameter
    obj.set(
        js_string!("getShaderParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // getShaderInfoLog
    obj.set(
        js_string!("getShaderInfoLog"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get(&shader_id) {
                return Ok(JsValue::from(js_string!(shader.get_info_log())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getShaderSource
    obj.set(
        js_string!("getShaderSource"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get(&shader_id) {
                return Ok(JsValue::from(js_string!(shader.get_source())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteShader
    obj.set(
        js_string!("deleteShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let shader_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(shader) = data.shaders.lock().unwrap().get_mut(&shader_id) {
                shader.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // createProgram
    obj.set(
        js_string!("createProgram"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);

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
    )
    .unwrap();

    // attachShader
    obj.set(
        js_string!("attachShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // detachShader
    obj.set(
        js_string!("detachShader"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // linkProgram
    obj.set(
        js_string!("linkProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // useProgram
    obj.set(
        js_string!("useProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // getProgramParameter
    obj.set(
        js_string!("getProgramParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();

    // getProgramInfoLog
    obj.set(
        js_string!("getProgramInfoLog"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                return Ok(JsValue::from(js_string!(program.get_info_log())));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // validateProgram
    obj.set(
        js_string!("validateProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                program.validate();
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteProgram
    obj.set(
        js_string!("deleteProgram"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get_mut(&program_id) {
                program.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getUniformLocation
    obj.set(
        js_string!("getUniformLocation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let program_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let name = args.get_or_undefined(1).to_string(ctx)?;

            if let Some(program) = data.programs.lock().unwrap().get(&program_id) {
                if let Some(location) = program.get_uniform_location(&name.to_std_string_escaped())
                {
                    let loc_obj = JsObject::with_null_proto();
                    loc_obj.set(js_string!("_id"), JsValue::from(location.id), false, ctx)?;
                    loc_obj.set(
                        js_string!("_program"),
                        JsValue::from(program_id),
                        false,
                        ctx,
                    )?;
                    return Ok(loc_obj.into());
                }
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getAttribLocation
    obj.set(
        js_string!("getAttribLocation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
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
    )
    .unwrap();
}
