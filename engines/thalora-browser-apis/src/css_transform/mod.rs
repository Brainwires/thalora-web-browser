//! CSS Transform Module
//!
//! Provides CSS 3D transform parsing and coordinate transformation for accurate
//! click handling on transformed elements (e.g., Cloudflare Turnstile's rotated checkbox).

pub mod matrix4;
pub mod transform_parser;

pub use matrix4::Matrix4;
pub use transform_parser::{parse_transform, parse_transform_origin, TransformFunction, TransformOrigin};

/// Map screen coordinates to element-local coordinates accounting for CSS transforms.
///
/// This function is used to correctly target clicks on elements that have CSS 3D transforms
/// applied. When an element is rotated/skewed/scaled via CSS transform, the visual position
/// differs from the DOM bounding box position.
///
/// # Arguments
///
/// * `screen_x`, `screen_y` - The screen coordinates where the click should land visually
/// * `element_transform` - The CSS transform property value (e.g., "matrix3d(...)" or "rotate(45deg)")
/// * `transform_origin` - The CSS transform-origin property value (e.g., "50% 50% 0")
/// * `element_width`, `element_height` - The element's dimensions (for percentage-based origins)
/// * `element_x`, `element_y` - The element's position in the document
///
/// # Returns
///
/// A tuple `(local_x, local_y)` representing the coordinates in element-local space
/// that, when the transform is applied, will result in the visual `(screen_x, screen_y)` position.
pub fn screen_to_element_coords(
    screen_x: f64,
    screen_y: f64,
    element_transform: &str,
    transform_origin: &str,
    element_width: f64,
    element_height: f64,
    element_x: f64,
    element_y: f64,
) -> (f64, f64) {
    // Parse the transform functions
    let transforms = parse_transform(element_transform);
    if transforms.is_empty() {
        // No transform applied, return original coordinates relative to element
        return (screen_x - element_x, screen_y - element_y);
    }

    // Parse transform-origin
    let origin = parse_transform_origin(transform_origin, element_width, element_height);

    // Build the composite transform matrix
    let mut composite = Matrix4::identity();

    // CSS transforms apply left-to-right in CSS syntax, but mathematically
    // the transform is applied as: T(origin) * Transform * T(-origin)
    //
    // For a point p:
    // 1. Translate point to origin-centered space: p' = p - origin
    // 2. Apply transform: p'' = Transform * p'
    // 3. Translate back: p''' = p'' + origin
    //
    // As a matrix: result = T(origin) * Transform * T(-origin) * point
    //
    // For multiple transforms in CSS (left-to-right): transform: A B C
    // The effective transform is: T(origin) * A * B * C * T(-origin)

    let origin_translate = Matrix4::translate(origin.x, origin.y, origin.z);
    let origin_translate_back = Matrix4::translate(-origin.x, -origin.y, -origin.z);

    // Start with translate to origin
    composite = composite.multiply(&origin_translate);

    // Apply each transform (left-to-right as in CSS)
    for tf in &transforms {
        let m = tf.to_matrix();
        composite = composite.multiply(&m);
    }

    // Translate back from origin
    composite = composite.multiply(&origin_translate_back);

    // Now we need to map screen coordinates to element-local coordinates
    // The element is at (element_x, element_y) in screen space
    // The transform is applied around the origin within the element

    // Convert screen coordinates to element-relative coordinates
    let rel_x = screen_x - element_x;
    let rel_y = screen_y - element_y;

    // To find where to click in the untransformed element to hit (rel_x, rel_y) after transform,
    // we need to apply the inverse transform
    if let Some(inverse) = composite.inverse() {
        let (local_x, local_y, _local_z) = inverse.transform_point(rel_x, rel_y, 0.0);
        (local_x, local_y)
    } else {
        // Matrix is not invertible (degenerate transform), fall back to original
        (rel_x, rel_y)
    }
}

/// Map element-local coordinates to screen coordinates through the transform.
///
/// This is the forward direction: given a point in the untransformed element,
/// find where it appears on screen after the transform is applied.
pub fn element_to_screen_coords(
    local_x: f64,
    local_y: f64,
    element_transform: &str,
    transform_origin: &str,
    element_width: f64,
    element_height: f64,
    element_x: f64,
    element_y: f64,
) -> (f64, f64) {
    let transforms = parse_transform(element_transform);
    if transforms.is_empty() {
        return (local_x + element_x, local_y + element_y);
    }

    let origin = parse_transform_origin(transform_origin, element_width, element_height);

    let mut composite = Matrix4::identity();
    let origin_translate = Matrix4::translate(origin.x, origin.y, origin.z);
    let origin_translate_back = Matrix4::translate(-origin.x, -origin.y, -origin.z);

    // Same order as screen_to_element_coords: T(origin) * Transforms * T(-origin)
    composite = composite.multiply(&origin_translate);

    for tf in &transforms {
        let m = tf.to_matrix();
        composite = composite.multiply(&m);
    }

    composite = composite.multiply(&origin_translate_back);

    let (screen_rel_x, screen_rel_y, _) = composite.transform_point(local_x, local_y, 0.0);
    (screen_rel_x + element_x, screen_rel_y + element_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_transform() {
        let (x, y) = screen_to_element_coords(
            150.0, 150.0,
            "",
            "50% 50%",
            100.0, 100.0,
            100.0, 100.0,
        );
        assert!((x - 50.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_translate_transform() {
        // Element at (100, 100), translate(50px, 50px)
        // Visual position is shifted by +50, +50
        // So clicking at screen (200, 200) should map to element-local (50, 50)
        let (x, y) = screen_to_element_coords(
            200.0, 200.0,
            "translate(50px, 50px)",
            "0 0",
            100.0, 100.0,
            100.0, 100.0,
        );
        assert!((x - 50.0).abs() < 0.001, "x = {}", x);
        assert!((y - 50.0).abs() < 0.001, "y = {}", y);
    }

    #[test]
    fn test_rotate_transform() {
        // Element at origin, 100x100, rotated 90deg around center
        // The center (50, 50) stays at (50, 50)
        // Point (100, 50) - right edge center - rotates to (50, 100)
        let (x, y) = screen_to_element_coords(
            50.0, 100.0,  // Screen position where (100, 50) lands after 90deg rotation
            "rotate(90deg)",
            "50% 50%",    // Default center origin
            100.0, 100.0,
            0.0, 0.0,
        );
        // Should map back to (100, 50) in element-local coords
        assert!((x - 100.0).abs() < 0.1, "x = {}", x);
        assert!((y - 50.0).abs() < 0.1, "y = {}", y);
    }

    #[test]
    fn test_scale_transform() {
        // Element at (100, 100), scaled 2x around center
        // Element center at element-local (50, 50)
        // After scale, visual extent is 200x200 centered at element center
        // Element-local (0, 0) appears at screen (100-50, 100-50) = (50, 50)
        // Element-local (100, 100) appears at screen (100+150, 100+150) = (250, 250)
        let (x, y) = screen_to_element_coords(
            250.0, 250.0,
            "scale(2)",
            "50% 50%",
            100.0, 100.0,
            100.0, 100.0,
        );
        assert!((x - 100.0).abs() < 0.1, "x = {}", x);
        assert!((y - 100.0).abs() < 0.1, "y = {}", y);
    }

    #[test]
    fn test_matrix3d_identity() {
        // Identity matrix should not change coordinates
        let (x, y) = screen_to_element_coords(
            150.0, 150.0,
            "matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)",
            "50% 50%",
            100.0, 100.0,
            100.0, 100.0,
        );
        assert!((x - 50.0).abs() < 0.001, "x = {}", x);
        assert!((y - 50.0).abs() < 0.001, "y = {}", y);
    }

    #[test]
    fn test_roundtrip_transform() {
        // Forward then inverse should give back original coordinates
        let element_transform = "rotate(30deg) scale(1.5) translate(10px, 20px)";
        let transform_origin = "50% 50%";
        let element_width = 100.0;
        let element_height = 100.0;
        let element_x = 50.0;
        let element_y = 50.0;

        let local_x = 30.0;
        let local_y = 40.0;

        // Forward: element-local to screen
        let (screen_x, screen_y) = element_to_screen_coords(
            local_x, local_y,
            element_transform, transform_origin,
            element_width, element_height,
            element_x, element_y,
        );

        // Inverse: screen to element-local
        let (back_x, back_y) = screen_to_element_coords(
            screen_x, screen_y,
            element_transform, transform_origin,
            element_width, element_height,
            element_x, element_y,
        );

        assert!((back_x - local_x).abs() < 0.01, "back_x = {}, expected {}", back_x, local_x);
        assert!((back_y - local_y).abs() < 0.01, "back_y = {}, expected {}", back_y, local_y);
    }
}
