//! Text rendering for Canvas 2D context
//!
//! Provides font discovery, text shaping, and glyph rasterization using fontdb and rustybuzz.
//! This enables real text rendering for canvas fillText/strokeText operations.

use fontdb::{Database, FaceInfo, ID, Source};
use rustybuzz::{Face, UnicodeBuffer, ShapePlan};
use std::sync::Arc;
use tiny_skia::{Color, Pixmap, Paint, PathBuilder, Transform, FillRule};

/// Embedded fallback font (DejaVu Sans subset) for when system fonts aren't available
/// This ensures we always have a font to render with
static FALLBACK_FONT_DATA: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");

/// Text metrics returned by measureText
#[derive(Debug, Clone, Default)]
pub struct TextMetrics {
    /// Width of the text in pixels
    pub width: f64,
    /// Distance from the baseline to the top of the bounding box
    pub actual_bounding_box_ascent: f64,
    /// Distance from the baseline to the bottom of the bounding box
    pub actual_bounding_box_descent: f64,
    /// Distance from the textAlign position to the left edge
    pub actual_bounding_box_left: f64,
    /// Distance from the textAlign position to the right edge
    pub actual_bounding_box_right: f64,
    /// Ascent from the font metrics
    pub font_bounding_box_ascent: f64,
    /// Descent from the font metrics
    pub font_bounding_box_descent: f64,
    /// Em square ascent
    pub em_height_ascent: f64,
    /// Em square descent
    pub em_height_descent: f64,
    /// Distance from baseline to hanging baseline
    pub hanging_baseline: f64,
    /// Distance from baseline to alphabetic baseline (always 0)
    pub alphabetic_baseline: f64,
    /// Distance from baseline to ideographic baseline
    pub ideographic_baseline: f64,
}

/// A shaped glyph ready for rendering
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u16,
    /// X offset from previous glyph position
    pub x_offset: f32,
    /// Y offset from baseline
    pub y_offset: f32,
    /// Advance to next glyph position
    pub x_advance: f32,
    /// Vertical advance (usually 0 for horizontal text)
    pub y_advance: f32,
}

/// Parsed font specification from CSS font string
#[derive(Debug, Clone)]
pub struct ParsedFont {
    /// Font size in pixels
    pub size: f32,
    /// Font family names (in order of preference)
    pub families: Vec<String>,
    /// Font weight (100-900, 400 = normal, 700 = bold)
    pub weight: u16,
    /// Whether font is italic
    pub italic: bool,
}

impl Default for ParsedFont {
    fn default() -> Self {
        Self {
            size: 10.0,
            families: vec!["sans-serif".to_string()],
            weight: 400,
            italic: false,
        }
    }
}

/// Text renderer using fontdb for font discovery and rustybuzz for shaping
pub struct TextRenderer {
    /// Font database with system fonts and fallback
    font_db: Database,
    /// ID of the fallback font
    fallback_font_id: Option<ID>,
}

impl TextRenderer {
    /// Create a new TextRenderer, loading system fonts and embedded fallback
    pub fn new() -> Self {
        let mut font_db = Database::new();

        // Load system fonts
        font_db.load_system_fonts();

        // Load embedded fallback font using Source::Binary which returns IDs
        let fallback_data: Arc<dyn AsRef<[u8]> + Sync + Send> = Arc::new(FALLBACK_FONT_DATA.to_vec());
        let ids = font_db.load_font_source(Source::Binary(fallback_data));
        let fallback_font_id = ids.first().copied();

        Self {
            font_db,
            fallback_font_id,
        }
    }

    /// Parse a CSS font string like "12px Arial" or "bold 16px 'Times New Roman', serif"
    pub fn parse_font(&self, font_str: &str) -> ParsedFont {
        let mut result = ParsedFont::default();
        let font_str = font_str.trim();

        if font_str.is_empty() {
            return result;
        }

        let parts: Vec<&str> = font_str.split_whitespace().collect();
        let mut i = 0;

        // Parse optional font-style and font-weight (can appear in any order)
        // CSS allows: normal | italic | oblique for style
        // CSS allows: normal | bold | bolder | lighter | 100-900 for weight
        while i < parts.len() {
            let part = parts[i].to_lowercase();

            // Check for font-style
            if part == "italic" || part == "oblique" {
                result.italic = true;
                i += 1;
                continue;
            }

            // Check for font-weight
            if part == "bold" {
                result.weight = 700;
                i += 1;
                continue;
            } else if part == "lighter" {
                result.weight = 300;
                i += 1;
                continue;
            } else if part == "bolder" {
                result.weight = 700;
                i += 1;
                continue;
            } else if let Ok(w) = part.parse::<u16>() {
                if (100..=900).contains(&w) {
                    result.weight = w;
                    i += 1;
                    continue;
                }
            }

            // "normal" can apply to either style or weight, just skip it
            if part == "normal" {
                i += 1;
                continue;
            }

            // If we get here, this token is not style or weight, break to parse size
            break;
        }

        // Parse font-size (required)
        if i < parts.len() {
            let size_str = parts[i];
            if let Some(px_size) = size_str.strip_suffix("px") {
                if let Ok(size) = px_size.parse::<f32>() {
                    result.size = size;
                    i += 1;
                }
            } else if let Some(pt_size) = size_str.strip_suffix("pt") {
                if let Ok(size) = pt_size.parse::<f32>() {
                    // 1pt = 1.333px (96/72)
                    result.size = size * 1.333;
                    i += 1;
                }
            } else if let Some(em_size) = size_str.strip_suffix("em") {
                if let Ok(size) = em_size.parse::<f32>() {
                    // Assume base size of 16px
                    result.size = size * 16.0;
                    i += 1;
                }
            }
        }

        // Parse font-family (rest of string, comma-separated)
        if i < parts.len() {
            let family_str = parts[i..].join(" ");
            result.families = family_str
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if result.families.is_empty() {
                result.families = vec!["sans-serif".to_string()];
            }
        }

        result
    }

    /// Find a font face matching the parsed font specification
    fn find_font(&self, parsed: &ParsedFont) -> Option<ID> {
        let weight = fontdb::Weight(parsed.weight);
        let style = if parsed.italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        };

        // Try each family in order
        for family in &parsed.families {
            let family_lower = family.to_lowercase();

            // Handle generic font families
            let query_families = match family_lower.as_str() {
                "serif" => vec!["Times New Roman", "Times", "DejaVu Serif", "Liberation Serif"],
                "sans-serif" => vec!["Arial", "Helvetica", "DejaVu Sans", "Liberation Sans"],
                "monospace" => vec!["Courier New", "Courier", "DejaVu Sans Mono", "Liberation Mono"],
                "cursive" => vec!["Comic Sans MS", "Apple Chancery"],
                "fantasy" => vec!["Impact", "Papyrus"],
                _ => vec![family.as_str()],
            };

            for query_family in query_families {
                // Query fontdb for matching font
                let query = fontdb::Query {
                    families: &[fontdb::Family::Name(query_family)],
                    weight,
                    style,
                    stretch: fontdb::Stretch::Normal,
                };

                if let Some(id) = self.font_db.query(&query) {
                    return Some(id);
                }
            }
        }

        // Fall back to embedded font
        self.fallback_font_id
    }

    /// Shape text using rustybuzz and return positioned glyphs
    pub fn shape_text(&self, text: &str, font_str: &str) -> Vec<ShapedGlyph> {
        let parsed = self.parse_font(font_str);

        let font_id = match self.find_font(&parsed) {
            Some(id) => id,
            None => return Vec::new(),
        };

        // Get font face data
        let face_info = match self.font_db.face(font_id) {
            Some(info) => info,
            None => return Vec::new(),
        };

        // Load font data for rustybuzz
        let font_data = match self.font_db.face_source(font_id) {
            Some((src, index)) => {
                match src {
                    fontdb::Source::Binary(data) => {
                        // data is Arc<dyn AsRef<[u8]> + Sync + Send>
                        let bytes: &[u8] = (*data).as_ref();
                        bytes.to_vec()
                    }
                    fontdb::Source::File(path) => {
                        match std::fs::read(&path) {
                            Ok(data) => data,
                            Err(_) => return Vec::new(),
                        }
                    }
                    fontdb::Source::SharedFile(path, data) => {
                        // data is Arc<dyn AsRef<[u8]> + Sync + Send>
                        let bytes: &[u8] = (*data).as_ref();
                        bytes.to_vec()
                    }
                }
            }
            None => return Vec::new(),
        };

        // Create rustybuzz face
        let face = match Face::from_slice(&font_data, 0) {
            Some(f) => f,
            None => return Vec::new(),
        };

        // Calculate scale factor (rustybuzz uses font units, we need pixels)
        let units_per_em = face.units_per_em() as f32;
        let scale = parsed.size / units_per_em;

        // Create and shape buffer
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);

        let output = rustybuzz::shape(&face, &[], buffer);

        // Convert to ShapedGlyphs
        let positions = output.glyph_positions();
        let infos = output.glyph_infos();

        infos.iter().zip(positions.iter()).map(|(info, pos)| {
            ShapedGlyph {
                glyph_id: info.glyph_id as u16,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
                x_advance: pos.x_advance as f32 * scale,
                y_advance: pos.y_advance as f32 * scale,
            }
        }).collect()
    }

    /// Measure text and return TextMetrics
    pub fn measure(&self, text: &str, font_str: &str) -> TextMetrics {
        let parsed = self.parse_font(font_str);
        let glyphs = self.shape_text(text, font_str);

        // Calculate total width from glyph advances
        let width: f32 = glyphs.iter().map(|g| g.x_advance).sum();

        // Get font metrics
        let font_id = self.find_font(&parsed);
        let (ascent, descent, line_gap) = if let Some(id) = font_id {
            if let Some((src, _)) = self.font_db.face_source(id) {
                let font_data = match src {
                    fontdb::Source::Binary(data) => {
                        let bytes: &[u8] = (*data).as_ref();
                        bytes.to_vec()
                    }
                    fontdb::Source::File(path) => std::fs::read(&path).unwrap_or_default(),
                    fontdb::Source::SharedFile(_, data) => {
                        let bytes: &[u8] = (*data).as_ref();
                        bytes.to_vec()
                    }
                };

                if let Some(face) = Face::from_slice(&font_data, 0) {
                    let units_per_em = face.units_per_em() as f32;
                    let scale = parsed.size / units_per_em;
                    (
                        face.ascender() as f32 * scale,
                        face.descender() as f32 * scale,
                        face.line_gap() as f32 * scale,
                    )
                } else {
                    (parsed.size * 0.8, -parsed.size * 0.2, 0.0)
                }
            } else {
                (parsed.size * 0.8, -parsed.size * 0.2, 0.0)
            }
        } else {
            (parsed.size * 0.8, -parsed.size * 0.2, 0.0)
        };

        TextMetrics {
            width: width as f64,
            actual_bounding_box_ascent: ascent as f64,
            actual_bounding_box_descent: (-descent) as f64, // descent is negative in font metrics
            actual_bounding_box_left: 0.0,
            actual_bounding_box_right: width as f64,
            font_bounding_box_ascent: ascent as f64,
            font_bounding_box_descent: (-descent) as f64,
            em_height_ascent: ascent as f64,
            em_height_descent: (-descent) as f64,
            hanging_baseline: ascent as f64 * 0.8,
            alphabetic_baseline: 0.0,
            ideographic_baseline: (-descent) as f64,
        }
    }

    /// Render text to a pixmap (for fill/stroke operations)
    /// Returns the rendered pixmap and the total width/height
    pub fn render_to_pixmap(
        &self,
        text: &str,
        font_str: &str,
        color: Color,
        is_stroke: bool,
        stroke_width: f32,
    ) -> Option<(Pixmap, f32, f32)> {
        let parsed = self.parse_font(font_str);
        let glyphs = self.shape_text(text, font_str);

        if glyphs.is_empty() {
            return None;
        }

        // Calculate total width and get font metrics for height
        let total_width: f32 = glyphs.iter().map(|g| g.x_advance).sum();

        let font_id = self.find_font(&parsed)?;
        let (src, _) = self.font_db.face_source(font_id)?;

        let font_data = match src {
            fontdb::Source::Binary(data) => {
                let bytes: &[u8] = (*data).as_ref();
                bytes.to_vec()
            }
            fontdb::Source::File(path) => std::fs::read(&path).ok()?,
            fontdb::Source::SharedFile(_, data) => {
                let bytes: &[u8] = (*data).as_ref();
                bytes.to_vec()
            }
        };

        let face = Face::from_slice(&font_data, 0)?;
        let units_per_em = face.units_per_em() as f32;
        let scale = parsed.size / units_per_em;

        let ascent = face.ascender() as f32 * scale;
        let descent = face.descender() as f32 * scale;
        let height = ascent - descent;

        // Add padding for stroke
        let padding = if is_stroke { stroke_width.ceil() as u32 + 2 } else { 2 };

        // Create pixmap with adequate size
        let pixmap_width = (total_width.ceil() as u32 + padding * 2).max(1);
        let pixmap_height = (height.ceil() as u32 + padding * 2).max(1);

        let mut pixmap = Pixmap::new(pixmap_width, pixmap_height)?;

        // Load ttf-parser font for glyph outlines
        let ttf_face = ttf_parser::Face::parse(&font_data, 0).ok()?;

        let mut x_pos = padding as f32;
        let baseline = padding as f32 + ascent;

        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;

        for glyph in &glyphs {
            // Get glyph outline
            if let Some(path) = self.glyph_to_path(&ttf_face, glyph.glyph_id, scale) {
                let transform = Transform::from_translate(
                    x_pos + glyph.x_offset,
                    baseline - glyph.y_offset,
                );

                if is_stroke {
                    let stroke = tiny_skia::Stroke {
                        width: stroke_width,
                        line_cap: tiny_skia::LineCap::Round,
                        line_join: tiny_skia::LineJoin::Round,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                } else {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
                }
            }

            x_pos += glyph.x_advance;
        }

        Some((pixmap, total_width, height))
    }

    /// Convert a glyph outline to a tiny-skia path
    fn glyph_to_path(&self, face: &ttf_parser::Face, glyph_id: u16, scale: f32) -> Option<tiny_skia::Path> {
        use ttf_parser::OutlineBuilder;

        struct PathBuilder {
            path: tiny_skia::PathBuilder,
            scale: f32,
        }

        impl OutlineBuilder for PathBuilder {
            fn move_to(&mut self, x: f32, y: f32) {
                // Flip Y axis (font coordinates have Y going up)
                self.path.move_to(x * self.scale, -y * self.scale);
            }

            fn line_to(&mut self, x: f32, y: f32) {
                self.path.line_to(x * self.scale, -y * self.scale);
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.path.quad_to(
                    x1 * self.scale, -y1 * self.scale,
                    x * self.scale, -y * self.scale,
                );
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.path.cubic_to(
                    x1 * self.scale, -y1 * self.scale,
                    x2 * self.scale, -y2 * self.scale,
                    x * self.scale, -y * self.scale,
                );
            }

            fn close(&mut self) {
                self.path.close();
            }
        }

        let glyph_id = ttf_parser::GlyphId(glyph_id);

        let mut builder = PathBuilder {
            path: tiny_skia::PathBuilder::new(),
            scale,
        };

        face.outline_glyph(glyph_id, &mut builder)?;

        builder.path.finish()
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_simple() {
        let renderer = TextRenderer::new();
        let parsed = renderer.parse_font("12px Arial");
        assert_eq!(parsed.size, 12.0);
        assert_eq!(parsed.families, vec!["Arial"]);
        assert_eq!(parsed.weight, 400);
        assert!(!parsed.italic);
    }

    #[test]
    fn test_parse_font_complex() {
        let renderer = TextRenderer::new();
        let parsed = renderer.parse_font("bold italic 16px 'Times New Roman', serif");
        assert_eq!(parsed.size, 16.0);
        assert!(parsed.families.contains(&"Times New Roman".to_string()));
        assert!(parsed.families.contains(&"serif".to_string()));
        assert_eq!(parsed.weight, 700);
        assert!(parsed.italic);
    }

    #[test]
    fn test_measure_text() {
        let renderer = TextRenderer::new();
        let metrics = renderer.measure("Hello", "12px sans-serif");
        // Width should be positive
        assert!(metrics.width > 0.0);
        // Ascent should be positive
        assert!(metrics.actual_bounding_box_ascent > 0.0);
    }
}
