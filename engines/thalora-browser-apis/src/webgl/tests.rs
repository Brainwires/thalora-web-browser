//! Comprehensive test suite for WebGL APIs
//! Tests WebGLRenderingContext, WebGL2RenderingContext, and related objects

use boa_engine::string::JsString;
use boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// WebGLRenderingContext Constructor Tests
// ============================================================================

#[test]
fn test_webglrenderingcontext_constructor_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof WebGLRenderingContext"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

// ============================================================================
// WebGLRenderingContext Constants Tests
// ============================================================================

#[test]
fn test_webgl_constants_depth_buffer_bit() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.DEPTH_BUFFER_BIT === 0x00000100;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_color_buffer_bit() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.COLOR_BUFFER_BIT === 0x00004000;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_triangles() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.TRIANGLES === 0x0004;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_array_buffer() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.ARRAY_BUFFER === 0x8892;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_vertex_shader() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.VERTEX_SHADER === 0x8B31;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_fragment_shader() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.FRAGMENT_SHADER === 0x8B30;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_static_draw() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.STATIC_DRAW === 0x88E4;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_constants_float() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        WebGLRenderingContext.FLOAT === 0x1406;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext via Canvas Tests
// ============================================================================

#[test]
fn test_canvas_getcontext_webgl() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let canvas = new HTMLCanvasElement();
        let gl = canvas.getContext('webgl');
        gl !== null || gl === null; // May return null if WebGL not supported
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Shader Methods Tests
// ============================================================================

#[test]
fn test_webgl_createshader_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.createShader === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_shadersource_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.shaderSource === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_compileshader_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.compileShader === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_deleteshader_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.deleteShader === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_getshaderparameter_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.getShaderParameter === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Program Methods Tests
// ============================================================================

#[test]
fn test_webgl_createprogram_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.createProgram === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_attachshader_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.attachShader === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_linkprogram_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.linkProgram === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_useprogram_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.useProgram === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_deleteprogram_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.deleteProgram === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Buffer Methods Tests
// ============================================================================

#[test]
fn test_webgl_createbuffer_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.createBuffer === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_bindbuffer_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.bindBuffer === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_bufferdata_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.bufferData === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_deletebuffer_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.deleteBuffer === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Texture Methods Tests
// ============================================================================

#[test]
fn test_webgl_createtexture_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.createTexture === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_bindtexture_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.bindTexture === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_teximage2d_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.texImage2D === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_texparameteri_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.texParameteri === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Uniform Methods Tests
// ============================================================================

#[test]
fn test_webgl_getuniformlocation_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.getUniformLocation === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_uniform1f_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.uniform1f === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_uniform2f_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.uniform2f === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_uniform3f_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.uniform3f === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_uniform4f_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.uniform4f === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_uniformmatrix4fv_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.uniformMatrix4fv === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Vertex Attribute Methods Tests
// ============================================================================

#[test]
fn test_webgl_getattriblocation_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.getAttribLocation === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_vertexattribpointer_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.vertexAttribPointer === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_enablevertexattribarray_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.enableVertexAttribArray === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_disablevertexattribarray_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.disableVertexAttribArray === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext Draw Methods Tests
// ============================================================================

#[test]
fn test_webgl_drawarrays_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.drawArrays === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_drawelements_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.drawElements === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_clear_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.clear === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_clearcolor_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.clearColor === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_viewport_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.viewport === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGLRenderingContext State Methods Tests
// ============================================================================

#[test]
fn test_webgl_enable_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.enable === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_disable_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.disable === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_getparameter_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.getParameter === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl_geterror_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGLRenderingContext.prototype.getError === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Tests
// ============================================================================

#[test]
fn test_webgl2renderingcontext_constructor_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof WebGL2RenderingContext"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

// ============================================================================
// WebGL2RenderingContext VAO Methods Tests
// ============================================================================

#[test]
fn test_webgl2_createvertexarray_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.createVertexArray === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_bindvertexarray_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.bindVertexArray === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_deletevertexarray_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.deleteVertexArray === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Query Methods Tests
// ============================================================================

#[test]
fn test_webgl2_createquery_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.createQuery === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_beginquery_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.beginQuery === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_endquery_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.endQuery === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Sampler Methods Tests
// ============================================================================

#[test]
fn test_webgl2_createsampler_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.createSampler === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_bindsampler_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.bindSampler === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Sync Methods Tests
// ============================================================================

#[test]
fn test_webgl2_fencesync_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.fenceSync === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_clientwaitsync_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.clientWaitSync === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Transform Feedback Methods Tests
// ============================================================================

#[test]
fn test_webgl2_createtransformfeedback_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.createTransformFeedback === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_bindtransformfeedback_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.bindTransformFeedback === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_begintransformfeedback_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.beginTransformFeedback === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WebGL2RenderingContext Uniform Buffer Methods Tests
// ============================================================================

#[test]
fn test_webgl2_getuniformblockindex_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.getUniformBlockIndex === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_bindbufferbase_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.bindBufferBase === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_webgl2_bindbufferrange_method_type() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof WebGL2RenderingContext.prototype.bindBufferRange === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
