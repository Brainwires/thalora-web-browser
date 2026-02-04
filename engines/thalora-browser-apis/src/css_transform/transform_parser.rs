//! CSS Transform Property Parser
//!
//! Parses CSS transform property values into a list of transform functions.
//!
//! Supported transform functions:
//! - `matrix(a, b, c, d, e, f)` - 2D affine transform
//! - `matrix3d(...)` - 3D transform (16 values)
//! - `translate(x, y?)`, `translateX(x)`, `translateY(y)`, `translateZ(z)`, `translate3d(x, y, z)`
//! - `scale(x, y?)`, `scaleX(x)`, `scaleY(y)`, `scaleZ(z)`, `scale3d(x, y, z)`
//! - `rotate(angle)`, `rotateX(angle)`, `rotateY(angle)`, `rotateZ(angle)`, `rotate3d(x, y, z, angle)`
//! - `skew(ax, ay?)`, `skewX(angle)`, `skewY(angle)`
//! - `perspective(d)`

use super::matrix4::{deg_to_rad, grad_to_rad, turn_to_rad, Matrix4};

/// A single CSS transform function.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    /// 2D matrix: matrix(a, b, c, d, e, f)
    Matrix([f64; 6]),
    /// 3D matrix: matrix3d(16 values)
    Matrix3d([f64; 16]),
    /// Translate in X and Y: translate(x, y) or translate(x)
    Translate(f64, f64),
    /// Translate in X only: translateX(x)
    TranslateX(f64),
    /// Translate in Y only: translateY(y)
    TranslateY(f64),
    /// Translate in Z only: translateZ(z)
    TranslateZ(f64),
    /// Translate in 3D: translate3d(x, y, z)
    Translate3d(f64, f64, f64),
    /// Uniform or 2D scale: scale(x, y) or scale(x)
    Scale(f64, f64),
    /// Scale in X only: scaleX(x)
    ScaleX(f64),
    /// Scale in Y only: scaleY(y)
    ScaleY(f64),
    /// Scale in Z only: scaleZ(z)
    ScaleZ(f64),
    /// 3D scale: scale3d(x, y, z)
    Scale3d(f64, f64, f64),
    /// 2D rotation (around Z): rotate(angle)
    Rotate(f64),
    /// Rotation around X axis: rotateX(angle)
    RotateX(f64),
    /// Rotation around Y axis: rotateY(angle)
    RotateY(f64),
    /// Rotation around Z axis: rotateZ(angle)
    RotateZ(f64),
    /// 3D rotation: rotate3d(x, y, z, angle)
    Rotate3d(f64, f64, f64, f64),
    /// Skew: skew(ax, ay) or skew(ax)
    Skew(f64, f64),
    /// Skew in X: skewX(angle)
    SkewX(f64),
    /// Skew in Y: skewY(angle)
    SkewY(f64),
    /// Perspective: perspective(d)
    Perspective(f64),
}

impl TransformFunction {
    /// Convert this transform function to a 4x4 matrix.
    pub fn to_matrix(&self) -> Matrix4 {
        match self {
            TransformFunction::Matrix(v) => Matrix4::from_matrix2d(*v),
            TransformFunction::Matrix3d(v) => Matrix4::from_matrix3d(*v),
            TransformFunction::Translate(x, y) => Matrix4::translate(*x, *y, 0.0),
            TransformFunction::TranslateX(x) => Matrix4::translate(*x, 0.0, 0.0),
            TransformFunction::TranslateY(y) => Matrix4::translate(0.0, *y, 0.0),
            TransformFunction::TranslateZ(z) => Matrix4::translate(0.0, 0.0, *z),
            TransformFunction::Translate3d(x, y, z) => Matrix4::translate(*x, *y, *z),
            TransformFunction::Scale(x, y) => Matrix4::scale(*x, *y, 1.0),
            TransformFunction::ScaleX(x) => Matrix4::scale(*x, 1.0, 1.0),
            TransformFunction::ScaleY(y) => Matrix4::scale(1.0, *y, 1.0),
            TransformFunction::ScaleZ(z) => Matrix4::scale(1.0, 1.0, *z),
            TransformFunction::Scale3d(x, y, z) => Matrix4::scale(*x, *y, *z),
            TransformFunction::Rotate(angle) => Matrix4::rotate_z(*angle),
            TransformFunction::RotateX(angle) => Matrix4::rotate_x(*angle),
            TransformFunction::RotateY(angle) => Matrix4::rotate_y(*angle),
            TransformFunction::RotateZ(angle) => Matrix4::rotate_z(*angle),
            TransformFunction::Rotate3d(x, y, z, angle) => Matrix4::rotate_3d(*x, *y, *z, *angle),
            TransformFunction::Skew(ax, ay) => {
                // skew(ax, ay) = skewX(ax) * skewY(ay)
                let mx = Matrix4::skew_x(*ax);
                let my = Matrix4::skew_y(*ay);
                mx.multiply(&my)
            }
            TransformFunction::SkewX(angle) => Matrix4::skew_x(*angle),
            TransformFunction::SkewY(angle) => Matrix4::skew_y(*angle),
            TransformFunction::Perspective(d) => Matrix4::perspective(*d),
        }
    }
}

/// Transform origin with resolved values.
#[derive(Debug, Clone, PartialEq)]
pub struct TransformOrigin {
    /// X coordinate of origin (in pixels)
    pub x: f64,
    /// Y coordinate of origin (in pixels)
    pub y: f64,
    /// Z coordinate of origin (in pixels)
    pub z: f64,
}

impl Default for TransformOrigin {
    fn default() -> Self {
        // Default is center: 50% 50% 0
        // The actual pixel values depend on element size, so default to 0
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Parse a CSS transform property value into a list of transform functions.
///
/// # Arguments
///
/// * `transform` - The CSS transform property value (e.g., "rotate(45deg) scale(2)")
///
/// # Returns
///
/// A vector of `TransformFunction` in the order they appear in the CSS.
/// Returns an empty vector for "none" or invalid input.
pub fn parse_transform(transform: &str) -> Vec<TransformFunction> {
    let transform = transform.trim();

    if transform.is_empty() || transform.eq_ignore_ascii_case("none") {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut chars = transform.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        // Read function name
        let mut func_name = String::new();
        while chars.peek().map_or(false, |c| c.is_alphanumeric() || *c == '-' || *c == '_') {
            func_name.push(chars.next().unwrap());
        }

        // Skip whitespace before '('
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        // Expect '('
        if chars.peek() != Some(&'(') {
            // Invalid syntax, skip to next potential function
            continue;
        }
        chars.next(); // consume '('

        // Read arguments until ')'
        let mut args_str = String::new();
        let mut paren_depth = 1;
        while let Some(&c) = chars.peek() {
            if c == '(' {
                paren_depth += 1;
            } else if c == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    chars.next(); // consume closing ')'
                    break;
                }
            }
            args_str.push(chars.next().unwrap());
        }

        // Parse the transform function
        if let Some(tf) = parse_transform_function(&func_name, &args_str) {
            result.push(tf);
        }
    }

    result
}

/// Parse a single transform function from its name and arguments string.
fn parse_transform_function(name: &str, args: &str) -> Option<TransformFunction> {
    let name_lower = name.to_ascii_lowercase();
    let values = parse_arguments(args);

    match name_lower.as_str() {
        "matrix" => {
            if values.len() >= 6 {
                Some(TransformFunction::Matrix([
                    values[0], values[1], values[2], values[3], values[4], values[5],
                ]))
            } else {
                None
            }
        }
        "matrix3d" => {
            if values.len() >= 16 {
                let mut arr = [0.0; 16];
                arr.copy_from_slice(&values[..16]);
                Some(TransformFunction::Matrix3d(arr))
            } else {
                None
            }
        }
        "translate" => {
            let x = values.first().copied().unwrap_or(0.0);
            let y = values.get(1).copied().unwrap_or(0.0);
            Some(TransformFunction::Translate(x, y))
        }
        "translatex" => {
            let x = values.first().copied().unwrap_or(0.0);
            Some(TransformFunction::TranslateX(x))
        }
        "translatey" => {
            let y = values.first().copied().unwrap_or(0.0);
            Some(TransformFunction::TranslateY(y))
        }
        "translatez" => {
            let z = values.first().copied().unwrap_or(0.0);
            Some(TransformFunction::TranslateZ(z))
        }
        "translate3d" => {
            let x = values.first().copied().unwrap_or(0.0);
            let y = values.get(1).copied().unwrap_or(0.0);
            let z = values.get(2).copied().unwrap_or(0.0);
            Some(TransformFunction::Translate3d(x, y, z))
        }
        "scale" => {
            let x = values.first().copied().unwrap_or(1.0);
            let y = values.get(1).copied().unwrap_or(x); // Default y = x for uniform scale
            Some(TransformFunction::Scale(x, y))
        }
        "scalex" => {
            let x = values.first().copied().unwrap_or(1.0);
            Some(TransformFunction::ScaleX(x))
        }
        "scaley" => {
            let y = values.first().copied().unwrap_or(1.0);
            Some(TransformFunction::ScaleY(y))
        }
        "scalez" => {
            let z = values.first().copied().unwrap_or(1.0);
            Some(TransformFunction::ScaleZ(z))
        }
        "scale3d" => {
            let x = values.first().copied().unwrap_or(1.0);
            let y = values.get(1).copied().unwrap_or(1.0);
            let z = values.get(2).copied().unwrap_or(1.0);
            Some(TransformFunction::Scale3d(x, y, z))
        }
        "rotate" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::Rotate(angle))
        }
        "rotatex" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::RotateX(angle))
        }
        "rotatey" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::RotateY(angle))
        }
        "rotatez" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::RotateZ(angle))
        }
        "rotate3d" => {
            // rotate3d(x, y, z, angle)
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() >= 4 {
                let x = parse_number(parts[0].trim()).unwrap_or(0.0);
                let y = parse_number(parts[1].trim()).unwrap_or(0.0);
                let z = parse_number(parts[2].trim()).unwrap_or(1.0);
                let angle = parse_angle(parts[3].trim());
                Some(TransformFunction::Rotate3d(x, y, z, angle))
            } else {
                None
            }
        }
        "skew" => {
            let parts: Vec<&str> = args.split(',').collect();
            let ax = parse_angle(parts.first().map(|s| s.trim()).unwrap_or("0"));
            let ay = if parts.len() >= 2 {
                parse_angle(parts[1].trim())
            } else {
                0.0
            };
            Some(TransformFunction::Skew(ax, ay))
        }
        "skewx" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::SkewX(angle))
        }
        "skewy" => {
            let angle = parse_angle_from_args(&args);
            Some(TransformFunction::SkewY(angle))
        }
        "perspective" => {
            let d = values.first().copied().unwrap_or(0.0);
            if d > 0.0 {
                Some(TransformFunction::Perspective(d))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse arguments string into numeric values.
/// Handles units like px, em, %, deg, rad, grad, turn.
fn parse_arguments(args: &str) -> Vec<f64> {
    args.split(',')
        .filter_map(|s| parse_value_with_unit(s.trim()))
        .collect()
}

/// Parse a single angle from the args string.
fn parse_angle_from_args(args: &str) -> f64 {
    parse_angle(args.trim())
}

/// Parse an angle value with unit (deg, rad, grad, turn).
/// Returns radians.
fn parse_angle(s: &str) -> f64 {
    let s = s.trim().to_ascii_lowercase();

    // IMPORTANT: Check "grad" before "rad" because "grad" ends with "rad"!
    if let Some(grad) = s.strip_suffix("grad") {
        return grad_to_rad(parse_number(grad).unwrap_or(0.0));
    }
    if let Some(deg) = s.strip_suffix("deg") {
        return deg_to_rad(parse_number(deg).unwrap_or(0.0));
    }
    if let Some(rad) = s.strip_suffix("rad") {
        return parse_number(rad).unwrap_or(0.0);
    }
    if let Some(turn) = s.strip_suffix("turn") {
        return turn_to_rad(parse_number(turn).unwrap_or(0.0));
    }

    // Unitless number - treat as degrees (CSS default for some functions)
    // Actually, CSS requires units for angles. Treat unitless as radians for safety.
    parse_number(&s).unwrap_or(0.0)
}

/// Parse a length/percentage value with unit.
/// Returns the numeric value (percentages are not resolved here).
fn parse_value_with_unit(s: &str) -> Option<f64> {
    let s = s.trim().to_ascii_lowercase();

    // Handle percentages (just return the number, caller handles resolution)
    if let Some(pct) = s.strip_suffix('%') {
        return parse_number(pct);
    }

    // Handle length units
    for unit in &["px", "em", "rem", "vh", "vw", "vmin", "vmax", "cm", "mm", "in", "pt", "pc"] {
        if let Some(val) = s.strip_suffix(unit) {
            let num = parse_number(val)?;
            // Convert to pixels (approximate)
            return Some(match *unit {
                "px" => num,
                "em" | "rem" => num * 16.0, // Assume 16px base font
                "vh" | "vw" => num * 10.0,  // Assume ~1000px viewport
                "vmin" | "vmax" => num * 10.0,
                "cm" => num * 37.795,
                "mm" => num * 3.7795,
                "in" => num * 96.0,
                "pt" => num * 1.333,
                "pc" => num * 16.0,
                _ => num,
            });
        }
    }

    // Handle angle units (for values that might be angles)
    if s.ends_with("deg") || s.ends_with("rad") || s.ends_with("grad") || s.ends_with("turn") {
        return Some(parse_angle(&s));
    }

    // Unitless number
    parse_number(&s)
}

/// Parse a number from a string.
fn parse_number(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

/// Parse the CSS transform-origin property value.
///
/// # Arguments
///
/// * `origin` - The CSS transform-origin value (e.g., "50% 50%", "center", "left top")
/// * `element_width` - Element width for percentage resolution
/// * `element_height` - Element height for percentage resolution
///
/// # Returns
///
/// A `TransformOrigin` with resolved pixel values.
pub fn parse_transform_origin(origin: &str, element_width: f64, element_height: f64) -> TransformOrigin {
    let origin = origin.trim();

    if origin.is_empty() {
        // Default: center center 0px (50% 50% 0)
        return TransformOrigin {
            x: element_width / 2.0,
            y: element_height / 2.0,
            z: 0.0,
        };
    }

    let parts: Vec<&str> = origin.split_whitespace().collect();

    let x = parts.first().map(|s| parse_origin_value(s, element_width, true))
        .unwrap_or(element_width / 2.0);
    let y = parts.get(1).map(|s| parse_origin_value(s, element_height, false))
        .unwrap_or(element_height / 2.0);
    let z = parts.get(2).map(|s| parse_length_value(s)).unwrap_or(0.0);

    TransformOrigin { x, y, z }
}

/// Parse a single origin value (x or y).
fn parse_origin_value(s: &str, dimension: f64, is_horizontal: bool) -> f64 {
    let s = s.trim().to_ascii_lowercase();

    // Handle keywords
    match s.as_str() {
        "left" if is_horizontal => return 0.0,
        "right" if is_horizontal => return dimension,
        "center" => return dimension / 2.0,
        "top" if !is_horizontal => return 0.0,
        "bottom" if !is_horizontal => return dimension,
        _ => {}
    }

    // Handle percentage
    if let Some(pct_str) = s.strip_suffix('%') {
        if let Some(pct) = parse_number(pct_str) {
            return dimension * pct / 100.0;
        }
    }

    // Handle length
    parse_length_value(&s)
}

/// Parse a length value (for z-origin or absolute values).
fn parse_length_value(s: &str) -> f64 {
    let s = s.trim().to_ascii_lowercase();

    // Handle length units
    for unit in &["px", "em", "rem", "vh", "vw", "cm", "mm", "in", "pt", "pc"] {
        if let Some(val) = s.strip_suffix(unit) {
            let num = parse_number(val).unwrap_or(0.0);
            return match *unit {
                "px" => num,
                "em" | "rem" => num * 16.0,
                "vh" | "vw" => num * 10.0,
                "cm" => num * 37.795,
                "mm" => num * 3.7795,
                "in" => num * 96.0,
                "pt" => num * 1.333,
                "pc" => num * 16.0,
                _ => num,
            };
        }
    }

    // Unitless (treated as px)
    parse_number(&s).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_parse_empty() {
        assert!(parse_transform("").is_empty());
        assert!(parse_transform("none").is_empty());
        assert!(parse_transform("  NONE  ").is_empty());
    }

    #[test]
    fn test_parse_translate() {
        let result = parse_transform("translate(10px, 20px)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Translate(x, y) => {
                assert!((x - 10.0).abs() < EPSILON);
                assert!((y - 20.0).abs() < EPSILON);
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_parse_translate_single_value() {
        let result = parse_transform("translate(10px)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Translate(x, y) => {
                assert!((x - 10.0).abs() < EPSILON);
                assert!((y - 0.0).abs() < EPSILON);
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_parse_rotate() {
        let result = parse_transform("rotate(45deg)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Rotate(angle) => {
                assert!((angle - PI / 4.0).abs() < EPSILON);
            }
            _ => panic!("Expected Rotate"),
        }
    }

    #[test]
    fn test_parse_rotate_rad() {
        let result = parse_transform("rotate(1.5708rad)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Rotate(angle) => {
                assert!((angle - 1.5708).abs() < EPSILON);
            }
            _ => panic!("Expected Rotate"),
        }
    }

    #[test]
    fn test_parse_scale() {
        let result = parse_transform("scale(2)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Scale(x, y) => {
                assert!((x - 2.0).abs() < EPSILON);
                assert!((y - 2.0).abs() < EPSILON); // y defaults to x
            }
            _ => panic!("Expected Scale"),
        }
    }

    #[test]
    fn test_parse_scale_two_values() {
        let result = parse_transform("scale(2, 3)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Scale(x, y) => {
                assert!((x - 2.0).abs() < EPSILON);
                assert!((y - 3.0).abs() < EPSILON);
            }
            _ => panic!("Expected Scale"),
        }
    }

    #[test]
    fn test_parse_matrix() {
        let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Matrix(values) => {
                assert!((values[0] - 1.0).abs() < EPSILON);
                assert!((values[4] - 10.0).abs() < EPSILON);
                assert!((values[5] - 20.0).abs() < EPSILON);
            }
            _ => panic!("Expected Matrix"),
        }
    }

    #[test]
    fn test_parse_matrix3d() {
        let result = parse_transform(
            "matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)"
        );
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Matrix3d(values) => {
                // Identity matrix
                assert!((values[0] - 1.0).abs() < EPSILON);
                assert!((values[5] - 1.0).abs() < EPSILON);
                assert!((values[10] - 1.0).abs() < EPSILON);
                assert!((values[15] - 1.0).abs() < EPSILON);
            }
            _ => panic!("Expected Matrix3d"),
        }
    }

    #[test]
    fn test_parse_multiple_transforms() {
        let result = parse_transform("translate(10px, 20px) rotate(45deg) scale(2)");
        assert_eq!(result.len(), 3);
        assert!(matches!(&result[0], TransformFunction::Translate(_, _)));
        assert!(matches!(&result[1], TransformFunction::Rotate(_)));
        assert!(matches!(&result[2], TransformFunction::Scale(_, _)));
    }

    #[test]
    fn test_parse_skew() {
        let result = parse_transform("skewX(30deg)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::SkewX(angle) => {
                assert!((angle - deg_to_rad(30.0)).abs() < EPSILON);
            }
            _ => panic!("Expected SkewX"),
        }
    }

    #[test]
    fn test_parse_perspective() {
        let result = parse_transform("perspective(1000px)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Perspective(d) => {
                assert!((d - 1000.0).abs() < EPSILON);
            }
            _ => panic!("Expected Perspective"),
        }
    }

    #[test]
    fn test_parse_rotate3d() {
        let result = parse_transform("rotate3d(1, 0, 0, 90deg)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Rotate3d(x, y, z, angle) => {
                assert!((x - 1.0).abs() < EPSILON);
                assert!((y - 0.0).abs() < EPSILON);
                assert!((z - 0.0).abs() < EPSILON);
                assert!((angle - PI / 2.0).abs() < EPSILON);
            }
            _ => panic!("Expected Rotate3d"),
        }
    }

    #[test]
    fn test_transform_origin_default() {
        let origin = parse_transform_origin("", 100.0, 200.0);
        assert!((origin.x - 50.0).abs() < EPSILON);
        assert!((origin.y - 100.0).abs() < EPSILON);
        assert!((origin.z - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_origin_center() {
        let origin = parse_transform_origin("center center", 100.0, 200.0);
        assert!((origin.x - 50.0).abs() < EPSILON);
        assert!((origin.y - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_origin_percent() {
        let origin = parse_transform_origin("25% 75%", 100.0, 200.0);
        assert!((origin.x - 25.0).abs() < EPSILON);
        assert!((origin.y - 150.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_origin_keywords() {
        let origin = parse_transform_origin("left top", 100.0, 200.0);
        assert!((origin.x - 0.0).abs() < EPSILON);
        assert!((origin.y - 0.0).abs() < EPSILON);

        let origin = parse_transform_origin("right bottom", 100.0, 200.0);
        assert!((origin.x - 100.0).abs() < EPSILON);
        assert!((origin.y - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_origin_with_z() {
        let origin = parse_transform_origin("50% 50% 100px", 100.0, 200.0);
        assert!((origin.x - 50.0).abs() < EPSILON);
        assert!((origin.y - 100.0).abs() < EPSILON);
        assert!((origin.z - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_to_matrix_translate() {
        let tf = TransformFunction::Translate(10.0, 20.0);
        let m = tf.to_matrix();
        let (x, y, z) = m.transform_point(0.0, 0.0, 0.0);
        assert!((x - 10.0).abs() < EPSILON);
        assert!((y - 20.0).abs() < EPSILON);
        assert!((z - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_transform_to_matrix_rotate() {
        let tf = TransformFunction::Rotate(deg_to_rad(90.0));
        let m = tf.to_matrix();
        let (x, y, _) = m.transform_point(1.0, 0.0, 0.0);
        assert!((x - 0.0).abs() < EPSILON, "x = {}", x);
        assert!((y - 1.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_transform_to_matrix_scale() {
        let tf = TransformFunction::Scale(2.0, 3.0);
        let m = tf.to_matrix();
        let (x, y, _) = m.transform_point(1.0, 1.0, 0.0);
        assert!((x - 2.0).abs() < EPSILON);
        assert!((y - 3.0).abs() < EPSILON);
    }

    #[test]
    fn test_parse_turn_unit() {
        let result = parse_transform("rotate(0.25turn)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Rotate(angle) => {
                // 0.25 turn = 90 degrees = PI/2 radians
                assert!((angle - PI / 2.0).abs() < EPSILON);
            }
            _ => panic!("Expected Rotate"),
        }
    }

    #[test]
    fn test_parse_grad_unit() {
        let result = parse_transform("rotate(100grad)");
        assert_eq!(result.len(), 1);
        match &result[0] {
            TransformFunction::Rotate(angle) => {
                // 100 gradians = 90 degrees = PI/2 radians
                assert!((angle - PI / 2.0).abs() < EPSILON);
            }
            _ => panic!("Expected Rotate"),
        }
    }
}
