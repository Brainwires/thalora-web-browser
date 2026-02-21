use cosmic_text::{FontSystem, Buffer, Metrics, Attrs, Family, Weight, Shaping};
use parking_lot::Mutex;
use std::sync::OnceLock;

static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

fn get_font_system() -> &'static Mutex<FontSystem> {
    FONT_SYSTEM.get_or_init(|| {
        let mut fs = FontSystem::new();
        load_bundled_fonts(&mut fs);
        Mutex::new(fs)
    })
}

/// Dynamically discover and load all `.ttf`/`.otf` fonts from the project's fonts directory.
///
/// Searches multiple candidate directories so this works in both development builds
/// (relative to `CARGO_MANIFEST_DIR`) and deployed/installed builds (relative to the executable).
fn load_bundled_fonts(fs: &mut FontSystem) {
    let candidates = [
        // Relative to source tree (development builds)
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/gui/fonts"),
        // Relative to executable (installed/deployed)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("fonts")))
            .unwrap_or_default(),
    ];

    for dir in &candidates {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| {
                    ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf")
                }) {
                    if let Ok(data) = std::fs::read(&path) {
                        fs.db_mut().load_font_data(data);
                    }
                }
            }
            return; // Found a valid fonts directory, stop searching
        }
    }
    eprintln!("[text_measure] Warning: No fonts directory found");
}

pub struct TextMeasurement {
    pub width: f32,
    pub height: f32,
    pub num_lines: usize,
}

/// Measure text with real font shaping and line wrapping.
///
/// - `font_family_css`: CSS font-family string, e.g. `"Arial, sans-serif"`
/// - `font_weight_css`: CSS font-weight string, e.g. `"bold"`, `"700"`
/// - `available_width`: container width for line wrapping (None = no wrap)
/// - `line_height_multiplier`: e.g. 1.4
pub fn measure_text(
    text: &str,
    font_family_css: &str,
    font_size_px: f32,
    font_weight_css: Option<&str>,
    available_width: Option<f32>,
    line_height_multiplier: f32,
) -> TextMeasurement {
    if text.is_empty() {
        return TextMeasurement {
            width: 0.0,
            height: font_size_px * line_height_multiplier,
            num_lines: 1,
        };
    }

    let mut font_system = get_font_system().lock();

    let line_height_px = font_size_px * line_height_multiplier;
    let metrics = Metrics::new(font_size_px, line_height_px);
    let mut buffer = Buffer::new(&mut font_system, metrics);

    buffer.set_size(&mut font_system, available_width, None);

    let family = parse_font_family(font_family_css);
    let weight = parse_font_weight(font_weight_css);
    let attrs = Attrs::new().family(family).weight(weight);

    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system, true);

    let mut max_width: f32 = 0.0;
    let mut total_height: f32 = 0.0;
    let mut num_lines: usize = 0;

    for run in buffer.layout_runs() {
        max_width = max_width.max(run.line_w);
        total_height += run.line_height;
        num_lines += 1;
    }

    if num_lines == 0 {
        total_height = line_height_px;
        num_lines = 1;
    }

    TextMeasurement { width: max_width, height: total_height, num_lines }
}

/// Parse CSS font-family string to cosmic-text Family.
/// Maps generic CSS family names to cosmic-text equivalents; defaults to SansSerif.
fn parse_font_family(css: &str) -> Family<'static> {
    for name in css.split(',') {
        let name = name.trim().trim_matches(|c| c == '"' || c == '\'');
        match name.to_lowercase().as_str() {
            "serif" => return Family::Serif,
            "sans-serif" => return Family::SansSerif,
            "monospace" => return Family::Monospace,
            "cursive" => return Family::Cursive,
            "fantasy" => return Family::Fantasy,
            _ => {}
        }
    }
    Family::SansSerif
}

/// Parse CSS font-weight string to cosmic-text Weight.
fn parse_font_weight(css: Option<&str>) -> Weight {
    match css {
        Some("bold" | "bolder") => Weight::BOLD,
        Some("lighter") => Weight(300),
        Some("normal") => Weight::NORMAL,
        Some(n) => n.parse::<u16>().map(Weight).unwrap_or(Weight::NORMAL),
        None => Weight::NORMAL,
    }
}

/// A single visual line of text as determined by cosmic-text line breaking.
pub struct TextLine {
    /// The text substring for this visual line.
    pub text: String,
    /// Width of this line in pixels.
    pub width: f32,
    /// Height of this line in pixels.
    pub height: f32,
    /// Y offset from the top of the text block.
    pub y_offset: f32,
}

/// Result of measuring text into individual visual lines.
pub struct TextLinesResult {
    /// Per-visual-line data.
    pub lines: Vec<TextLine>,
    /// Maximum width across all lines.
    pub total_width: f32,
    /// Sum of all line heights.
    pub total_height: f32,
}

/// Measure text and return per-visual-line data.
///
/// Uses the same font setup as `measure_text()` but returns individual lines
/// so the C# rendering side can render each line without re-wrapping.
pub fn measure_text_lines(
    text: &str,
    font_family_css: &str,
    font_size_px: f32,
    font_weight_css: Option<&str>,
    available_width: Option<f32>,
    line_height_multiplier: f32,
) -> TextLinesResult {
    if text.is_empty() {
        return TextLinesResult {
            lines: vec![TextLine {
                text: String::new(),
                width: 0.0,
                height: font_size_px * line_height_multiplier,
                y_offset: 0.0,
            }],
            total_width: 0.0,
            total_height: font_size_px * line_height_multiplier,
        };
    }

    let mut font_system = get_font_system().lock();

    let line_height_px = font_size_px * line_height_multiplier;
    let metrics = Metrics::new(font_size_px, line_height_px);
    let mut buffer = Buffer::new(&mut font_system, metrics);

    buffer.set_size(&mut font_system, available_width, None);

    let family = parse_font_family(font_family_css);
    let weight = parse_font_weight(font_weight_css);
    let attrs = Attrs::new().family(family).weight(weight);

    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system, true);

    let mut lines = Vec::new();
    let mut max_width: f32 = 0.0;
    let mut y_offset: f32 = 0.0;

    for run in buffer.layout_runs() {
        // Extract the text substring for this visual line from glyph byte offsets
        let line_text = if run.glyphs.is_empty() {
            // Empty run (e.g. trailing newline) — use empty string
            String::new()
        } else {
            let start = run.glyphs.iter().map(|g| g.start).min().unwrap_or(0);
            let end = run.glyphs.iter().map(|g| g.end).max().unwrap_or(0);
            run.text.get(start..end).unwrap_or("").to_string()
        };

        max_width = max_width.max(run.line_w);

        lines.push(TextLine {
            text: line_text,
            width: run.line_w,
            height: run.line_height,
            y_offset,
        });

        y_offset += run.line_height;
    }

    let total_height = if lines.is_empty() {
        lines.push(TextLine {
            text: text.to_string(),
            width: 0.0,
            height: line_height_px,
            y_offset: 0.0,
        });
        line_height_px
    } else {
        y_offset
    };

    TextLinesResult {
        lines,
        total_width: max_width,
        total_height,
    }
}
