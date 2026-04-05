//! SVG DOM element types
//!
//! Provides SVG-specific element classes that extend the base Element.
//! These ensure that querySelector('svg path') returns proper SVG elements
//! with SVG-specific properties like viewBox, pathLength, etc.
//!
//! Per the SVG spec, all SVG elements share a common SVGElement base
//! and are in the SVG namespace (http://www.w3.org/2000/svg).

use boa_engine::{
    Context, JsResult, NativeFunction, js_string,
    object::{FunctionObjectBuilder, JsObject},
    value::JsValue,
};

/// SVG namespace URI
pub const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// SVG element tag names that should be recognized as SVG elements
pub const SVG_ELEMENT_TAGS: &[&str] = &[
    "svg",
    "g",
    "defs",
    "symbol",
    "use",
    // Shape elements
    "path",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polyline",
    "polygon",
    // Text elements
    "text",
    "tspan",
    "textPath",
    // Structural elements
    "clipPath",
    "mask",
    "pattern",
    "marker",
    // Gradient elements
    "linearGradient",
    "radialGradient",
    "stop",
    // Filter elements
    "filter",
    "feBlend",
    "feColorMatrix",
    "feGaussianBlur",
    "feOffset",
    "feMerge",
    "feMergeNode",
    "feFlood",
    "feComposite",
    // Other
    "foreignObject",
    "image",
    "title",
    "desc",
    "metadata",
    "animate",
    "animateTransform",
    "animateMotion",
    "set",
];

/// Check if a tag name is an SVG element
pub fn is_svg_element(tag: &str) -> bool {
    SVG_ELEMENT_TAGS.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

/// Register SVG-related globals and helpers in the JS context
pub fn register_svg_globals(context: &mut Context) -> JsResult<()> {
    // Register SVGElement as a global (inherits from Element)
    let svg_element_ctor = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(svg_element_constructor),
    )
    .name(js_string!("SVGElement"))
    .length(0)
    .build();

    let global = context.global_object();
    global.set(js_string!("SVGElement"), svg_element_ctor, false, context)?;

    // Register SVGSVGElement
    let svg_svg_ctor = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(svg_svg_element_constructor),
    )
    .name(js_string!("SVGSVGElement"))
    .length(0)
    .build();
    global.set(js_string!("SVGSVGElement"), svg_svg_ctor, false, context)?;

    Ok(())
}

fn svg_element_constructor(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a plain Element and tag it as SVG
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = element_constructor.construct(&[], Some(&element_constructor), context)?;

    // Set namespace
    element.set(
        js_string!("namespaceURI"),
        js_string!(SVG_NAMESPACE),
        false,
        context,
    )?;

    Ok(element.into())
}

fn svg_svg_element_constructor(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let element_constructor = context.intrinsics().constructors().element().constructor();
    let element = element_constructor.construct(&[], Some(&element_constructor), context)?;

    element.set(
        js_string!("namespaceURI"),
        js_string!(SVG_NAMESPACE),
        false,
        context,
    )?;
    element.set(js_string!("tagName"), js_string!("svg"), false, context)?;

    // SVGSVGElement-specific properties
    // viewBox returns an SVGAnimatedRect (simplified as object with baseVal)
    let view_box = JsObject::with_object_proto(context.intrinsics());
    let base_val = JsObject::with_object_proto(context.intrinsics());
    base_val.set(js_string!("x"), 0, false, context)?;
    base_val.set(js_string!("y"), 0, false, context)?;
    base_val.set(js_string!("width"), 0, false, context)?;
    base_val.set(js_string!("height"), 0, false, context)?;
    view_box.set(js_string!("baseVal"), base_val, false, context)?;
    element.set(js_string!("viewBox"), view_box, false, context)?;

    Ok(element.into())
}
