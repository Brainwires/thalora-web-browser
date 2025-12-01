//! Comprehensive test suite for Canvas 2D APIs
//! Tests HTMLCanvasElement, CanvasRenderingContext2D, Path2D, and OffscreenCanvas

use boa_engine::{Context, Source, JsValue};
use boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// HTMLCanvasElement Constructor Tests
// ============================================================================

#[test]
fn test_htmlcanvaselement_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof HTMLCanvasElement")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_htmlcanvaselement_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        canvas !== null && canvas !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_default_dimensions() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        canvas.width === 300 && canvas.height === 150;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_custom_dimensions() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement(640, 480);
        canvas.width === 640 && canvas.height === 480;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_width_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        canvas.width = 500;
        canvas.width === 500;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_height_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        canvas.height = 400;
        canvas.height === 400;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLCanvasElement getContext Tests
// ============================================================================

#[test]
fn test_htmlcanvaselement_getcontext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof HTMLCanvasElement.prototype.getContext === 'function' ||
        typeof new HTMLCanvasElement().getContext === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_getcontext_2d() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx !== null && ctx !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_getcontext_returns_same_context() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx1 = canvas.getContext('2d');
        let ctx2 = canvas.getContext('2d');
        ctx1 === ctx2;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_todataurl_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        typeof canvas.toDataURL === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlcanvaselement_todataurl_returns_string() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement(10, 10);
        let dataUrl = canvas.toDataURL();
        typeof dataUrl === 'string' && dataUrl.startsWith('data:image/png');
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Constructor Tests
// ============================================================================

#[test]
fn test_canvasrenderingcontext2d_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof CanvasRenderingContext2D")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_canvasrenderingcontext2d_canvas_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.canvas === canvas;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Style Properties Tests
// ============================================================================

#[test]
fn test_context2d_fillstyle_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.fillStyle === '#000000' || ctx.fillStyle === 'black' || ctx.fillStyle === '#000';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_fillstyle_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.fillStyle = '#ff0000';
        ctx.fillStyle === '#ff0000' || ctx.fillStyle === 'red';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_strokestyle_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.strokeStyle = 'blue';
        ctx.strokeStyle !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_linewidth_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.lineWidth === 1;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_linewidth_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.lineWidth = 5;
        ctx.lineWidth === 5;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_globalalpha_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.globalAlpha === 1;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_globalalpha_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.globalAlpha = 0.5;
        ctx.globalAlpha === 0.5;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Text Properties Tests
// ============================================================================

#[test]
fn test_context2d_font_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.font === '10px sans-serif';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_font_setter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.font = '16px Arial';
        ctx.font === '16px Arial';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_textalign_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.textAlign === 'start';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_textbaseline_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.textBaseline === 'alphabetic';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D State Methods Tests
// ============================================================================

#[test]
fn test_context2d_save_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.save === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_restore_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.restore === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_save_restore_preserves_state() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        ctx.fillStyle = '#ff0000';
        ctx.save();
        ctx.fillStyle = '#00ff00';
        ctx.restore();
        ctx.fillStyle === '#ff0000' || ctx.fillStyle === 'red';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Path Methods Tests
// ============================================================================

#[test]
fn test_context2d_beginpath_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.beginPath === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_closepath_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.closePath === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_moveto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.moveTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_lineto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.lineTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_arc_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.arc === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_rect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.rect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_beziercurveto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.bezierCurveTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_quadraticcurveto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.quadraticCurveTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Drawing Methods Tests
// ============================================================================

#[test]
fn test_context2d_fill_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.fill === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_stroke_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.stroke === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_fillrect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.fillRect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_strokerect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.strokeRect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_clearrect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.clearRect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Transform Methods Tests
// ============================================================================

#[test]
fn test_context2d_scale_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.scale === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_rotate_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.rotate === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_translate_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.translate === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_transform_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.transform === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_settransform_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.setTransform === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_resettransform_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.resetTransform === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Text Methods Tests
// ============================================================================

#[test]
fn test_context2d_filltext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.fillText === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_stroketext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.strokeText === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_measuretext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.measureText === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_measuretext_returns_object() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        let metrics = ctx.measureText('Hello');
        typeof metrics === 'object' && metrics !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_measuretext_width_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        let metrics = ctx.measureText('Hello');
        typeof metrics.width === 'number' && metrics.width >= 0;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CanvasRenderingContext2D Image Data Methods Tests
// ============================================================================

#[test]
fn test_context2d_getimagedata_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.getImageData === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_putimagedata_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.putImageData === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_context2d_createimagedata_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement();
        let ctx = canvas.getContext('2d');
        typeof ctx.createImageData === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Path2D Tests
// ============================================================================

#[test]
fn test_path2d_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Path2D")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_path2d_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let path = new Path2D();
        path !== null && path !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_path2d_moveto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let path = new Path2D();
        typeof path.moveTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_path2d_lineto_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let path = new Path2D();
        typeof path.lineTo === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_path2d_closepath_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let path = new Path2D();
        typeof path.closePath === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// OffscreenCanvas Tests
// ============================================================================

#[test]
fn test_offscreencanvas_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof OffscreenCanvas")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_offscreencanvas_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new OffscreenCanvas(100, 100);
        canvas !== null && canvas !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_offscreencanvas_width_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new OffscreenCanvas(200, 150);
        canvas.width === 200;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_offscreencanvas_height_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new OffscreenCanvas(200, 150);
        canvas.height === 150;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_offscreencanvas_getcontext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new OffscreenCanvas(100, 100);
        typeof canvas.getContext === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_canvas_draw_rectangle() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement(100, 100);
        let ctx = canvas.getContext('2d');
        ctx.fillStyle = 'red';
        ctx.fillRect(10, 10, 50, 50);
        true; // Test that drawing doesn't throw
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_canvas_draw_path() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement(100, 100);
        let ctx = canvas.getContext('2d');
        ctx.beginPath();
        ctx.moveTo(10, 10);
        ctx.lineTo(50, 50);
        ctx.lineTo(10, 50);
        ctx.closePath();
        ctx.fill();
        true; // Test that path drawing doesn't throw
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_canvas_state_stack() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let canvas = new HTMLCanvasElement(100, 100);
        let ctx = canvas.getContext('2d');
        ctx.globalAlpha = 0.5;
        ctx.save();
        ctx.globalAlpha = 0.25;
        ctx.save();
        ctx.globalAlpha = 0.1;
        ctx.restore();
        let alpha1 = ctx.globalAlpha;
        ctx.restore();
        let alpha2 = ctx.globalAlpha;
        alpha1 === 0.25 && alpha2 === 0.5;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
