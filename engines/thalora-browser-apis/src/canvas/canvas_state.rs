//! Canvas state management
//!
//! Manages the drawing state including transforms, styles, and clipping regions.

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform,
};

/// Fill or stroke style
#[derive(Debug, Clone)]
pub enum CanvasStyle {
    /// Solid color
    Color(Color),
    /// Linear gradient (not fully implemented yet)
    LinearGradient {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        stops: Vec<(f32, Color)>,
    },
    /// Radial gradient (not fully implemented yet)
    RadialGradient {
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
        stops: Vec<(f32, Color)>,
    },
    /// Pattern (not fully implemented yet)
    Pattern,
}

impl Default for CanvasStyle {
    fn default() -> Self {
        CanvasStyle::Color(Color::BLACK)
    }
}

impl CanvasStyle {
    /// Parse a CSS color string into a CanvasStyle
    pub fn from_css_color(color_str: &str) -> Option<Self> {
        let color_str = color_str.trim().to_lowercase();

        // Handle named colors
        let color = match color_str.as_str() {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::from_rgba8(255, 0, 0, 255),
            "green" => Color::from_rgba8(0, 128, 0, 255),
            "blue" => Color::from_rgba8(0, 0, 255, 255),
            "yellow" => Color::from_rgba8(255, 255, 0, 255),
            "cyan" | "aqua" => Color::from_rgba8(0, 255, 255, 255),
            "magenta" | "fuchsia" => Color::from_rgba8(255, 0, 255, 255),
            "gray" | "grey" => Color::from_rgba8(128, 128, 128, 255),
            "silver" => Color::from_rgba8(192, 192, 192, 255),
            "maroon" => Color::from_rgba8(128, 0, 0, 255),
            "olive" => Color::from_rgba8(128, 128, 0, 255),
            "purple" => Color::from_rgba8(128, 0, 128, 255),
            "teal" => Color::from_rgba8(0, 128, 128, 255),
            "navy" => Color::from_rgba8(0, 0, 128, 255),
            "orange" => Color::from_rgba8(255, 165, 0, 255),
            "pink" => Color::from_rgba8(255, 192, 203, 255),
            "transparent" => Color::from_rgba8(0, 0, 0, 0),
            _ => {
                // Try to parse hex colors
                if color_str.starts_with('#') {
                    Self::parse_hex_color(&color_str)?
                } else if color_str.starts_with("rgb") {
                    Self::parse_rgb_color(&color_str)?
                } else if color_str.starts_with("rgba") {
                    Self::parse_rgba_color(&color_str)?
                } else {
                    return None;
                }
            }
        };

        Some(CanvasStyle::Color(color))
    }

    fn parse_hex_color(s: &str) -> Option<Color> {
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
                Some(Color::from_rgba8(r, g, b, 255))
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(Color::from_rgba8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                let a = u8::from_str_radix(&s[6..8], 16).ok()?;
                Some(Color::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    fn parse_rgb_color(s: &str) -> Option<Color> {
        let s = s.trim_start_matches("rgb(").trim_end_matches(')');
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        let r: u8 = parts[0].trim().parse().ok()?;
        let g: u8 = parts[1].trim().parse().ok()?;
        let b: u8 = parts[2].trim().parse().ok()?;
        Some(Color::from_rgba8(r, g, b, 255))
    }

    fn parse_rgba_color(s: &str) -> Option<Color> {
        let s = s.trim_start_matches("rgba(").trim_end_matches(')');
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return None;
        }
        let r: u8 = parts[0].trim().parse().ok()?;
        let g: u8 = parts[1].trim().parse().ok()?;
        let b: u8 = parts[2].trim().parse().ok()?;
        let a: f32 = parts[3].trim().parse().ok()?;
        Some(Color::from_rgba8(r, g, b, (a * 255.0) as u8))
    }

    /// Convert to a tiny_skia Paint
    pub fn to_paint(&self) -> Paint<'static> {
        match self {
            CanvasStyle::Color(color) => {
                let mut paint = Paint::default();
                paint.set_color(*color);
                paint
            }
            // Gradients and patterns would need more complex handling
            _ => {
                let mut paint = Paint::default();
                paint.set_color(Color::BLACK);
                paint
            }
        }
    }
}

/// A single saved state
#[derive(Debug, Clone)]
pub struct DrawingState {
    /// Current transformation matrix
    pub transform: Transform,
    /// Fill style
    pub fill_style: CanvasStyle,
    /// Stroke style
    pub stroke_style: CanvasStyle,
    /// Line width
    pub line_width: f32,
    /// Line cap style
    pub line_cap: LineCap,
    /// Line join style
    pub line_join: LineJoin,
    /// Miter limit
    pub miter_limit: f32,
    /// Line dash pattern
    pub line_dash: Vec<f32>,
    /// Line dash offset
    pub line_dash_offset: f32,
    /// Global alpha
    pub global_alpha: f32,
    /// Shadow blur
    pub shadow_blur: f32,
    /// Shadow color
    pub shadow_color: Color,
    /// Shadow offset X
    pub shadow_offset_x: f32,
    /// Shadow offset Y
    pub shadow_offset_y: f32,
    /// Font string (e.g., "10px sans-serif")
    pub font: String,
    /// Text alignment
    pub text_align: String,
    /// Text baseline
    pub text_baseline: String,
    /// Clipping path (optional)
    pub clip_path: Option<Path>,
}

impl Default for DrawingState {
    fn default() -> Self {
        Self {
            transform: Transform::identity(),
            fill_style: CanvasStyle::Color(Color::BLACK),
            stroke_style: CanvasStyle::Color(Color::BLACK),
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            global_alpha: 1.0,
            shadow_blur: 0.0,
            shadow_color: Color::from_rgba8(0, 0, 0, 0),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            font: "10px sans-serif".to_string(),
            text_align: "start".to_string(),
            text_baseline: "alphabetic".to_string(),
            clip_path: None,
        }
    }
}

/// Canvas rendering state manager
#[derive(Debug)]
pub struct CanvasState {
    /// The pixel buffer
    pub pixmap: Pixmap,
    /// Current drawing state
    pub current: DrawingState,
    /// Stack of saved states
    pub state_stack: Vec<DrawingState>,
    /// Current path being built
    pub current_path: PathBuilder,
}

impl CanvasState {
    /// Create a new CanvasState with the given dimensions
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let pixmap = Pixmap::new(width, height)?;
        Some(Self {
            pixmap,
            current: DrawingState::default(),
            state_stack: Vec::new(),
            current_path: PathBuilder::new(),
        })
    }

    /// Save the current state
    pub fn save(&mut self) {
        self.state_stack.push(self.current.clone());
    }

    /// Restore the previous state
    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current = state;
        }
    }

    /// Get the current stroke settings
    pub fn get_stroke(&self) -> Stroke {
        let mut stroke = Stroke::default();
        stroke.width = self.current.line_width;
        stroke.line_cap = self.current.line_cap;
        stroke.line_join = self.current.line_join;
        stroke.miter_limit = self.current.miter_limit;
        if !self.current.line_dash.is_empty() {
            stroke.dash = tiny_skia::StrokeDash::new(
                self.current.line_dash.clone(),
                self.current.line_dash_offset,
            );
        }
        stroke
    }

    /// Reset the current path
    pub fn begin_path(&mut self) {
        self.current_path = PathBuilder::new();
    }

    /// Move to a point
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_path.move_to(x, y);
    }

    /// Draw a line to a point
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.current_path.line_to(x, y);
    }

    /// Draw a quadratic curve
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.current_path.quad_to(cpx, cpy, x, y);
    }

    /// Draw a bezier curve
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.current_path.cubic_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    /// Add a rectangle to the path
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.current_path.move_to(x, y);
        self.current_path.line_to(x + width, y);
        self.current_path.line_to(x + width, y + height);
        self.current_path.line_to(x, y + height);
        self.current_path.close();
    }

    /// Close the current path
    pub fn close_path(&mut self) {
        self.current_path.close();
    }

    /// Draw an arc
    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) {
        // Convert arc to bezier curves
        // This is a simplified implementation
        let mut angle = start_angle;
        let end = if counterclockwise {
            if end_angle > start_angle {
                end_angle - std::f32::consts::TAU
            } else {
                end_angle
            }
        } else if end_angle < start_angle {
            end_angle + std::f32::consts::TAU
        } else {
            end_angle
        };

        let steps = 32;
        let step = (end - angle) / steps as f32;

        let first_x = x + radius * angle.cos();
        let first_y = y + radius * angle.sin();
        self.current_path.move_to(first_x, first_y);

        for _ in 0..steps {
            angle += step;
            let px = x + radius * angle.cos();
            let py = y + radius * angle.sin();
            self.current_path.line_to(px, py);
        }
    }

    /// Fill the current path
    pub fn fill(&mut self) {
        if let Some(path) = self.current_path.clone().finish() {
            let mut paint = self.current.fill_style.to_paint();
            paint.anti_alias = true;

            // Apply global alpha
            if self.current.global_alpha < 1.0 {
                // For solid colors, we can adjust alpha
                if let CanvasStyle::Color(ref c) = self.current.fill_style {
                    let new_alpha = (c.alpha() * self.current.global_alpha).min(1.0);
                    paint.set_color(
                        Color::from_rgba(c.red(), c.green(), c.blue(), new_alpha)
                            .unwrap_or(Color::BLACK),
                    );
                }
            }

            self.pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                self.current.transform,
                None,
            );
        }
    }

    /// Stroke the current path
    pub fn stroke(&mut self) {
        if let Some(path) = self.current_path.clone().finish() {
            let mut paint = self.current.stroke_style.to_paint();
            paint.anti_alias = true;

            // Apply global alpha
            if self.current.global_alpha < 1.0 {
                if let CanvasStyle::Color(ref c) = self.current.stroke_style {
                    let new_alpha = (c.alpha() * self.current.global_alpha).min(1.0);
                    paint.set_color(
                        Color::from_rgba(c.red(), c.green(), c.blue(), new_alpha)
                            .unwrap_or(Color::BLACK),
                    );
                }
            }

            let stroke = self.get_stroke();

            self.pixmap
                .stroke_path(&path, &paint, &stroke, self.current.transform, None);
        }
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = tiny_skia::Rect::from_xywh(x, y, width, height);
        if let Some(rect) = rect {
            let mut paint = self.current.fill_style.to_paint();
            paint.anti_alias = true;

            self.pixmap
                .fill_rect(rect, &paint, self.current.transform, None);
        }
    }

    /// Stroke a rectangle
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + width, y);
        pb.line_to(x + width, y + height);
        pb.line_to(x, y + height);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = self.current.stroke_style.to_paint();
            paint.anti_alias = true;
            let stroke = self.get_stroke();

            self.pixmap
                .stroke_path(&path, &paint, &stroke, self.current.transform, None);
        }
    }

    /// Clear a rectangle
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let rect = tiny_skia::Rect::from_xywh(x, y, width, height);
        if let Some(rect) = rect {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(0, 0, 0, 0));
            paint.blend_mode = tiny_skia::BlendMode::Clear;

            self.pixmap
                .fill_rect(rect, &paint, self.current.transform, None);
        }
    }

    /// Get the pixel data as RGBA bytes
    pub fn get_image_data(&self, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        let pixmap_width = self.pixmap.width();

        for py in y..(y + height) {
            for px in x..(x + width) {
                if px < self.pixmap.width() && py < self.pixmap.height() {
                    let idx = (py * pixmap_width + px) as usize;
                    if let Some(pixel) = self.pixmap.pixels().get(idx) {
                        // PremultipliedColorU8 to RGBA
                        let a = pixel.alpha();
                        if a > 0 {
                            data.push((pixel.red() as u16 * 255 / a as u16) as u8);
                            data.push((pixel.green() as u16 * 255 / a as u16) as u8);
                            data.push((pixel.blue() as u16 * 255 / a as u16) as u8);
                        } else {
                            data.push(0);
                            data.push(0);
                            data.push(0);
                        }
                        data.push(a);
                    } else {
                        data.extend_from_slice(&[0, 0, 0, 0]);
                    }
                } else {
                    data.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        data
    }

    /// Set pixel data from RGBA bytes
    pub fn put_image_data(&mut self, data: &[u8], x: i32, y: i32, width: u32, height: u32) {
        let pixmap_width = self.pixmap.width() as i32;
        let pixmap_height = self.pixmap.height() as i32;

        for py in 0..height as i32 {
            for px in 0..width as i32 {
                let dest_x = x + px;
                let dest_y = y + py;

                if dest_x >= 0 && dest_x < pixmap_width && dest_y >= 0 && dest_y < pixmap_height {
                    let src_idx = ((py * width as i32 + px) * 4) as usize;
                    if src_idx + 3 < data.len() {
                        let r = data[src_idx];
                        let g = data[src_idx + 1];
                        let b = data[src_idx + 2];
                        let a = data[src_idx + 3];

                        // Convert to premultiplied alpha
                        let pr = (r as u16 * a as u16 / 255) as u8;
                        let pg = (g as u16 * a as u16 / 255) as u8;
                        let pb = (b as u16 * a as u16 / 255) as u8;

                        let dest_idx = (dest_y * pixmap_width + dest_x) as usize;
                        if let Some(pixel) = self.pixmap.pixels_mut().get_mut(dest_idx) {
                            if let Some(color) =
                                tiny_skia::PremultipliedColorU8::from_rgba(pr, pg, pb, a)
                            {
                                *pixel = color;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Encode the canvas as PNG
    pub fn to_png(&self) -> Option<Vec<u8>> {
        self.pixmap.encode_png().ok()
    }

    /// Apply a transform to the current matrix
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let new_transform = Transform::from_row(a, b, c, d, e, f);
        self.current.transform = self.current.transform.post_concat(new_transform);
    }

    /// Set the transform matrix directly
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.current.transform = Transform::from_row(a, b, c, d, e, f);
    }

    /// Reset the transform to identity
    pub fn reset_transform(&mut self) {
        self.current.transform = Transform::identity();
    }

    /// Scale the current transform
    pub fn scale(&mut self, x: f32, y: f32) {
        self.current.transform = self.current.transform.post_scale(x, y);
    }

    /// Rotate the current transform
    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        let rotation = Transform::from_row(cos, sin, -sin, cos, 0.0, 0.0);
        self.current.transform = self.current.transform.post_concat(rotation);
    }

    /// Translate the current transform
    pub fn translate(&mut self, x: f32, y: f32) {
        self.current.transform = self.current.transform.post_translate(x, y);
    }
}
