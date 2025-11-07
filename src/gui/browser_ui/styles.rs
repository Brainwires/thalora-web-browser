//! CSS parsing and style application

use egui::{Color32, RichText};
use super::types::*;
use crate::gui::{FontDescriptor, FontWeight, FontSize};

impl super::BrowserUI {
    /// Create RichText with CSS styling applied
    pub(super) fn create_styled_text(&self, text: &str, css: &CssStyle, default_size: f32) -> RichText {
        let font_size = css.font_size.unwrap_or(default_size);
        let mut rich_text = RichText::new(text).size(font_size);

        // Apply color
        if let Some(color) = css.text_color {
            rich_text = rich_text.color(color);
        }

        // Apply background color
        if let Some(bg_color) = css.bg_color {
            rich_text = rich_text.background_color(bg_color);
        }

        // Apply bold
        if css.font_weight.unwrap_or(false) {
            rich_text = rich_text.strong();
        }

        // Apply font family
        if let Some(family) = &css.font_family {
            let descriptor = FontDescriptor::new(family.clone(), font_size);
            rich_text = rich_text.font(descriptor.to_egui_font_id());
        }

        rich_text
    }

    /// Parse CSS pixel value (e.g., "16px" -> 16.0)
    pub(super) fn parse_px(&self, value: &str) -> Option<f32> {
        value.trim_end_matches("px").parse().ok()
    }

    /// Parse CSS color (basic RGB parser)
    pub(super) fn parse_color(&self, color: &str) -> Option<Color32> {
        if color.starts_with("rgb(") && color.ends_with(')') {
            let rgb_str = &color[4..color.len()-1];
            let parts: Vec<&str> = rgb_str.split(',').collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].trim().parse().ok()?;
                let g: u8 = parts[1].trim().parse().ok()?;
                let b: u8 = parts[2].trim().parse().ok()?;
                return Some(Color32::from_rgb(r, g, b));
            }
        }
        None
    }

    /// Parse inline style attribute for all CSS properties
    pub(super) fn parse_inline_style(&self, style: &str) -> CssStyle {
        let mut css_style = CssStyle::default();

        for rule in style.split(';') {
            let parts: Vec<&str> = rule.split(':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "color" => {
                        css_style.text_color = self.parse_css_color(parts[1]);
                    }
                    "background-color" | "background" => {
                        css_style.bg_color = self.parse_css_color(parts[1]);
                    }
                    "font-size" => {
                        let size = FontSize::parse_css(parts[1], self.base_font_size);
                        css_style.font_size = Some(size);
                    }
                    "font-weight" => {
                        let weight = FontWeight::from_css(parts[1]);
                        css_style.font_weight = Some(weight.is_bold());
                    }
                    "font-family" => {
                        let family = FontDescriptor::parse_family(parts[1]);
                        css_style.font_family = Some(family);
                    }
                    "text-align" => {
                        css_style.text_align = match parts[1].trim() {
                            "left" => Some(TextAlign::Left),
                            "center" => Some(TextAlign::Center),
                            "right" => Some(TextAlign::Right),
                            "justify" => Some(TextAlign::Justify),
                            _ => None,
                        };
                    }
                    "padding" => {
                        css_style.padding = self.parse_css_spacing(parts[1]);
                    }
                    "padding-top" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.top = val;
                        }
                    }
                    "padding-bottom" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.bottom = val;
                        }
                    }
                    "padding-left" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.left = val;
                        }
                    }
                    "padding-right" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.padding.right = val;
                        }
                    }
                    "margin" => {
                        css_style.margin = self.parse_css_spacing(parts[1]);
                    }
                    "margin-top" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.top = val;
                        }
                    }
                    "margin-bottom" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.bottom = val;
                        }
                    }
                    "margin-left" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.left = val;
                        }
                    }
                    "margin-right" => {
                        if let Some(val) = self.parse_css_size(parts[1]) {
                            css_style.margin.right = val;
                        }
                    }
                    "border" => {
                        // Parse shorthand border: "1px solid black"
                        let border_parts: Vec<&str> = parts[1].split_whitespace().collect();
                        if border_parts.len() >= 1 {
                            css_style.border_width = self.parse_css_size(border_parts[0]);
                        }
                        if border_parts.len() >= 3 {
                            css_style.border_color = self.parse_css_color(border_parts[2]);
                        }
                    }
                    "border-width" => {
                        css_style.border_width = self.parse_css_size(parts[1]);
                    }
                    "border-color" => {
                        css_style.border_color = self.parse_css_color(parts[1]);
                    }
                    "border-radius" => {
                        css_style.border_radius = self.parse_css_size(parts[1]);
                    }
                    "display" => {
                        css_style.display = match parts[1].trim() {
                            "block" => Some(DisplayType::Block),
                            "inline" => Some(DisplayType::Inline),
                            "flex" => Some(DisplayType::Flex),
                            "none" => Some(DisplayType::None),
                            _ => None,
                        };
                    }
                    "width" => {
                        css_style.width = self.parse_css_size(parts[1]);
                    }
                    "height" => {
                        css_style.height = self.parse_css_size(parts[1]);
                    }
                    _ => {}
                }
            }
        }

        css_style
    }

    /// Parse CSS size values (px, pt, em, rem, %)
    pub(super) fn parse_css_size(&self, size: &str) -> Option<f32> {
        let size = size.trim();

        if size.ends_with("px") {
            size.trim_end_matches("px").parse::<f32>().ok()
        } else if size.ends_with("pt") {
            // Convert pt to px (1pt = 1.333px)
            size.trim_end_matches("pt").parse::<f32>().ok().map(|v| v * 1.333)
        } else if size.ends_with("em") || size.ends_with("rem") {
            // Convert em/rem to px (assume 14px base)
            size.trim_end_matches("em").trim_end_matches("rem").parse::<f32>().ok().map(|v| v * 14.0)
        } else if size.ends_with("%") {
            // For percentage, just strip the % and parse
            size.trim_end_matches("%").parse::<f32>().ok()
        } else {
            // Try parsing as raw number (assume px)
            size.parse::<f32>().ok()
        }
    }

    /// Parse CSS spacing values (can be 1-4 values: "10px" or "10px 20px" or "10px 20px 30px 40px")
    pub(super) fn parse_css_spacing(&self, spacing: &str) -> CssSpacing {
        let parts: Vec<&str> = spacing.split_whitespace().collect();
        let mut result = CssSpacing::default();

        match parts.len() {
            1 => {
                // All sides same
                if let Some(val) = self.parse_css_size(parts[0]) {
                    result.top = val;
                    result.bottom = val;
                    result.left = val;
                    result.right = val;
                }
            }
            2 => {
                // top/bottom, left/right
                if let Some(tb) = self.parse_css_size(parts[0]) {
                    result.top = tb;
                    result.bottom = tb;
                }
                if let Some(lr) = self.parse_css_size(parts[1]) {
                    result.left = lr;
                    result.right = lr;
                }
            }
            3 => {
                // top, left/right, bottom
                if let Some(t) = self.parse_css_size(parts[0]) {
                    result.top = t;
                }
                if let Some(lr) = self.parse_css_size(parts[1]) {
                    result.left = lr;
                    result.right = lr;
                }
                if let Some(b) = self.parse_css_size(parts[2]) {
                    result.bottom = b;
                }
            }
            4 => {
                // top, right, bottom, left
                if let Some(t) = self.parse_css_size(parts[0]) {
                    result.top = t;
                }
                if let Some(r) = self.parse_css_size(parts[1]) {
                    result.right = r;
                }
                if let Some(b) = self.parse_css_size(parts[2]) {
                    result.bottom = b;
                }
                if let Some(l) = self.parse_css_size(parts[3]) {
                    result.left = l;
                }
            }
            _ => {}
        }

        result
    }

    /// Parse CSS color values (hex, rgb, named colors)
    pub(super) fn parse_css_color(&self, color: &str) -> Option<Color32> {
        let color = color.trim();

        // RGB/RGBA
        if color.starts_with("rgb") {
            return self.parse_color(color);
        }

        // Hex colors
        if color.starts_with('#') {
            let hex = &color[1..];
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Some(Color32::from_rgb(r, g, b));
                }
            } else if hex.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..1].repeat(2), 16),
                    u8::from_str_radix(&hex[1..2].repeat(2), 16),
                    u8::from_str_radix(&hex[2..3].repeat(2), 16),
                ) {
                    return Some(Color32::from_rgb(r, g, b));
                }
            }
        }

        // Named colors
        match color.to_lowercase().as_str() {
            "red" => Some(Color32::from_rgb(255, 0, 0)),
            "green" => Some(Color32::from_rgb(0, 128, 0)),
            "blue" => Some(Color32::from_rgb(0, 0, 255)),
            "black" => Some(Color32::BLACK),
            "white" => Some(Color32::WHITE),
            "gray" | "grey" => Some(Color32::GRAY),
            "yellow" => Some(Color32::from_rgb(255, 255, 0)),
            "orange" => Some(Color32::from_rgb(255, 165, 0)),
            "purple" => Some(Color32::from_rgb(128, 0, 128)),
            _ => None,
        }
    }
}
