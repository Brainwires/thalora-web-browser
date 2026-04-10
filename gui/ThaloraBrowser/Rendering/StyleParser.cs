using System.Globalization;
using Avalonia;
using Avalonia.Media;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Parses CSS string values into Avalonia types.
/// Used by ControlTreeBuilder to convert ResolvedStyles from Rust into
/// Avalonia control properties.
/// </summary>
internal static class StyleParser
{
    // --- Bundled font families ---
    // Registered via EmbeddedFontCollection in Program.cs with key URI "fonts:ThaloraBrowser".
    // Font files are in src/gui/fonts/ and embedded as AvaloniaResource.
    // Rust's map_to_bundled_font() resolves CSS font stacks to one of these three names.

    internal static readonly FontFamily BundledNotoSans =
        new("fonts:ThaloraBrowser#Noto Sans");
    internal static readonly FontFamily BundledNotoSerif =
        new("fonts:ThaloraBrowser#Noto Serif");
    internal static readonly FontFamily BundledFiraMono =
        new("fonts:ThaloraBrowser#Fira Mono");

    /// <summary>
    /// Resolve a bundled font name (as emitted by Rust's map_to_bundled_font()) to an
    /// Avalonia FontFamily.  Rust outputs one of "NotoSans", "NotoSerif", "FiraMono".
    /// Anything unrecognised defaults to NotoSans.
    /// </summary>
    internal static FontFamily ResolveFontFamily(string? name) => name?.Trim() switch
    {
        "NotoSerif" => BundledNotoSerif,
        "FiraMono" => BundledFiraMono,
        _ => BundledNotoSans,
    };

    /// <summary>
    /// Parse a CSS length string (e.g., "16px", "1.5em", "50%", "auto") to pixels.
    /// Returns null for "auto" or unparseable values.
    /// parentFontSize is used for em/rem resolution.
    /// parentSize is used for percentage resolution.
    /// viewportWidth/viewportHeight are used for vw/vh resolution.
    /// </summary>
    internal static double? ParseLength(string? value, double parentFontSize = 16, double parentSize = 0,
        double viewportWidth = 0, double viewportHeight = 0)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "auto" || value == "none")
            return null;

        value = value.Trim();

        if (value.EndsWith("px"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var px))
                return px;
        }
        else if (value.EndsWith("rem"))
        {
            // MUST check "rem" before "em" since "rem" also EndsWith("em")
            if (double.TryParse(value.AsSpan(0, value.Length - 3),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var rem))
                return rem * 16; // Root font size is always 16px
        }
        else if (value.EndsWith("em"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var em))
                return em * parentFontSize;
        }
        else if (value.EndsWith("%"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 1),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var pct))
                return pct / 100.0 * parentSize;
        }
        else if (value.EndsWith("vw"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var vw))
                return vw / 100.0 * viewportWidth;
        }
        else if (value.EndsWith("vh"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var vh))
                return vh / 100.0 * viewportHeight;
        }
        else if (value.EndsWith("pt"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var pt))
                return pt * 4.0 / 3.0; // 1pt = 4/3 px
        }
        else if (value == "0")
        {
            return 0;
        }
        else
        {
            // Try bare number
            if (double.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out var bare))
                return bare;
        }

        return null;
    }

    /// <summary>
    /// Parse a CSS font-size string and return pixels.
    /// Handles named sizes (small, medium, large, etc.) and relative units.
    /// </summary>
    internal static double ParseFontSize(string? value, double parentFontSize = 16)
    {
        if (string.IsNullOrWhiteSpace(value))
            return parentFontSize;

        // Named sizes
        var result = value.Trim().ToLowerInvariant() switch
        {
            "xx-small" => 9,
            "x-small" => 10,
            "small" => 13,
            "medium" => 16,
            "large" => 18,
            "x-large" => 24,
            "xx-large" => 32,
            "xxx-large" => 48,
            "smaller" => parentFontSize * 0.833,
            "larger" => parentFontSize * 1.2,
            // For font-size, percentage values are relative to the parent's font size.
            // Pass parentFontSize as both the em base AND the parentSize (for % resolution).
            _ => ParseLength(value, parentFontSize, parentSize: parentFontSize) ?? parentFontSize,
        };

        // Avalonia crashes with ArgumentOutOfRangeException on FontSize = 0.
        // CSS font-size:0 is legal but useless; clamp to minimum 1px.
        return Math.Max(result, 1);
    }

    /// <summary>
    /// Parse a CSS color string to an Avalonia Color.
    /// Rust pre-normalizes hex and rgb/hsl colors to #rrggbb / #rrggbbaa before
    /// serialization, so this path only needs to handle hex and named colors.
    /// The color() CSS Level-4 function (rare, e.g. Cloudflare blog) falls through
    /// to Avalonia's parser which may handle it, or returns null as a safe default.
    /// </summary>
    internal static Color? ParseColor(string? value)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "transparent" || value == "none"
            || value == "inherit" || value == "initial" || value == "unset" || value == "currentColor")
            return null;

        // Skip unresolved CSS variables — they'll default to theme color
        if (value.Contains("var("))
            return null;

        // Avalonia handles hex (#rrggbb, #rrggbbaa, #rgb) and CSS named colors natively.
        if (Color.TryParse(value.Trim(), out var color))
            return color;

        return null;
    }

    /// <summary>
    /// Parse a CSS color string to an Avalonia IBrush.
    /// Returns null for "transparent" or unparseable values.
    /// </summary>
    internal static IBrush? ParseBrush(string? value)
    {
        var color = ParseColor(value);
        return color.HasValue ? new SolidColorBrush(color.Value) : null;
    }

    /// <summary>
    /// Parse a CSS font-weight string to Avalonia FontWeight.
    /// Rust normalizes keywords ("bold" → "700", "normal" → "400") before
    /// serialization, so only numeric strings are expected here.
    /// </summary>
    internal static FontWeight ParseFontWeight(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return FontWeight.Normal;

        if (ushort.TryParse(value.Trim(), out var w))
            return (FontWeight)w;

        return FontWeight.Normal;
    }

    /// <summary>
    /// Parse a CSS font-style string to Avalonia FontStyle.
    /// </summary>
    internal static FontStyle ParseFontStyle(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return FontStyle.Normal;

        return value.Trim().ToLowerInvariant() switch
        {
            "italic" => FontStyle.Italic,
            "oblique" => FontStyle.Oblique,
            _ => FontStyle.Normal,
        };
    }

    /// <summary>
    /// Parse a CSS text-align string to Avalonia TextAlignment.
    /// </summary>
    internal static TextAlignment ParseTextAlignment(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return TextAlignment.Left;

        return value.Trim().ToLowerInvariant() switch
        {
            "center" => TextAlignment.Center,
            "right" => TextAlignment.Right,
            "justify" => TextAlignment.Justify,
            "start" => TextAlignment.Left,
            "end" => TextAlignment.Right,
            _ => TextAlignment.Left,
        };
    }

    /// <summary>
    /// Parse a CSS border-radius string to an Avalonia CornerRadius.
    /// Supports single value ("4px") and shorthand ("4px 8px").
    /// </summary>
    internal static CornerRadius ParseBorderRadius(string? value, double parentFontSize = 16, double parentSize = 0)
    {
        if (string.IsNullOrWhiteSpace(value))
            return default;

        var parts = value.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (parts.Length == 1)
        {
            // Percentage border-radius: use parentSize if available, else use large value
            // that Avalonia will clamp to half the element dimension
            if (parts[0].TrimEnd().EndsWith("%"))
            {
                if (parentSize > 0)
                {
                    var r = ParseLength(parts[0], parentFontSize, parentSize) ?? 0;
                    return new CornerRadius(r);
                }
                // No parent size known — use large value for circular/pill shape
                return new CornerRadius(9999);
            }
            var r2 = ParseLength(parts[0], parentFontSize) ?? 0;
            return new CornerRadius(r2);
        }
        if (parts.Length == 2)
        {
            var tl = ParseLength(parts[0], parentFontSize) ?? 0;
            var tr = ParseLength(parts[1], parentFontSize) ?? 0;
            return new CornerRadius(tl, tr, tl, tr);
        }
        if (parts.Length == 4)
        {
            var tl = ParseLength(parts[0], parentFontSize) ?? 0;
            var tr = ParseLength(parts[1], parentFontSize) ?? 0;
            var br = ParseLength(parts[2], parentFontSize) ?? 0;
            var bl = ParseLength(parts[3], parentFontSize) ?? 0;
            return new CornerRadius(tl, tr, br, bl);
        }

        return default;
    }

    /// <summary>
    /// Parse a StyleBoxSides (margin/padding) into an Avalonia Thickness.
    /// </summary>
    internal static Thickness ParseBoxSides(StyleBoxSides? sides, double parentFontSize = 16, double parentSize = 0,
        double viewportWidth = 0, double viewportHeight = 0)
    {
        if (sides == null)
            return default;

        var top = ParseLength(sides.Top, parentFontSize, parentSize, viewportWidth, viewportHeight) ?? 0;
        var right = ParseLength(sides.Right, parentFontSize, parentSize, viewportWidth, viewportHeight) ?? 0;
        var bottom = ParseLength(sides.Bottom, parentFontSize, parentSize, viewportWidth, viewportHeight) ?? 0;
        var left = ParseLength(sides.Left, parentFontSize, parentSize, viewportWidth, viewportHeight) ?? 0;

        return new Thickness(left, top, right, bottom);
    }

    /// <summary>
    /// Check if a margin side value is "auto" (for centering).
    /// </summary>
    internal static bool IsAutoMargin(string? value)
    {
        return string.Equals(value?.Trim(), "auto", StringComparison.OrdinalIgnoreCase);
    }

    /// <summary>
    /// Parse a CSS line-height value.
    /// Returns a multiplier (e.g., 1.5 for "1.5" or "150%").
    /// </summary>
    internal static double ParseLineHeight(string? value, double fontSize = 16)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "normal")
            return 1.4; // default

        value = value.Trim();

        if (value.EndsWith("px"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 2),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var px))
                return px / fontSize;
        }
        else if (value.EndsWith("%"))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 1),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var pct))
                return pct / 100.0;
        }
        else if (double.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out var multiplier))
        {
            return multiplier;
        }

        return 1.4;
    }

}
