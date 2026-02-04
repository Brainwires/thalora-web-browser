//! 4x4 Matrix operations for CSS 3D transforms
//!
//! CSS transforms use 4x4 homogeneous matrices stored in column-major order.
//! The matrix3d() function expects values in this order:
//! ```text
//! matrix3d(m11, m21, m31, m41,
//!          m12, m22, m32, m42,
//!          m13, m23, m33, m43,
//!          m14, m24, m34, m44)
//! ```
//!
//! Which represents the matrix:
//! ```text
//! | m11 m12 m13 m14 |   | a1 a2 a3 a4 |
//! | m21 m22 m23 m24 | = | b1 b2 b3 b4 |
//! | m31 m32 m33 m34 |   | c1 c2 c3 c4 |
//! | m41 m42 m43 m44 |   | d1 d2 d3 d4 |
//! ```

use std::f64::consts::PI;

/// A 4x4 transformation matrix stored in column-major order.
///
/// Internally stored as `[f64; 16]` where:
/// - `data[0..4]` is column 0 (m11, m21, m31, m41)
/// - `data[4..8]` is column 1 (m12, m22, m32, m42)
/// - `data[8..12]` is column 2 (m13, m23, m33, m43)
/// - `data[12..16]` is column 3 (m14, m24, m34, m44)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    /// Column-major storage: data[col * 4 + row]
    data: [f64; 16],
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Matrix4 {
    /// Create a new matrix from column-major data.
    pub fn new(data: [f64; 16]) -> Self {
        Self { data }
    }

    /// Create an identity matrix.
    pub fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,  // column 0
                0.0, 1.0, 0.0, 0.0,  // column 1
                0.0, 0.0, 1.0, 0.0,  // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Get element at (row, col) - 0-indexed.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[col * 4 + row]
    }

    /// Set element at (row, col) - 0-indexed.
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[col * 4 + row] = value;
    }

    /// Create from CSS matrix3d() values (16 values in column-major order as CSS specifies).
    ///
    /// CSS matrix3d(a1,b1,c1,d1, a2,b2,c2,d2, a3,b3,c3,d3, a4,b4,c4,d4) represents:
    /// ```text
    /// | a1 a2 a3 a4 |
    /// | b1 b2 b3 b4 |
    /// | c1 c2 c3 c4 |
    /// | d1 d2 d3 d4 |
    /// ```
    pub fn from_matrix3d(values: [f64; 16]) -> Self {
        // CSS matrix3d is already in column-major order as we want it
        Self { data: values }
    }

    /// Create from CSS matrix() values (6 values for 2D affine transform).
    ///
    /// CSS matrix(a, b, c, d, e, f) represents:
    /// ```text
    /// | a c e |     | a c 0 e |
    /// | b d f | --> | b d 0 f |
    /// | 0 0 1 |     | 0 0 1 0 |
    ///               | 0 0 0 1 |
    /// ```
    pub fn from_matrix2d(values: [f64; 6]) -> Self {
        let [a, b, c, d, e, f] = values;
        Self {
            data: [
                a, b, 0.0, 0.0,     // column 0
                c, d, 0.0, 0.0,     // column 1
                0.0, 0.0, 1.0, 0.0, // column 2
                e, f, 0.0, 1.0,     // column 3
            ],
        }
    }

    /// Create a translation matrix.
    pub fn translate(tx: f64, ty: f64, tz: f64) -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,  // column 0
                0.0, 1.0, 0.0, 0.0,  // column 1
                0.0, 0.0, 1.0, 0.0,  // column 2
                tx, ty, tz, 1.0,     // column 3
            ],
        }
    }

    /// Create a 2D translation matrix.
    pub fn translate_2d(tx: f64, ty: f64) -> Self {
        Self::translate(tx, ty, 0.0)
    }

    /// Create a uniform scale matrix.
    pub fn scale_uniform(s: f64) -> Self {
        Self::scale(s, s, s)
    }

    /// Create a scale matrix with different factors for each axis.
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            data: [
                sx, 0.0, 0.0, 0.0,  // column 0
                0.0, sy, 0.0, 0.0,  // column 1
                0.0, 0.0, sz, 0.0,  // column 2
                0.0, 0.0, 0.0, 1.0, // column 3
            ],
        }
    }

    /// Create a 2D scale matrix.
    pub fn scale_2d(sx: f64, sy: f64) -> Self {
        Self::scale(sx, sy, 1.0)
    }

    /// Create a rotation matrix around the X axis.
    pub fn rotate_x(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,  // column 0
                0.0, c, s, 0.0,      // column 1
                0.0, -s, c, 0.0,     // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Create a rotation matrix around the Y axis.
    pub fn rotate_y(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            data: [
                c, 0.0, -s, 0.0,     // column 0
                0.0, 1.0, 0.0, 0.0,  // column 1
                s, 0.0, c, 0.0,      // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Create a rotation matrix around the Z axis (2D rotation).
    pub fn rotate_z(angle_rad: f64) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self {
            data: [
                c, s, 0.0, 0.0,      // column 0
                -s, c, 0.0, 0.0,     // column 1
                0.0, 0.0, 1.0, 0.0,  // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Create a rotation matrix around an arbitrary axis using Rodrigues' rotation formula.
    ///
    /// The axis (x, y, z) should be normalized.
    pub fn rotate_3d(x: f64, y: f64, z: f64, angle_rad: f64) -> Self {
        // Normalize the axis
        let len = (x * x + y * y + z * z).sqrt();
        if len < 1e-10 {
            return Self::identity();
        }
        let x = x / len;
        let y = y / len;
        let z = z / len;

        let c = angle_rad.cos();
        let s = angle_rad.sin();
        let t = 1.0 - c;

        Self {
            data: [
                // column 0
                t * x * x + c,
                t * x * y + s * z,
                t * x * z - s * y,
                0.0,
                // column 1
                t * x * y - s * z,
                t * y * y + c,
                t * y * z + s * x,
                0.0,
                // column 2
                t * x * z + s * y,
                t * y * z - s * x,
                t * z * z + c,
                0.0,
                // column 3
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    /// Create a skew matrix along the X axis.
    pub fn skew_x(angle_rad: f64) -> Self {
        let t = angle_rad.tan();
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,  // column 0
                t, 1.0, 0.0, 0.0,    // column 1
                0.0, 0.0, 1.0, 0.0,  // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Create a skew matrix along the Y axis.
    pub fn skew_y(angle_rad: f64) -> Self {
        let t = angle_rad.tan();
        Self {
            data: [
                1.0, t, 0.0, 0.0,    // column 0
                0.0, 1.0, 0.0, 0.0,  // column 1
                0.0, 0.0, 1.0, 0.0,  // column 2
                0.0, 0.0, 0.0, 1.0,  // column 3
            ],
        }
    }

    /// Create a perspective matrix.
    ///
    /// CSS perspective(d) creates a perspective projection where d is the distance
    /// from the viewer to the z=0 plane.
    pub fn perspective(d: f64) -> Self {
        if d.abs() < 1e-10 {
            return Self::identity();
        }
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0,        // column 0
                0.0, 1.0, 0.0, 0.0,        // column 1
                0.0, 0.0, 1.0, -1.0 / d,   // column 2
                0.0, 0.0, 0.0, 1.0,        // column 3
            ],
        }
    }

    /// Multiply two matrices: self * other.
    ///
    /// CSS transforms apply left-to-right, so for `transform: A B C`:
    /// - Point is transformed by: A * B * C * point
    pub fn multiply(&self, other: &Matrix4) -> Matrix4 {
        let mut result = [0.0; 16];

        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.get(row, k) * other.get(k, col);
                }
                result[col * 4 + row] = sum;
            }
        }

        Matrix4 { data: result }
    }

    /// Compute the inverse of this matrix using Gaussian elimination with partial pivoting.
    ///
    /// Returns `None` if the matrix is singular (not invertible).
    pub fn inverse(&self) -> Option<Matrix4> {
        // Create augmented matrix [A | I]
        let mut aug = [[0.0; 8]; 4];
        for row in 0..4 {
            for col in 0..4 {
                aug[row][col] = self.get(row, col);
                aug[row][col + 4] = if row == col { 1.0 } else { 0.0 };
            }
        }

        // Forward elimination with partial pivoting
        for col in 0..4 {
            // Find pivot
            let mut max_row = col;
            let mut max_val = aug[col][col].abs();
            for row in (col + 1)..4 {
                let val = aug[row][col].abs();
                if val > max_val {
                    max_val = val;
                    max_row = row;
                }
            }

            // Check if matrix is singular
            if max_val < 1e-14 {
                return None;
            }

            // Swap rows
            if max_row != col {
                aug.swap(col, max_row);
            }

            // Eliminate column
            let pivot = aug[col][col];
            for row in 0..4 {
                if row != col {
                    let factor = aug[row][col] / pivot;
                    for k in col..8 {
                        aug[row][k] -= factor * aug[col][k];
                    }
                }
            }

            // Scale pivot row
            for k in col..8 {
                aug[col][k] /= pivot;
            }
        }

        // Extract inverse from augmented matrix
        let mut inv = [0.0; 16];
        for row in 0..4 {
            for col in 0..4 {
                inv[col * 4 + row] = aug[row][col + 4];
            }
        }

        Some(Matrix4 { data: inv })
    }

    /// Transform a 3D point by this matrix.
    ///
    /// The point is treated as homogeneous coordinates (x, y, z, 1) and the result
    /// is normalized by the w component.
    pub fn transform_point(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let w = self.get(3, 0) * x + self.get(3, 1) * y + self.get(3, 2) * z + self.get(3, 3);

        if w.abs() < 1e-14 {
            // Point at infinity
            return (f64::INFINITY, f64::INFINITY, f64::INFINITY);
        }

        let out_x = (self.get(0, 0) * x + self.get(0, 1) * y + self.get(0, 2) * z + self.get(0, 3)) / w;
        let out_y = (self.get(1, 0) * x + self.get(1, 1) * y + self.get(1, 2) * z + self.get(1, 3)) / w;
        let out_z = (self.get(2, 0) * x + self.get(2, 1) * y + self.get(2, 2) * z + self.get(2, 3)) / w;

        (out_x, out_y, out_z)
    }

    /// Transform a 2D point (z=0) and project back to 2D.
    pub fn transform_point_2d(&self, x: f64, y: f64) -> (f64, f64) {
        let (out_x, out_y, _) = self.transform_point(x, y, 0.0);
        (out_x, out_y)
    }

    /// Compute the determinant of this matrix.
    pub fn determinant(&self) -> f64 {
        let m = |r: usize, c: usize| self.get(r, c);

        // Expand along the first row
        let det = m(0, 0) * self.minor_3x3(0, 0)
                - m(0, 1) * self.minor_3x3(0, 1)
                + m(0, 2) * self.minor_3x3(0, 2)
                - m(0, 3) * self.minor_3x3(0, 3);

        det
    }

    /// Compute the determinant of the 3x3 minor matrix excluding row `skip_row` and column `skip_col`.
    fn minor_3x3(&self, skip_row: usize, skip_col: usize) -> f64 {
        let mut minor = [[0.0; 3]; 3];
        let mut mi = 0;
        for ri in 0..4 {
            if ri == skip_row {
                continue;
            }
            let mut mj = 0;
            for ci in 0..4 {
                if ci == skip_col {
                    continue;
                }
                minor[mi][mj] = self.get(ri, ci);
                mj += 1;
            }
            mi += 1;
        }

        // 3x3 determinant
        minor[0][0] * (minor[1][1] * minor[2][2] - minor[1][2] * minor[2][1])
            - minor[0][1] * (minor[1][0] * minor[2][2] - minor[1][2] * minor[2][0])
            + minor[0][2] * (minor[1][0] * minor[2][1] - minor[1][1] * minor[2][0])
    }

    /// Check if this matrix is approximately equal to another.
    pub fn approx_eq(&self, other: &Matrix4, epsilon: f64) -> bool {
        for i in 0..16 {
            if (self.data[i] - other.data[i]).abs() > epsilon {
                return false;
            }
        }
        true
    }

    /// Get the raw data as a slice.
    pub fn as_slice(&self) -> &[f64; 16] {
        &self.data
    }
}

/// Convert degrees to radians.
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// Convert radians to degrees.
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

/// Convert gradians to radians.
pub fn grad_to_rad(grad: f64) -> f64 {
    grad * PI / 200.0
}

/// Convert turns to radians.
pub fn turn_to_rad(turn: f64) -> f64 {
    turn * 2.0 * PI
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_identity() {
        let m = Matrix4::identity();
        assert!((m.get(0, 0) - 1.0).abs() < EPSILON);
        assert!((m.get(1, 1) - 1.0).abs() < EPSILON);
        assert!((m.get(2, 2) - 1.0).abs() < EPSILON);
        assert!((m.get(3, 3) - 1.0).abs() < EPSILON);
        assert!((m.get(0, 1)).abs() < EPSILON);
        assert!((m.get(1, 0)).abs() < EPSILON);
    }

    #[test]
    fn test_translate() {
        let m = Matrix4::translate(10.0, 20.0, 30.0);
        let (x, y, z) = m.transform_point(0.0, 0.0, 0.0);
        assert!((x - 10.0).abs() < EPSILON);
        assert!((y - 20.0).abs() < EPSILON);
        assert!((z - 30.0).abs() < EPSILON);
    }

    #[test]
    fn test_scale() {
        let m = Matrix4::scale(2.0, 3.0, 4.0);
        let (x, y, z) = m.transform_point(1.0, 1.0, 1.0);
        assert!((x - 2.0).abs() < EPSILON);
        assert!((y - 3.0).abs() < EPSILON);
        assert!((z - 4.0).abs() < EPSILON);
    }

    #[test]
    fn test_rotate_z_90() {
        let m = Matrix4::rotate_z(deg_to_rad(90.0));
        let (x, y, _) = m.transform_point(1.0, 0.0, 0.0);
        // (1, 0) rotated 90° CCW should become (0, 1)
        assert!((x - 0.0).abs() < EPSILON, "x = {}", x);
        assert!((y - 1.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_rotate_z_180() {
        let m = Matrix4::rotate_z(deg_to_rad(180.0));
        let (x, y, _) = m.transform_point(1.0, 0.0, 0.0);
        // (1, 0) rotated 180° should become (-1, 0)
        assert!((x - (-1.0)).abs() < EPSILON, "x = {}", x);
        assert!((y - 0.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_multiply_identity() {
        let a = Matrix4::translate(5.0, 10.0, 15.0);
        let i = Matrix4::identity();
        let result = a.multiply(&i);
        assert!(result.approx_eq(&a, EPSILON));
    }

    #[test]
    fn test_multiply_translate() {
        let a = Matrix4::translate(5.0, 0.0, 0.0);
        let b = Matrix4::translate(0.0, 10.0, 0.0);
        let result = a.multiply(&b);
        let (x, y, z) = result.transform_point(0.0, 0.0, 0.0);
        assert!((x - 5.0).abs() < EPSILON);
        assert!((y - 10.0).abs() < EPSILON);
        assert!((z - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_inverse_identity() {
        let m = Matrix4::identity();
        let inv = m.inverse().expect("Identity should be invertible");
        assert!(inv.approx_eq(&m, EPSILON));
    }

    #[test]
    fn test_inverse_translate() {
        let m = Matrix4::translate(10.0, 20.0, 30.0);
        let inv = m.inverse().expect("Translation should be invertible");
        let result = m.multiply(&inv);
        assert!(result.approx_eq(&Matrix4::identity(), EPSILON));
    }

    #[test]
    fn test_inverse_rotate() {
        let m = Matrix4::rotate_z(deg_to_rad(45.0));
        let inv = m.inverse().expect("Rotation should be invertible");
        let result = m.multiply(&inv);
        assert!(result.approx_eq(&Matrix4::identity(), EPSILON));
    }

    #[test]
    fn test_inverse_scale() {
        let m = Matrix4::scale(2.0, 3.0, 4.0);
        let inv = m.inverse().expect("Scale should be invertible");
        let result = m.multiply(&inv);
        assert!(result.approx_eq(&Matrix4::identity(), EPSILON));
    }

    #[test]
    fn test_inverse_complex() {
        // Combine translate, rotate, scale
        let t = Matrix4::translate(10.0, 20.0, 0.0);
        let r = Matrix4::rotate_z(deg_to_rad(30.0));
        let s = Matrix4::scale(1.5, 2.0, 1.0);
        let m = t.multiply(&r).multiply(&s);

        let inv = m.inverse().expect("Combined transform should be invertible");
        let result = m.multiply(&inv);
        assert!(result.approx_eq(&Matrix4::identity(), EPSILON));
    }

    #[test]
    fn test_from_matrix2d() {
        // CSS matrix(a, b, c, d, e, f) = | a c e |
        //                                | b d f |
        //                                | 0 0 1 |
        let m = Matrix4::from_matrix2d([2.0, 0.0, 0.0, 2.0, 10.0, 20.0]); // scale(2) translate(10, 20)
        let (x, y, _) = m.transform_point(0.0, 0.0, 0.0);
        assert!((x - 10.0).abs() < EPSILON);
        assert!((y - 20.0).abs() < EPSILON);
    }

    #[test]
    fn test_perspective() {
        let m = Matrix4::perspective(1000.0);
        // Points on z=0 plane should not be affected
        let (x, y, z) = m.transform_point(100.0, 50.0, 0.0);
        assert!((x - 100.0).abs() < EPSILON);
        assert!((y - 50.0).abs() < EPSILON);
        assert!((z - 0.0).abs() < EPSILON);

        // Point at z=500 (halfway to perspective origin) should be scaled up
        let (x, y, _) = m.transform_point(100.0, 50.0, 500.0);
        // w = 1 - 500/1000 = 0.5, so coordinates are doubled
        assert!((x - 200.0).abs() < EPSILON, "x = {}", x);
        assert!((y - 100.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_skew_x() {
        let m = Matrix4::skew_x(deg_to_rad(45.0));
        let (x, y, _) = m.transform_point(0.0, 1.0, 0.0);
        // Point (0, 1) skewed 45° in X should become (1, 1)
        assert!((x - 1.0).abs() < EPSILON, "x = {}", x);
        assert!((y - 1.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_skew_y() {
        let m = Matrix4::skew_y(deg_to_rad(45.0));
        let (x, y, _) = m.transform_point(1.0, 0.0, 0.0);
        // Point (1, 0) skewed 45° in Y should become (1, 1)
        assert!((x - 1.0).abs() < EPSILON, "x = {}", x);
        assert!((y - 1.0).abs() < EPSILON, "y = {}", y);
    }

    #[test]
    fn test_rotate_3d() {
        // Rotate 90° around Z axis using rotate3d
        let m = Matrix4::rotate_3d(0.0, 0.0, 1.0, deg_to_rad(90.0));
        let m_z = Matrix4::rotate_z(deg_to_rad(90.0));
        assert!(m.approx_eq(&m_z, EPSILON));
    }

    #[test]
    fn test_determinant() {
        let m = Matrix4::identity();
        assert!((m.determinant() - 1.0).abs() < EPSILON);

        let m = Matrix4::scale(2.0, 3.0, 4.0);
        assert!((m.determinant() - 24.0).abs() < EPSILON);
    }

    #[test]
    fn test_singular_matrix() {
        // Create a singular matrix (all zeros)
        let m = Matrix4::new([0.0; 16]);
        assert!(m.inverse().is_none());
    }
}
