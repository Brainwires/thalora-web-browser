//! Mouse movement simulation for realistic human-like interactions
//!
//! Generates Bezier curve paths with human-like characteristics including:
//! - Random control points with slight deviation
//! - Varying speed (faster in middle of movement)
//! - Micro-jitter for hand tremor simulation

use rand::Rng;
use std::time::Duration;

/// A single point in a mouse movement path
#[derive(Debug, Clone, Copy)]
pub struct MousePoint {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Delay before this point (from previous point)
    pub delay: Duration,
    /// Movement delta X (for movementX property)
    pub movement_x: f64,
    /// Movement delta Y (for movementY property)
    pub movement_y: f64,
}

/// A complete mouse movement path from start to end
#[derive(Debug, Clone)]
pub struct MousePath {
    /// All points in the path
    pub points: Vec<MousePoint>,
    /// Total duration of the movement
    pub total_duration: Duration,
}

/// Configuration for mouse path generation
#[derive(Debug, Clone)]
pub struct MousePathConfig {
    /// Number of points to generate along the path
    pub num_points: usize,
    /// Amount of random deviation for control points (0.0 - 1.0)
    pub deviation: f64,
    /// Amount of micro-jitter to add (pixels)
    pub jitter: f64,
    /// Minimum delay between points (ms)
    pub min_delay_ms: u64,
    /// Maximum delay between points (ms)
    pub max_delay_ms: u64,
    /// Whether to use easing (slower at start and end)
    pub use_easing: bool,
}

impl Default for MousePathConfig {
    fn default() -> Self {
        Self {
            num_points: 20,
            deviation: 0.2,
            jitter: 0.5,
            min_delay_ms: 5,
            max_delay_ms: 25,
            use_easing: true,
        }
    }
}

impl MousePath {
    /// Generate a human-like mouse movement path using Bezier curves
    pub fn generate(start: (f64, f64), end: (f64, f64), duration_ms: u64) -> Self {
        Self::generate_with_config(start, end, duration_ms, MousePathConfig::default())
    }

    /// Generate a mouse path with custom configuration
    pub fn generate_with_config(
        start: (f64, f64),
        end: (f64, f64),
        duration_ms: u64,
        config: MousePathConfig,
    ) -> Self {
        let mut rng = rand::thread_rng();

        // Calculate distance for scaling
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let distance = (dx * dx + dy * dy).sqrt();

        // Generate control points with random deviation
        // For a cubic Bezier, we need 2 control points between start and end
        let deviation_scale = distance * config.deviation;

        // Control point 1: about 1/3 of the way, with perpendicular deviation
        let perpendicular_angle = (dy.atan2(dx)) + std::f64::consts::FRAC_PI_2;
        let dev1 = (rng.r#gen::<f64>() - 0.5) * 2.0 * deviation_scale;
        let cp1 = (
            start.0 + dx * 0.33 + dev1 * perpendicular_angle.cos(),
            start.1 + dy * 0.33 + dev1 * perpendicular_angle.sin(),
        );

        // Control point 2: about 2/3 of the way, with perpendicular deviation
        let dev2 = (rng.r#gen::<f64>() - 0.5) * 2.0 * deviation_scale;
        let cp2 = (
            start.0 + dx * 0.67 + dev2 * perpendicular_angle.cos(),
            start.1 + dy * 0.67 + dev2 * perpendicular_angle.sin(),
        );

        let mut points = Vec::with_capacity(config.num_points);
        let mut prev_x = start.0;
        let mut prev_y = start.1;
        let mut total_time = 0u64;

        for i in 0..config.num_points {
            // Calculate t value (0 to 1)
            let t = i as f64 / (config.num_points - 1) as f64;

            // Apply easing function for more realistic speed variation
            let eased_t = if config.use_easing {
                Self::ease_in_out_sine(t)
            } else {
                t
            };

            // Calculate point on cubic Bezier curve
            let (mut x, mut y) = Self::cubic_bezier(start, cp1, cp2, end, eased_t);

            // Add micro-jitter for hand tremor (except at start and end)
            if i > 0 && i < config.num_points - 1 && config.jitter > 0.0 {
                x += (rng.r#gen::<f64>() - 0.5) * 2.0 * config.jitter;
                y += (rng.r#gen::<f64>() - 0.5) * 2.0 * config.jitter;
            }

            // Calculate delay with variation
            // Movement is faster in the middle (when t is near 0.5)
            let speed_factor = if config.use_easing {
                // Speed curve: slow at edges, fast in middle
                1.0 - (2.0 * (eased_t - 0.5)).abs() * 0.5
            } else {
                1.0
            };

            let base_delay = (duration_ms as f64 / config.num_points as f64) * speed_factor;
            let delay_jitter = (rng.r#gen::<f64>() - 0.5) * base_delay * 0.3;
            let delay = (base_delay + delay_jitter)
                .max(config.min_delay_ms as f64)
                .min(config.max_delay_ms as f64);

            let delay_ms = if i == 0 { 0 } else { delay as u64 };
            total_time += delay_ms;

            points.push(MousePoint {
                x,
                y,
                delay: Duration::from_millis(delay_ms),
                movement_x: x - prev_x,
                movement_y: y - prev_y,
            });

            prev_x = x;
            prev_y = y;
        }

        Self {
            points,
            total_duration: Duration::from_millis(total_time),
        }
    }

    /// Cubic Bezier interpolation
    fn cubic_bezier(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        t: f64,
    ) -> (f64, f64) {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        (
            mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0,
            mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1,
        )
    }

    /// Ease-in-out sine function for natural acceleration/deceleration
    fn ease_in_out_sine(t: f64) -> f64 {
        -(((t * std::f64::consts::PI).cos() - 1.0) / 2.0)
    }

    /// Get the final position of the path
    pub fn end_position(&self) -> Option<(f64, f64)> {
        self.points.last().map(|p| (p.x, p.y))
    }

    /// Get the starting position of the path
    pub fn start_position(&self) -> Option<(f64, f64)> {
        self.points.first().map(|p| (p.x, p.y))
    }
}

/// Generates a realistic mouse click sequence with movement
#[derive(Debug, Clone)]
pub struct ClickSequence {
    /// Mouse movement path leading to the click
    pub movement_path: MousePath,
    /// Delay between mousedown and mouseup (in ms)
    pub click_duration_ms: u64,
    /// Whether this is a double-click
    pub is_double_click: bool,
    /// Delay between clicks for double-click (in ms)
    pub double_click_delay_ms: u64,
}

impl ClickSequence {
    /// Generate a realistic click sequence
    pub fn generate(current_pos: (f64, f64), target_pos: (f64, f64)) -> Self {
        let mut rng = rand::thread_rng();

        // Calculate movement duration based on distance
        let dx = target_pos.0 - current_pos.0;
        let dy = target_pos.1 - current_pos.1;
        let distance = (dx * dx + dy * dy).sqrt();

        // Fitts' Law approximation: movement time scales with log of distance/target_width
        // Assume target width of ~20px for typical buttons
        let base_duration = 200.0 + (distance / 20.0).ln() * 100.0;
        let duration_ms = (base_duration + rng.r#gen::<f64>() * 50.0) as u64;

        // Generate the movement path
        let movement_path = MousePath::generate(current_pos, target_pos, duration_ms);

        // Random click duration (typical human click is 50-150ms)
        let click_duration_ms = rng.gen_range(50..150);

        Self {
            movement_path,
            click_duration_ms,
            is_double_click: false,
            double_click_delay_ms: 0,
        }
    }

    /// Generate a double-click sequence
    pub fn generate_double_click(current_pos: (f64, f64), target_pos: (f64, f64)) -> Self {
        let mut sequence = Self::generate(current_pos, target_pos);
        sequence.is_double_click = true;

        // Double-click delay is typically 100-300ms between clicks
        let mut rng = rand::thread_rng();
        sequence.double_click_delay_ms = rng.gen_range(100..300);

        sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_path_generation() {
        let path = MousePath::generate((0.0, 0.0), (100.0, 100.0), 300);

        // Should have default number of points
        assert_eq!(path.points.len(), 20);

        // First point should be at start
        let first = path.points.first().unwrap();
        assert!((first.x - 0.0).abs() < 1.0);
        assert!((first.y - 0.0).abs() < 1.0);

        // Last point should be near end (with jitter)
        let last = path.points.last().unwrap();
        assert!((last.x - 100.0).abs() < 5.0);
        assert!((last.y - 100.0).abs() < 5.0);

        // Total duration should be reasonable
        assert!(path.total_duration.as_millis() > 100);
        assert!(path.total_duration.as_millis() < 1000);
    }

    #[test]
    fn test_click_sequence_generation() {
        let sequence = ClickSequence::generate((50.0, 50.0), (200.0, 150.0));

        assert!(!sequence.movement_path.points.is_empty());
        assert!(sequence.click_duration_ms >= 50);
        assert!(sequence.click_duration_ms <= 150);
        assert!(!sequence.is_double_click);
    }

    #[test]
    fn test_bezier_curve_endpoints() {
        // Bezier at t=0 should be start point
        let start = (0.0, 0.0);
        let end = (100.0, 100.0);
        let cp1 = (30.0, 50.0);
        let cp2 = (70.0, 50.0);

        let (x0, y0) = MousePath::cubic_bezier(start, cp1, cp2, end, 0.0);
        assert!((x0 - 0.0).abs() < 0.0001);
        assert!((y0 - 0.0).abs() < 0.0001);

        let (x1, y1) = MousePath::cubic_bezier(start, cp1, cp2, end, 1.0);
        assert!((x1 - 100.0).abs() < 0.0001);
        assert!((y1 - 100.0).abs() < 0.0001);
    }
}
