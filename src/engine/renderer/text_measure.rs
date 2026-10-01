use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Weight};
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
        // Shared fonts directory used by both Rust and C# GUI (development builds)
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/gui/fonts"),
        // Legacy path for backwards compatibility
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts"),
        // Relative to executable (installed/deployed)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("fonts")))
            .unwrap_or_default(),
    ];

    for dir in &candidates {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf")
                }) && let Ok(data) = std::fs::read(&path)
                {
                    fs.db_mut().load_font_data(data);
                    count += 1;
                }
            }
            eprintln!(
                "[text_measure] Loaded {} fonts from {}",
                count,
                dir.display()
            );
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

    TextMeasurement {
        width: max_width,
        height: total_height,
        num_lines,
    }
}

/// Parse CSS font-family string to cosmic-text Family.
/// Walks the comma-separated font stack. CSS generic families map to cosmic-text
/// generic families; named fonts (like "Noto Sans") are matched by name against
/// the loaded font database.
fn parse_font_family(css: &str) -> Family<'static> {
    // We need a 'static lifetime for the Family::Name variant. We use a static
    // mapping for common named fonts so we don't allocate per-call.
    for name in css.split(',') {
        let name = name.trim().trim_matches(|c| c == '"' || c == '\'');
        match name.to_lowercase().as_str() {
            // CSS generic families
            "serif" => return Family::Serif,
            "sans-serif" => return Family::SansSerif,
            "monospace" => return Family::Monospace,
            "cursive" => return Family::Cursive,
            "fantasy" => return Family::Fantasy,
            // Named fonts we bundle — use Family::Name for exact matching
            "noto sans" => return Family::Name("Noto Sans"),
            "noto serif" => return Family::Name("Noto Serif"),
            "fira mono" | "fira code" => return Family::Name("Fira Mono"),
            // Common web fonts → map to our bundled equivalents
            "arial" | "helvetica" | "helvetica neue" | "segoe ui" | "roboto" | "open sans"
            | "inter" | "lato" | "poppins" | "-apple-system" | "system-ui"
            | "blinkmacsystemfont" => {
                return Family::Name("Noto Sans");
            }
            "times new roman" | "times" | "georgia" | "palatino" | "garamond" | "cambria" => {
                return Family::Name("Noto Serif");
            }
            "courier new" | "courier" | "consolas" | "menlo" | "monaco" | "source code pro"
            | "sf mono" => {
                return Family::Name("Fira Mono");
            }
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

static SWASH_CACHE: OnceLock<Mutex<SwashCache>> = OnceLock::new();

/// Text drawing parameters for [`draw_text`].
pub struct TextStyle<'a> {
    pub font_family_css: &'a str,
    pub font_size_px: f32,
    pub font_weight_css: Option<&'a str>,
    pub line_height_multiplier: f32,
    /// Non-premultiplied RGBA.
    pub color: (u8, u8, u8, u8),
}

/// Shape `text` (wrapping at `width`) and draw its glyphs into `pixmap` with
/// the top-left corner at (`x`, `y`), clipped to the pixmap.
pub fn draw_text(
    pixmap: &mut tiny_skia::Pixmap,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    style: &TextStyle,
) {
    if text.trim().is_empty() || style.color.3 == 0 {
        return;
    }
    let mut font_system = get_font_system().lock();
    let metrics = Metrics::new(
        style.font_size_px,
        style.font_size_px * style.line_height_multiplier,
    );
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(width.max(1.0)), None);
    let attrs = Attrs::new()
        .family(parse_font_family(style.font_family_css))
        .weight(parse_font_weight(style.font_weight_css));
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system, true);

    let mut cache = SWASH_CACHE
        .get_or_init(|| Mutex::new(SwashCache::new()))
        .lock();
    let (r, g, b, a) = style.color;
    let origin_x = x.round() as i32;
    let origin_y = y.round() as i32;
    buffer.draw(
        &mut font_system,
        &mut cache,
        cosmic_text::Color::rgba(r, g, b, a),
        |gx, gy, w, h, color| {
            for dy in 0..h as i32 {
                for dx in 0..w as i32 {
                    blend_pixel(
                        pixmap,
                        origin_x + gx + dx,
                        origin_y + gy + dy,
                        (color.r(), color.g(), color.b(), color.a()),
                    );
                }
            }
        },
    );
}

/// Source-over blend one non-premultiplied RGBA pixel into `pixmap`.
fn blend_pixel(pixmap: &mut tiny_skia::Pixmap, x: i32, y: i32, rgba: (u8, u8, u8, u8)) {
    let (width, height) = (pixmap.width() as i32, pixmap.height() as i32);
    let (r, g, b, a) = rgba;
    if a == 0 || x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let index = (y * width + x) as usize;
    let pixels = pixmap.pixels_mut();
    let dst = pixels[index];
    let a16 = u16::from(a);
    let inv = 255 - a16;
    let premul = |c: u8| (u16::from(c) * a16 + 127) / 255;
    let over = |src: u16, dst: u8| (src + (u16::from(dst) * inv + 127) / 255).min(255) as u8;
    let out_a = over(a16, dst.alpha());
    let out_r = over(premul(r), dst.red()).min(out_a);
    let out_g = over(premul(g), dst.green()).min(out_a);
    let out_b = over(premul(b), dst.blue()).min(out_a);
    if let Some(color) = tiny_skia::PremultipliedColorU8::from_rgba(out_r, out_g, out_b, out_a) {
        pixels[index] = color;
    }
}
