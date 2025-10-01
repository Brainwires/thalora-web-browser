# Font System Documentation

## Overview

The GUI browser includes a comprehensive font management system that handles:

- **CSS Font Properties**: Full support for font-family, font-size, font-weight, font-style
- **Multiple Units**: px, pt, em, rem, %, and keyword values (small, medium, large, etc.)
- **Font Families**: Support for serif, sans-serif, and monospace with automatic fallbacks
- **Font Weights**: 100-900 numeric weights and keyword values (thin, light, normal, bold, etc.)
- **Dynamic Font Loading**: Infrastructure for loading custom fonts and web fonts

## Components

### FontManager (`src/gui/fonts.rs`)

The `FontManager` handles font registration, loading, and configuration:

```rust
let mut font_manager = FontManager::new();
font_manager.install_fonts(ctx); // Install into egui context
```

### FontDescriptor

Represents a complete font specification with family, weight, style, and size:

```rust
let descriptor = FontDescriptor::new("sans-serif".to_string(), 16.0)
    .with_weight(FontWeight::Bold)
    .with_style(FontStyle::Italic);
```

### FontWeight

Enum representing CSS font-weight values from 100 (Thin) to 900 (Black):

- `FontWeight::from_css("bold")` - Parse CSS weight values
- `weight.is_bold()` - Check if weight >= 600

### FontSize

Utility for parsing CSS font-size values:

```rust
// Parse various CSS units
FontSize::parse_css("16px", base_size)    // Pixels
FontSize::parse_css("1.5em", base_size)   // Relative to parent
FontSize::parse_css("1rem", base_size)    // Relative to root (16px)
FontSize::parse_css("large", base_size)   // Keyword sizes
FontSize::parse_css("150%", base_size)    // Percentage

// Heading sizes
FontSize::heading_size(1)  // 32px for h1
FontSize::heading_size(2)  // 24px for h2
// etc.
```

## CSS Support

### Supported Properties

- `font-family`: Comma-separated font families with fallbacks
- `font-size`: All CSS units (px, pt, em, rem, %) and keywords
- `font-weight`: Numeric (100-900) and keywords (normal, bold, etc.)
- `font-style`: normal, italic, oblique
- `color`: CSS colors for text
- `background-color`: Background for text elements

### Font Family Mapping

The system automatically maps common font families:

| CSS Family | Egui Family |
|-----------|-------------|
| serif, times, georgia | Proportional |
| sans-serif, arial, helvetica | Proportional |
| monospace, courier, consolas | Monospace |

### Examples

```html
<!-- Heading with custom font -->
<h1 style="font-family: serif; font-size: 48px; font-weight: 800;">
  Large Serif Heading
</h1>

<!-- Paragraph with multiple styles -->
<p style="font-family: 'Helvetica Neue', Arial, sans-serif; font-size: 16px; color: #333;">
  Body text with fallback fonts
</p>

<!-- Monospace code -->
<code style="font-family: monospace; font-size: 14px; background-color: #f5f5f5;">
  Code snippet
</code>
```

## Integration with Browser UI

The `BrowserUI` integrates the font system automatically:

1. **Initialization**: Fonts are installed once when the renderer starts
2. **CSS Parsing**: Inline styles are parsed and applied to rendered elements
3. **Text Rendering**: All text uses `create_styled_text()` for consistent styling

### Internal Methods

- `init_fonts(ctx)` - Install fonts into egui context
- `create_styled_text(text, css, default_size)` - Create RichText with full styling
- `parse_inline_style(style)` - Parse CSS inline styles

## Future Enhancements

### Web Fonts (@font-face)

Future support for loading custom fonts from URLs:

```rust
font_manager.load_font_from_url(
    "CustomFont".to_string(),
    "https://example.com/fonts/custom.woff2"
).await?;
```

### Font Rendering Optimizations

- Font atlasing for better performance
- Subpixel rendering
- Kerning and ligature support
- Variable font support

### Advanced Typography

- Line height control
- Letter spacing (tracking)
- Text decoration (underline, strikethrough)
- Text transform (uppercase, lowercase, capitalize)

## Testing

Use the included test file to verify font rendering:

```bash
# Build and run the GUI browser
cargo build --release
./target/release/thalora browser

# Open test file in browser
file:///path/to/test_fonts.html
```

The test file demonstrates:
- Different heading levels with proper sizing
- Font family variations (serif, sans-serif, monospace)
- Font weights and styles
- Inline style combinations
- Styled containers with backgrounds

## Performance Considerations

- Fonts are loaded once and cached
- Font descriptors are created on-demand
- egui handles font atlasing and caching internally
- Minimal overhead for styled text rendering

## Browser Compatibility

The font system aims for compatibility with standard browser rendering:

| Feature | Support Level |
|---------|--------------|
| Basic font properties | ✅ Full |
| Font weight 100-900 | ✅ Full |
| CSS units (px, em, rem, %) | ✅ Full |
| Keyword sizes | ✅ Full |
| Font fallbacks | ✅ Full |
| System fonts | ✅ Limited to egui defaults |
| Web fonts | 🚧 Planned |
| Variable fonts | 🚧 Future |
