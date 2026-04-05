//! Canvas 2D text rendering with real font shaping and glyph rasterization.
//!
//! Uses fontdb for font discovery, rustybuzz for text shaping, and ttf_parser
//! for glyph outlines. Glyphs are converted to tiny-skia paths for rendering.

use fontdb::{Database, Family, Query, Style, Weight};
use rustybuzz::ttf_parser::{self, GlyphId, OutlineBuilder};
use std::sync::LazyLock;
use tiny_skia::{FillRule, PathBuilder, Transform};

use super::canvas_state::CanvasState;

// ---------------------------------------------------------------------------
// Shared font database (loaded once, system + bundled fonts)
// ---------------------------------------------------------------------------

static FONT_DB: LazyLock<Database> = LazyLock::new(|| {
    let mut db = Database::new();

    // Load system fonts first
    db.load_system_fonts();

    // Also load bundled fonts from the project's fonts directories
    let candidates = [
        // Shared fonts directory used by both Rust and C# GUI (development builds)
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../src/gui/fonts"),
        // Legacy path for backwards compatibility
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fonts"),
        // Relative to executable (installed/deployed)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("fonts")))
            .unwrap_or_default(),
    ];

    for dir in &candidates {
        if dir.is_dir() {
            db.load_fonts_dir(dir);
        }
    }

    db
});

// ---------------------------------------------------------------------------
// CSS font property parsing
// ---------------------------------------------------------------------------

/// Parsed representation of the CSS `font` shorthand property.
#[derive(Debug, Clone)]
pub struct ParsedFont {
    pub style: FontStyle,
    pub weight: u16,
    pub size_px: f32,
    pub families: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for ParsedFont {
    fn default() -> Self {
        Self {
            style: FontStyle::Normal,
            weight: 400,
            size_px: 10.0,
            families: vec!["sans-serif".to_string()],
        }
    }
}

/// Parse the canvas `font` CSS shorthand, e.g. "bold italic 16px Arial, sans-serif".
///
/// Simplified parser that handles the most common patterns:
///   [style] [weight] size[/lineHeight] family[, family]*
pub fn parse_css_font(font_str: &str) -> ParsedFont {
    let font_str = font_str.trim();
    if font_str.is_empty() {
        return ParsedFont::default();
    }

    let mut result = ParsedFont::default();

    // Split into tokens, but we need to be careful about the family list after size.
    // Strategy: scan tokens left-to-right. Once we see a size token (ends with px/pt/em/%),
    // everything after that is the family list.
    let mut tokens: Vec<&str> = Vec::new();
    let mut family_start = None;

    // We'll split by whitespace, but need to find where size token is
    let parts: Vec<&str> = font_str.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        if is_size_token(part) {
            // Everything before this is style/weight, this is size,
            // everything after is family
            tokens.push(part);
            if i + 1 < parts.len() {
                family_start = Some(i + 1);
            }
            break;
        }
        tokens.push(part);
    }

    // Parse style/weight tokens (everything before the size token)
    let size_idx = tokens.len().saturating_sub(1);
    for (i, token) in tokens.iter().enumerate() {
        if i == size_idx {
            // This is the size token
            result.size_px = parse_size_token(token);
        } else {
            match token.to_lowercase().as_str() {
                "italic" => result.style = FontStyle::Italic,
                "oblique" => result.style = FontStyle::Oblique,
                "normal" => {} // default
                "bold" => result.weight = 700,
                "bolder" => result.weight = 700,
                "lighter" => result.weight = 300,
                other => {
                    if let Ok(w) = other.parse::<u16>() {
                        if (1..=1000).contains(&w) {
                            result.weight = w;
                        }
                    }
                }
            }
        }
    }

    // Parse family list
    if let Some(start) = family_start {
        let family_str = parts[start..].join(" ");
        result.families = family_str
            .split(',')
            .map(|f| f.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
            .filter(|f| !f.is_empty())
            .collect();
    }

    if result.families.is_empty() {
        result.families.push("sans-serif".to_string());
    }

    result
}

fn is_size_token(s: &str) -> bool {
    // Handle "16px", "12pt", "1.5em", "100%", "16px/20px" (with line-height)
    let s = if let Some(idx) = s.find('/') {
        &s[..idx]
    } else {
        s
    };
    s.ends_with("px")
        || s.ends_with("pt")
        || s.ends_with("em")
        || s.ends_with("rem")
        || s.ends_with('%')
        || s.parse::<f32>().is_ok()
}

fn parse_size_token(s: &str) -> f32 {
    // Strip optional line-height suffix (e.g. "16px/20px")
    let s = if let Some(idx) = s.find('/') {
        &s[..idx]
    } else {
        s
    };

    if let Some(v) = s.strip_suffix("px") {
        v.parse().unwrap_or(10.0)
    } else if let Some(v) = s.strip_suffix("pt") {
        // 1pt = 1.333px
        v.parse::<f32>().unwrap_or(10.0) * 1.333
    } else if let Some(v) = s.strip_suffix("em") {
        // Assume 1em = 16px (default browser font size)
        v.parse::<f32>().unwrap_or(1.0) * 16.0
    } else if let Some(v) = s.strip_suffix("rem") {
        v.parse::<f32>().unwrap_or(1.0) * 16.0
    } else if let Some(v) = s.strip_suffix('%') {
        v.parse::<f32>().unwrap_or(100.0) / 100.0 * 16.0
    } else {
        s.parse().unwrap_or(10.0)
    }
}

// ---------------------------------------------------------------------------
// Font resolution: CSS family name -> fontdb ID -> font data
// ---------------------------------------------------------------------------

fn css_family_to_fontdb(name: &str) -> Family<'_> {
    match name.to_lowercase().as_str() {
        "serif" => Family::Serif,
        "sans-serif" => Family::SansSerif,
        "monospace" => Family::Monospace,
        "cursive" => Family::Cursive,
        "fantasy" => Family::Fantasy,
        "system-ui" | "-apple-system" | "blinkmacsystemfont" => Family::SansSerif,
        _ => Family::Name(name),
    }
}

/// Resolve the parsed font to a fontdb face ID.
fn resolve_font_id(parsed: &ParsedFont) -> Option<fontdb::ID> {
    let db = &*FONT_DB;

    let weight = Weight(parsed.weight);
    let style = match parsed.style {
        FontStyle::Normal => Style::Normal,
        FontStyle::Italic => Style::Italic,
        FontStyle::Oblique => Style::Oblique,
    };

    // Try each family in order
    for family_name in &parsed.families {
        let family = css_family_to_fontdb(family_name);
        let query = Query {
            families: &[family],
            weight,
            stretch: fontdb::Stretch::Normal,
            style,
        };
        if let Some(id) = db.query(&query) {
            return Some(id);
        }
    }

    // Fallback: sans-serif with normal weight
    let query = Query {
        families: &[Family::SansSerif],
        weight: Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: Style::Normal,
    };
    db.query(&query)
}

// ---------------------------------------------------------------------------
// Glyph outline -> tiny-skia path builder adapter
// ---------------------------------------------------------------------------

struct SkiaOutlineBuilder {
    builder: PathBuilder,
    /// Scale factor: font_size_px / units_per_em
    scale: f32,
    /// Vertical flip: font coordinates have Y-up, canvas has Y-down
    /// We store the ascender in px to flip properly.
    ascender_px: f32,
}

impl SkiaOutlineBuilder {
    fn new(scale: f32, ascender_px: f32) -> Self {
        Self {
            builder: PathBuilder::new(),
            scale,
            ascender_px,
        }
    }

    /// Transform from font units to canvas pixels (y-flip).
    fn tx(&self, x: f32) -> f32 {
        x * self.scale
    }
    fn ty(&self, y: f32) -> f32 {
        self.ascender_px - y * self.scale
    }
}

impl OutlineBuilder for SkiaOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(self.tx(x), self.ty(y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.tx(x), self.ty(y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder
            .quad_to(self.tx(x1), self.ty(y1), self.tx(x), self.ty(y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_to(
            self.tx(x1),
            self.ty(y1),
            self.tx(x2),
            self.ty(y2),
            self.tx(x),
            self.ty(y),
        );
    }
    fn close(&mut self) {
        self.builder.close();
    }
}

// ---------------------------------------------------------------------------
// Text metrics
// ---------------------------------------------------------------------------

/// Real text metrics from font shaping.
#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width: f64,
    pub actual_bounding_box_left: f64,
    pub actual_bounding_box_right: f64,
    pub font_bounding_box_ascent: f64,
    pub font_bounding_box_descent: f64,
    pub actual_bounding_box_ascent: f64,
    pub actual_bounding_box_descent: f64,
    pub em_height_ascent: f64,
    pub em_height_descent: f64,
    pub hanging_baseline: f64,
    pub alphabetic_baseline: f64,
    pub ideographic_baseline: f64,
}

/// Measure text using real font shaping.
pub fn real_measure_text(text: &str, font_str: &str) -> TextMetrics {
    let parsed = parse_css_font(font_str);
    let font_size = parsed.size_px;

    if text.is_empty() {
        return TextMetrics {
            width: 0.0,
            actual_bounding_box_left: 0.0,
            actual_bounding_box_right: 0.0,
            font_bounding_box_ascent: (font_size * 0.8) as f64,
            font_bounding_box_descent: (font_size * 0.2) as f64,
            actual_bounding_box_ascent: 0.0,
            actual_bounding_box_descent: 0.0,
            em_height_ascent: (font_size * 0.8) as f64,
            em_height_descent: (font_size * 0.2) as f64,
            hanging_baseline: (font_size * 0.8) as f64,
            alphabetic_baseline: 0.0,
            ideographic_baseline: (font_size * -0.2) as f64,
        };
    }

    let db = &*FONT_DB;
    let font_id = match resolve_font_id(&parsed) {
        Some(id) => id,
        None => return fallback_metrics(text, font_size),
    };

    db.with_face_data(font_id, |font_data, face_index| {
        let ttf_face = match ttf_parser::Face::parse(font_data, face_index) {
            Ok(f) => f,
            Err(_) => return fallback_metrics(text, font_size),
        };

        let upem = ttf_face.units_per_em() as f32;
        let scale = font_size / upem;
        let ascender = ttf_face.ascender() as f32 * scale;
        let descender = ttf_face.descender() as f32 * scale; // negative value

        // Shape text with rustybuzz
        let mut rb_face = match rustybuzz::Face::from_slice(font_data, face_index) {
            Some(f) => f,
            None => return fallback_metrics(text, font_size),
        };
        rb_face.set_pixels_per_em(Some((font_size as u16, font_size as u16)));

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        let glyph_buffer = rustybuzz::shape(&rb_face, &[], buffer);

        let positions = glyph_buffer.glyph_positions();
        let infos = glyph_buffer.glyph_infos();

        // Calculate total advance width
        let total_advance: f32 = positions.iter().map(|p| p.x_advance as f32).sum::<f32>() * scale;

        // Calculate actual bounding box by checking each glyph
        let mut min_x: f32 = 0.0;
        let mut max_x: f32 = 0.0;
        let mut min_y: f32 = 0.0; // above baseline (ascent)
        let mut max_y: f32 = 0.0; // below baseline (descent)
        let mut cursor_x: f32 = 0.0;

        for (pos, info) in positions.iter().zip(infos.iter()) {
            let glyph_id = GlyphId(info.glyph_id as u16);
            let x_off = pos.x_offset as f32 * scale;

            if let Some(bbox) = ttf_face.glyph_bounding_box(glyph_id) {
                let gx_min = cursor_x + x_off + bbox.x_min as f32 * scale;
                let gx_max = cursor_x + x_off + bbox.x_max as f32 * scale;
                let gy_min = bbox.y_min as f32 * scale; // descent (negative usually)
                let gy_max = bbox.y_max as f32 * scale; // ascent

                if gx_min < min_x {
                    min_x = gx_min;
                }
                if gx_max > max_x {
                    max_x = gx_max;
                }
                if gy_max > min_y {
                    min_y = gy_max; // ascent above baseline
                }
                if gy_min < max_y {
                    max_y = gy_min; // descent below baseline
                }
            }

            cursor_x += pos.x_advance as f32 * scale;
        }

        TextMetrics {
            width: total_advance as f64,
            actual_bounding_box_left: (-min_x) as f64,
            actual_bounding_box_right: max_x as f64,
            font_bounding_box_ascent: ascender as f64,
            font_bounding_box_descent: (-descender) as f64,
            actual_bounding_box_ascent: min_y as f64,
            actual_bounding_box_descent: (-max_y) as f64,
            em_height_ascent: ascender as f64,
            em_height_descent: (-descender) as f64,
            hanging_baseline: (ascender * 0.8) as f64,
            alphabetic_baseline: 0.0,
            ideographic_baseline: descender as f64,
        }
    })
    .unwrap_or_else(|| fallback_metrics(text, font_size))
}

fn fallback_metrics(text: &str, font_size: f32) -> TextMetrics {
    let width = text.len() as f64 * font_size as f64 * 0.6;
    let ascent = font_size as f64 * 0.8;
    let descent = font_size as f64 * 0.2;
    TextMetrics {
        width,
        actual_bounding_box_left: 0.0,
        actual_bounding_box_right: width,
        font_bounding_box_ascent: ascent,
        font_bounding_box_descent: descent,
        actual_bounding_box_ascent: ascent,
        actual_bounding_box_descent: descent,
        em_height_ascent: ascent,
        em_height_descent: descent,
        hanging_baseline: ascent * 0.8,
        alphabetic_baseline: 0.0,
        ideographic_baseline: -descent,
    }
}

// ---------------------------------------------------------------------------
// Text rendering (fillText / strokeText)
// ---------------------------------------------------------------------------

/// Mode for text rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRenderMode {
    Fill,
    Stroke,
}

/// Render text onto the canvas state at (x, y).
///
/// Handles textAlign, textBaseline, and optional maxWidth.
pub fn render_text(
    state: &mut CanvasState,
    text: &str,
    x: f32,
    y: f32,
    max_width: Option<f32>,
    mode: TextRenderMode,
) {
    if text.is_empty() {
        return;
    }

    let parsed = parse_css_font(&state.current.font);
    let font_size = parsed.size_px;

    let db = &*FONT_DB;
    let font_id = match resolve_font_id(&parsed) {
        Some(id) => id,
        None => return,
    };

    db.with_face_data(font_id, |font_data, face_index| {
        let ttf_face = match ttf_parser::Face::parse(font_data, face_index) {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut rb_face = match rustybuzz::Face::from_slice(font_data, face_index) {
            Some(f) => f,
            None => return,
        };
        rb_face.set_pixels_per_em(Some((font_size as u16, font_size as u16)));

        let upem = ttf_face.units_per_em() as f32;
        let scale = font_size / upem;
        let ascender = ttf_face.ascender() as f32 * scale;
        let descender = ttf_face.descender() as f32 * scale; // negative

        // Shape text
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        let glyph_buffer = rustybuzz::shape(&rb_face, &[], buffer);

        let positions = glyph_buffer.glyph_positions();
        let infos = glyph_buffer.glyph_infos();

        // Calculate total advance for alignment
        let total_advance: f32 = positions.iter().map(|p| p.x_advance as f32).sum::<f32>() * scale;

        // Apply maxWidth scaling if needed
        let h_scale = match max_width {
            Some(mw) if total_advance > mw && total_advance > 0.0 => mw / total_advance,
            _ => 1.0,
        };

        let effective_width = total_advance * h_scale;

        // textAlign adjustment
        let align_offset = match state.current.text_align.as_str() {
            "center" => -effective_width / 2.0,
            "right" | "end" => -effective_width,
            // "left" | "start" | _ => 0.0
            _ => 0.0,
        };

        // textBaseline adjustment
        let baseline_offset = match state.current.text_baseline.as_str() {
            "top" => ascender,
            "hanging" => ascender * 0.8,
            "middle" => ascender / 2.0,
            "alphabetic" => 0.0,
            "ideographic" => descender,
            "bottom" => descender,
            _ => 0.0, // alphabetic is default
        };

        let start_x = x + align_offset;
        let start_y = y + baseline_offset;

        // Build a combined path for all glyphs
        let mut combined = PathBuilder::new();
        let mut cursor_x: f32 = 0.0;
        let mut has_glyphs = false;

        for (pos, info) in positions.iter().zip(infos.iter()) {
            let glyph_id = GlyphId(info.glyph_id as u16);
            let glyph_x = cursor_x + pos.x_offset as f32 * scale;
            let glyph_y = pos.y_offset as f32 * scale;

            // Build outline for this glyph
            let mut outline_builder = SkiaOutlineBuilder::new(scale, ascender);
            if ttf_face
                .outline_glyph(glyph_id, &mut outline_builder)
                .is_some()
            {
                if let Some(glyph_path) = outline_builder.builder.finish() {
                    // Translate glyph to its position
                    let glyph_transform = Transform::from_translate(
                        start_x + glyph_x * h_scale,
                        start_y - ascender + glyph_y,
                    );
                    // Apply h_scale for maxWidth compression
                    let glyph_transform = if (h_scale - 1.0).abs() > f32::EPSILON {
                        glyph_transform.post_scale(h_scale, 1.0)
                    } else {
                        glyph_transform
                    };

                    if let Some(transformed) = glyph_path.transform(glyph_transform) {
                        // Merge into combined path by iterating segments
                        for segment in transformed.segments() {
                            match segment {
                                tiny_skia::PathSegment::MoveTo(p) => {
                                    combined.move_to(p.x, p.y);
                                }
                                tiny_skia::PathSegment::LineTo(p) => {
                                    combined.line_to(p.x, p.y);
                                }
                                tiny_skia::PathSegment::QuadTo(p1, p) => {
                                    combined.quad_to(p1.x, p1.y, p.x, p.y);
                                }
                                tiny_skia::PathSegment::CubicTo(p1, p2, p) => {
                                    combined.cubic_to(p1.x, p1.y, p2.x, p2.y, p.x, p.y);
                                }
                                tiny_skia::PathSegment::Close => {
                                    combined.close();
                                }
                            }
                        }
                        has_glyphs = true;
                    }
                }
            }

            cursor_x += pos.x_advance as f32 * scale;
        }

        if !has_glyphs {
            return;
        }

        let Some(final_path) = combined.finish() else {
            return;
        };

        match mode {
            TextRenderMode::Fill => {
                let mut paint = state.current.fill_style.to_paint();
                paint.anti_alias = true;

                // Apply global alpha
                if state.current.global_alpha < 1.0 {
                    if let super::canvas_state::CanvasStyle::Color(ref c) = state.current.fill_style
                    {
                        let new_alpha = (c.alpha() * state.current.global_alpha).min(1.0);
                        if let Some(color) =
                            tiny_skia::Color::from_rgba(c.red(), c.green(), c.blue(), new_alpha)
                        {
                            paint.set_color(color);
                        }
                    }
                }

                state.pixmap.fill_path(
                    &final_path,
                    &paint,
                    FillRule::Winding,
                    state.current.transform,
                    None,
                );
            }
            TextRenderMode::Stroke => {
                let mut paint = state.current.stroke_style.to_paint();
                paint.anti_alias = true;

                if state.current.global_alpha < 1.0 {
                    if let super::canvas_state::CanvasStyle::Color(ref c) =
                        state.current.stroke_style
                    {
                        let new_alpha = (c.alpha() * state.current.global_alpha).min(1.0);
                        if let Some(color) =
                            tiny_skia::Color::from_rgba(c.red(), c.green(), c.blue(), new_alpha)
                        {
                            paint.set_color(color);
                        }
                    }
                }

                let stroke = state.get_stroke();
                state.pixmap.stroke_path(
                    &final_path,
                    &paint,
                    &stroke,
                    state.current.transform,
                    None,
                );
            }
        }
    });
}
