//! Headless screenshots: rasterise a [`LayoutResult`] into a PNG.
//!
//! This is an approximate renderer for agents, not a pixel-exact browser
//! paint: it draws backgrounds, borders, image placeholders and text (shaped
//! with cosmic-text) in document order, using the boxes computed by the
//! layout engine.

use super::layout::{ElementLayout, LayoutResult};
use super::page_layout::parse_color_to_rgba;
use super::text_measure::{TextStyle, draw_text};
use anyhow::{Result, anyhow};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

/// Tallest full-page screenshot we produce (pixels).
const MAX_FULL_PAGE_HEIGHT: u32 = 16_000;

/// Screenshot geometry.
#[derive(Debug, Clone, Copy)]
pub struct ScreenshotOptions {
    pub width: u32,
    pub height: u32,
    /// Capture the whole page height instead of just the viewport.
    pub full_page: bool,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            full_page: false,
        }
    }
}

/// Inherited text style while walking the tree.
#[derive(Clone)]
struct Inherited {
    color: (u8, u8, u8, u8),
    font_size: f32,
    font_family: String,
    font_weight: Option<String>,
    line_height: f32,
    opacity: f32,
}

impl Default for Inherited {
    fn default() -> Self {
        Self {
            color: (0, 0, 0, 255),
            font_size: 16.0,
            font_family: "sans-serif".to_string(),
            font_weight: None,
            line_height: 1.2,
            opacity: 1.0,
        }
    }
}

/// Render `layout` to PNG bytes.
pub fn render_png(layout: &LayoutResult, options: ScreenshotOptions) -> Result<Vec<u8>> {
    let pixmap = render(layout, options)?;
    pixmap
        .encode_png()
        .map_err(|e| anyhow!("PNG encoding failed: {e}"))
}

/// Render `layout` to a pixmap (white background).
pub fn render(layout: &LayoutResult, options: ScreenshotOptions) -> Result<Pixmap> {
    let height = if options.full_page {
        (layout.height.ceil() as u32).clamp(options.height, MAX_FULL_PAGE_HEIGHT)
    } else {
        options.height
    };
    let mut pixmap = Pixmap::new(options.width.max(1), height.max(1))
        .ok_or_else(|| anyhow!("invalid screenshot size"))?;
    pixmap.fill(Color::WHITE);
    for element in &layout.elements {
        paint(&mut pixmap, element, &Inherited::default());
    }
    Ok(pixmap)
}

fn paint(pixmap: &mut Pixmap, el: &ElementLayout, parent: &Inherited) {
    if !el.is_visible || el.display.as_deref() == Some("none") {
        return;
    }
    let mut style = parent.clone();
    if let Some(color) = el.color.as_deref().and_then(layout_color) {
        style.color = color;
    }
    if let Some(size) = el.font_size.filter(|s| *s > 0.0) {
        style.font_size = size as f32;
    }
    if let Some(family) = &el.font_family {
        style.font_family = family.clone();
    }
    if el.font_weight.is_some() {
        style.font_weight = el.font_weight.clone();
    }
    if let Some(line_height) = el.line_height.filter(|l| *l > 0.0) {
        style.line_height = line_height as f32;
    }
    style.opacity *= el.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    if style.opacity <= 0.0 {
        return;
    }

    let (x, y, w, h) = (el.x as f32, el.y as f32, el.width as f32, el.height as f32);

    if el.tag == "#text" {
        if let Some(text) = el.text_content.as_deref() {
            let (r, g, b, a) = style.color;
            let alpha = (f32::from(a) * style.opacity).round() as u8;
            draw_text(
                pixmap,
                text,
                x,
                y,
                w.max(1.0),
                &TextStyle {
                    font_family_css: &style.font_family,
                    font_size_px: style.font_size,
                    font_weight_css: style.font_weight.as_deref(),
                    line_height_multiplier: style.line_height,
                    color: (r, g, b, alpha),
                },
            );
        }
        return;
    }

    if let Some(bg) = el.background_color.as_deref().and_then(layout_color) {
        fill_rect(pixmap, x, y, w, h, bg, style.opacity);
    }

    if el.tag == "img" {
        // Images aren't fetched yet: draw a placeholder box
        fill_rect(pixmap, x, y, w, h, (230, 230, 230, 255), style.opacity);
        stroke_box(
            pixmap,
            x,
            y,
            w,
            h,
            [1.0; 4],
            (170, 170, 170, 255),
            style.opacity,
        );
    }

    let border_color = el
        .border_color
        .as_deref()
        .and_then(layout_color)
        .unwrap_or(style.color);
    let widths = match &el.border_sides {
        Some(sides) => [sides.top, sides.right, sides.bottom, sides.left].map(|v| v as f32),
        None => [el.border_width.unwrap_or(0.0) as f32; 4],
    };
    if widths.iter().any(|w| *w > 0.0) {
        stroke_box(pixmap, x, y, w, h, widths, border_color, style.opacity);
    }

    for child in &el.children {
        paint(pixmap, child, &style);
    }
}

/// Fill an axis-aligned rectangle with a non-premultiplied colour.
fn fill_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rgba: (u8, u8, u8, u8),
    opacity: f32,
) {
    let alpha = (f32::from(rgba.3) * opacity).round() as u8;
    if alpha == 0 || w <= 0.0 || h <= 0.0 {
        return;
    }
    let Some(rect) = Rect::from_xywh(x, y, w, h) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(rgba.0, rgba.1, rgba.2, alpha);
    paint.anti_alias = false;
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

/// Draw a border as four filled edges (top, right, bottom, left widths).
#[allow(clippy::too_many_arguments)]
fn stroke_box(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    widths: [f32; 4],
    rgba: (u8, u8, u8, u8),
    opacity: f32,
) {
    let [top, right, bottom, left] = widths;
    fill_rect(pixmap, x, y, w, top, rgba, opacity);
    fill_rect(pixmap, x + w - right, y, right, h, rgba, opacity);
    fill_rect(pixmap, x, y + h - bottom, w, bottom, rgba, opacity);
    fill_rect(pixmap, x, y, left, h, rgba, opacity);
}

/// Parse a colour as stored in [`ElementLayout`]: 8-digit hex is ARGB
/// (`#aarrggbb`, the layout's GUI-oriented format), other forms are CSS.
fn layout_color(s: &str) -> Option<(u8, u8, u8, u8)> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#')
        && hex.len() == 8
        && hex.is_ascii()
    {
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        return Some((byte(2)?, byte(4)?, byte(6)?, byte(0)?));
    }
    if let Some(rgba) = parse_color_to_rgba(s) {
        return Some(rgba);
    }
    let named = match s.to_ascii_lowercase().as_str() {
        "transparent" => (0, 0, 0, 0),
        "black" => (0, 0, 0, 255),
        "white" => (255, 255, 255, 255),
        "red" => (255, 0, 0, 255),
        "green" => (0, 128, 0, 255),
        "blue" => (0, 0, 255, 255),
        "yellow" => (255, 255, 0, 255),
        "orange" => (255, 165, 0, 255),
        "purple" => (128, 0, 128, 255),
        "gray" | "grey" => (128, 128, 128, 255),
        "silver" => (192, 192, 192, 255),
        "lightgray" | "lightgrey" => (211, 211, 211, 255),
        "darkgray" | "darkgrey" => (169, 169, 169, 255),
        "navy" => (0, 0, 128, 255),
        "teal" => (0, 128, 128, 255),
        "maroon" => (128, 0, 0, 255),
        _ => return None,
    };
    Some(named)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::renderer::page_layout::compute_page_layout;

    fn pixel(pixmap: &Pixmap, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let p = pixmap.pixel(x, y).unwrap().demultiply();
        (p.red(), p.green(), p.blue(), p.alpha())
    }

    #[test]
    fn colours_parse_in_layout_formats() {
        assert_eq!(layout_color("#ff0000"), Some((255, 0, 0, 255)));
        assert_eq!(layout_color("#80ff0000"), Some((255, 0, 0, 128)));
        assert_eq!(layout_color("rgb(0, 128, 255)"), Some((0, 128, 255, 255)));
        assert_eq!(layout_color("navy"), Some((0, 0, 128, 255)));
        assert_eq!(layout_color("nonsense"), None);
    }

    #[test]
    fn solid_box_is_painted_at_its_layout_position() {
        let html = r#"<html><body style="margin:0">
            <div style="position:absolute; left:10px; top:10px; width:100px; height:100px; background-color:#ff0000"></div>
        </body></html>"#;
        let layout = compute_page_layout(html, 400.0, 300.0).unwrap();
        let pixmap = render(
            &layout,
            ScreenshotOptions {
                width: 400,
                height: 300,
                full_page: false,
            },
        )
        .unwrap();
        // Somewhere inside the 100x100 red box, whatever offset the layout chose
        let red_pixels = (0..400)
            .flat_map(|x| (0..300).map(move |y| (x, y)))
            .filter(|&(x, y)| pixel(&pixmap, x, y) == (255, 0, 0, 255))
            .count();
        assert!(red_pixels >= 90 * 90, "red pixels: {red_pixels}");
        // The page background stays white
        assert_eq!(pixel(&pixmap, 399, 299), (255, 255, 255, 255));
    }

    #[test]
    fn text_produces_dark_pixels() {
        let html =
            "<html><body><p style=\"font-size:32px;color:#000000\">Hello Thalora</p></body></html>";
        let layout = compute_page_layout(html, 400.0, 200.0).unwrap();
        let pixmap = render(
            &layout,
            ScreenshotOptions {
                width: 400,
                height: 200,
                full_page: false,
            },
        )
        .unwrap();
        let dark = (0..400)
            .flat_map(|x| (0..200).map(move |y| (x, y)))
            .filter(|&(x, y)| pixel(&pixmap, x, y).0 < 128)
            .count();
        assert!(dark > 50, "expected glyph pixels, found {dark}");
    }

    #[test]
    fn png_encoding_works_and_full_page_grows() {
        let html = "<html><body><div style=\"height:3000px\">tall</div></body></html>";
        let layout = compute_page_layout(html, 800.0, 600.0).unwrap();
        let viewport = render(&layout, ScreenshotOptions::default()).unwrap();
        assert_eq!(viewport.height(), 800);
        let full = render(
            &layout,
            ScreenshotOptions {
                full_page: true,
                ..ScreenshotOptions::default()
            },
        )
        .unwrap();
        assert!(full.height() >= 3000, "height {}", full.height());
        let png = render_png(&layout, ScreenshotOptions::default()).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }
}
