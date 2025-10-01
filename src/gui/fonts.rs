//! Font management system for the GUI browser
//!
//! This module provides font loading, caching, and rendering support for the browser.
//! It handles system fonts, web fonts, and font fallback chains.

use egui::{FontId, FontFamily, FontDefinitions, TextStyle};
use std::collections::HashMap;
use anyhow::Result;

/// Font weight values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl FontWeight {
    /// Parse font weight from CSS value
    pub fn from_css(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "100" | "thin" => FontWeight::Thin,
            "200" | "extra-light" | "extralight" => FontWeight::ExtraLight,
            "300" | "light" => FontWeight::Light,
            "400" | "normal" | "regular" => FontWeight::Normal,
            "500" | "medium" => FontWeight::Medium,
            "600" | "semi-bold" | "semibold" | "demi-bold" | "demibold" => FontWeight::SemiBold,
            "700" | "bold" => FontWeight::Bold,
            "800" | "extra-bold" | "extrabold" | "ultra-bold" | "ultrabold" => FontWeight::ExtraBold,
            "900" | "black" | "heavy" => FontWeight::Black,
            "bolder" => FontWeight::Bold, // Simplified - should be relative
            "lighter" => FontWeight::Light, // Simplified - should be relative
            _ => FontWeight::Normal,
        }
    }

    /// Check if this weight should render as bold
    pub fn is_bold(&self) -> bool {
        (*self as i32) >= 600
    }
}

/// Font style (italic, oblique, normal)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    /// Parse font style from CSS value
    pub fn from_css(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            _ => FontStyle::Normal,
        }
    }
}

/// Font descriptor combining family, weight, and style
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub size: f32,
}

impl FontDescriptor {
    /// Create a new font descriptor
    pub fn new(family: String, size: f32) -> Self {
        Self {
            family,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            size,
        }
    }

    /// Create with weight
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Create with style
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    /// Parse font family from CSS value
    pub fn parse_family(css_family: &str) -> String {
        // Extract first font family from CSS font-family value
        // Example: "'Helvetica Neue', Helvetica, Arial, sans-serif"
        let families: Vec<&str> = css_family.split(',').collect();

        if let Some(first) = families.first() {
            let family = first.trim();
            // Remove quotes
            let family = family.trim_matches('\'').trim_matches('"');

            // Map common font families to egui equivalents
            match family.to_lowercase().as_str() {
                "serif" | "times" | "times new roman" | "georgia" => "serif".to_string(),
                "sans-serif" | "arial" | "helvetica" | "verdana" | "tahoma" => "sans-serif".to_string(),
                "monospace" | "courier" | "courier new" | "monaco" | "consolas" => "monospace".to_string(),
                _ => family.to_string(),
            }
        } else {
            "sans-serif".to_string()
        }
    }

    /// Convert to egui FontId
    pub fn to_egui_font_id(&self) -> FontId {
        let family = match self.family.to_lowercase().as_str() {
            "serif" => FontFamily::Proportional,
            "sans-serif" => FontFamily::Proportional,
            "monospace" => FontFamily::Monospace,
            _ => FontFamily::Proportional,
        };

        FontId::new(self.size, family)
    }
}

/// Font manager for loading and managing fonts
pub struct FontManager {
    font_cache: HashMap<String, Vec<u8>>,
    default_font_family: String,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        Self {
            font_cache: HashMap::new(),
            default_font_family: "sans-serif".to_string(),
        }
    }

    /// Configure egui fonts with better defaults
    pub fn configure_egui_fonts(&self) -> FontDefinitions {
        // Use egui's default fonts which are already embedded
        // This avoids font loading errors and provides good cross-platform support
        FontDefinitions::default()
    }

    /// Load a custom font from bytes
    pub fn load_font(&mut self, name: String, font_data: Vec<u8>) -> Result<()> {
        self.font_cache.insert(name, font_data);
        Ok(())
    }

    /// Get font data by name
    pub fn get_font(&self, name: &str) -> Option<&Vec<u8>> {
        self.font_cache.get(name)
    }

    /// Install fonts into egui context
    pub fn install_fonts(&self, ctx: &egui::Context) {
        let fonts = self.configure_egui_fonts();
        ctx.set_fonts(fonts);

        // Configure text styles with better sizes
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
        ]
        .into();

        ctx.set_style(style);
    }

    /// Get default font family
    pub fn default_family(&self) -> &str {
        &self.default_font_family
    }

    /// Set default font family
    pub fn set_default_family(&mut self, family: String) {
        self.default_font_family = family;
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Font size utilities
pub struct FontSize;

impl FontSize {
    /// Parse font size from CSS value (e.g., "16px", "1.2em", "large")
    pub fn parse_css(value: &str, base_size: f32) -> f32 {
        let value = value.trim().to_lowercase();

        // Absolute keywords
        match value.as_str() {
            "xx-small" => return 9.0,
            "x-small" => return 10.0,
            "small" => return 13.0,
            "medium" => return 16.0,
            "large" => return 18.0,
            "x-large" => return 24.0,
            "xx-large" => return 32.0,
            "xxx-large" => return 48.0,
            _ => {}
        }

        // Relative keywords
        match value.as_str() {
            "smaller" => return base_size * 0.85,
            "larger" => return base_size * 1.2,
            _ => {}
        }

        // Parse numeric values
        if value.ends_with("px") {
            if let Ok(px) = value.trim_end_matches("px").parse::<f32>() {
                return px;
            }
        } else if value.ends_with("pt") {
            if let Ok(pt) = value.trim_end_matches("pt").parse::<f32>() {
                return pt * 1.333; // 1pt = 1.333px
            }
        } else if value.ends_with("em") {
            if let Ok(em) = value.trim_end_matches("em").parse::<f32>() {
                return base_size * em;
            }
        } else if value.ends_with("rem") {
            if let Ok(rem) = value.trim_end_matches("rem").parse::<f32>() {
                return 16.0 * rem; // rem is relative to root (16px default)
            }
        } else if value.ends_with("%") {
            if let Ok(percent) = value.trim_end_matches("%").parse::<f32>() {
                return base_size * (percent / 100.0);
            }
        } else if let Ok(num) = value.parse::<f32>() {
            // Unitless value (assume pixels)
            return num;
        }

        base_size
    }

    /// Get default size for heading level
    pub fn heading_size(level: u8) -> f32 {
        match level {
            1 => 32.0,
            2 => 24.0,
            3 => 20.0,
            4 => 18.0,
            5 => 16.0,
            6 => 14.0,
            _ => 16.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_weight_parsing() {
        assert_eq!(FontWeight::from_css("400"), FontWeight::Normal);
        assert_eq!(FontWeight::from_css("bold"), FontWeight::Bold);
        assert_eq!(FontWeight::from_css("700"), FontWeight::Bold);
        assert!(FontWeight::Bold.is_bold());
        assert!(!FontWeight::Normal.is_bold());
    }

    #[test]
    fn test_font_size_parsing() {
        assert_eq!(FontSize::parse_css("16px", 14.0), 16.0);
        assert_eq!(FontSize::parse_css("2em", 14.0), 28.0);
        assert_eq!(FontSize::parse_css("large", 14.0), 18.0);
        assert_eq!(FontSize::parse_css("150%", 14.0), 21.0);
    }

    #[test]
    fn test_font_family_parsing() {
        assert_eq!(
            FontDescriptor::parse_family("'Helvetica Neue', Arial, sans-serif"),
            "sans-serif"
        );
        assert_eq!(
            FontDescriptor::parse_family("monospace"),
            "monospace"
        );
    }
}
