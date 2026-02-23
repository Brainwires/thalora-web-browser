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

    internal static readonly FontFamily BundledNotoSans =
        new("fonts:ThaloraBrowser#Noto Sans");
    internal static readonly FontFamily BundledNotoSerif =
        new("fonts:ThaloraBrowser#Noto Serif");
    internal static readonly FontFamily BundledFiraMono =
        new("fonts:ThaloraBrowser#Fira Mono");

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
            _ => ParseLength(value, parentFontSize) ?? parentFontSize,
        };

        // Avalonia crashes with ArgumentOutOfRangeException on FontSize = 0.
        // CSS font-size:0 is legal but useless; clamp to minimum 1px.
        return Math.Max(result, 1);
    }

    /// <summary>
    /// Parse a CSS color string to an Avalonia Color.
    /// Handles hex (#rgb, #rrggbb, #rrggbbaa), rgb(), rgba(), and named colors.
    /// </summary>
    internal static Color? ParseColor(string? value)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "transparent" || value == "none"
            || value == "inherit" || value == "initial" || value == "unset" || value == "currentColor")
            return null;

        value = value.Trim();

        // Skip unresolved CSS variables — they'll default to theme color
        if (value.Contains("var("))
            return null;

        // Try Avalonia's built-in parser first (handles hex and named colors)
        if (Color.TryParse(value, out var color))
            return color;

        // Handle rgb()/rgba() — both legacy (comma-separated) and modern (space-separated) syntax
        if (value.StartsWith("rgb"))
        {
            var inner = value;
            inner = inner.Replace("rgba(", "").Replace("rgb(", "").Replace(")", "").Trim();

            // Modern CSS syntax: "R G B" or "R G B / A"
            if (!inner.Contains(',') && inner.Contains(' '))
            {
                // Split on "/" for alpha
                var alphaParts = inner.Split('/');
                var channelStr = alphaParts[0].Trim();
                var channels = channelStr.Split(' ', StringSplitOptions.RemoveEmptyEntries);

                if (channels.Length >= 3
                    && TryParseColorChannel(channels[0], out var r)
                    && TryParseColorChannel(channels[1], out var g)
                    && TryParseColorChannel(channels[2], out var b))
                {
                    byte a = 255;
                    if (alphaParts.Length > 1 && double.TryParse(alphaParts[1].Trim(),
                        NumberStyles.Float, CultureInfo.InvariantCulture, out var alpha))
                    {
                        if (alpha <= 1.0)
                            a = (byte)(Math.Clamp(alpha, 0, 1) * 255);
                        else
                            a = (byte)Math.Clamp(alpha, 0, 255);
                    }
                    return Color.FromArgb(a, r, g, b);
                }
            }

            // Legacy syntax: "R, G, B" or "R, G, B, A"
            var parts = inner.Split(',', StringSplitOptions.TrimEntries);
            if (parts.Length >= 3
                && TryParseColorChannel(parts[0], out var cr)
                && TryParseColorChannel(parts[1], out var cg)
                && TryParseColorChannel(parts[2], out var cb))
            {
                byte ca = 255;
                if (parts.Length >= 4 && double.TryParse(parts[3],
                    NumberStyles.Float, CultureInfo.InvariantCulture, out var alpha))
                {
                    if (alpha <= 1.0)
                        ca = (byte)(Math.Clamp(alpha, 0, 1) * 255);
                    else
                        ca = (byte)Math.Clamp(alpha, 0, 255);
                }
                return Color.FromArgb(ca, cr, cg, cb);
            }
        }

        // Handle CSS color() function — e.g., "color(#738a94 l(-25%))" or "color(#2da7cb lightness(-4%))"
        // This is a CSS Color Level 4 relative color function used by sites like Cloudflare blog.
        if (value.StartsWith("color(") && value.EndsWith(")"))
        {
            var inner = value.Substring(6, value.Length - 7).Trim();
            // Split into base color and modifiers
            // Pattern: "#hex modifier(value) modifier(value)..."
            var firstSpace = inner.IndexOf(' ');
            if (firstSpace > 0)
            {
                var baseColorStr = inner.Substring(0, firstSpace).Trim();
                var modifiers = inner.Substring(firstSpace).Trim();

                var baseColor = ParseColor(baseColorStr);
                if (baseColor.HasValue)
                {
                    // Convert to HSL for lightness/whiteness modifications
                    var c = baseColor.Value;
                    double r = c.R / 255.0, g = c.G / 255.0, b = c.B / 255.0;
                    double max = Math.Max(r, Math.Max(g, b));
                    double min = Math.Min(r, Math.Min(g, b));
                    double hue = 0, sat = 0, lit = (max + min) / 2.0;

                    if (max != min)
                    {
                        double d = max - min;
                        sat = lit > 0.5 ? d / (2.0 - max - min) : d / (max + min);
                        if (max == r) hue = (g - b) / d + (g < b ? 6 : 0);
                        else if (max == g) hue = (b - r) / d + 2;
                        else hue = (r - g) / d + 4;
                        hue *= 60;
                    }

                    // Apply modifiers: l(-25%), lightness(-4%), whiteness(7%)
                    foreach (var mod in modifiers.Split(' ', StringSplitOptions.RemoveEmptyEntries))
                    {
                        var m = mod.Trim();
                        // Parse "l(value%)" or "lightness(value%)"
                        string? percentStr = null;
                        if (m.StartsWith("l(") && m.EndsWith(")") && m.Length > 3)
                            percentStr = m.Substring(2, m.Length - 3).TrimEnd('%');
                        else if (m.StartsWith("lightness(") && m.EndsWith(")") && m.Length > 11)
                            percentStr = m.Substring(10, m.Length - 11).TrimEnd('%');
                        else if (m.StartsWith("whiteness(") && m.EndsWith(")") && m.Length > 11)
                            percentStr = m.Substring(10, m.Length - 11).TrimEnd('%');

                        if (percentStr != null && double.TryParse(percentStr,
                            NumberStyles.Float, CultureInfo.InvariantCulture, out var pct))
                        {
                            // Apply as relative adjustment to lightness
                            lit = Math.Clamp(lit + pct / 100.0, 0, 1);
                        }
                    }

                    var (rr, gg, bb) = HslToRgb(hue, sat, lit);
                    return Color.FromArgb(c.A, (byte)rr, (byte)gg, (byte)bb);
                }
            }
            else
            {
                // No modifiers, just "color(#hex)" — parse as is
                return ParseColor(inner);
            }
        }

        // Handle hsl()/hsla()
        if (value.StartsWith("hsl"))
        {
            var inner = value.Replace("hsla(", "").Replace("hsl(", "").Replace(")", "").Trim();
            // Try both comma and space separators
            var parts = inner.Contains(',')
                ? inner.Split(',', StringSplitOptions.TrimEntries)
                : inner.Split(' ', StringSplitOptions.RemoveEmptyEntries);

            if (parts.Length >= 3
                && double.TryParse(parts[0].TrimEnd('d', 'e', 'g'), NumberStyles.Float, CultureInfo.InvariantCulture, out var h)
                && double.TryParse(parts[1].TrimEnd('%'), NumberStyles.Float, CultureInfo.InvariantCulture, out var s)
                && double.TryParse(parts[2].TrimEnd('%').Split('/')[0].Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out var l))
            {
                // Convert HSL to RGB
                s /= 100.0; l /= 100.0;
                var (rr, gg, bb) = HslToRgb(h, s, l);
                byte alpha = 255;
                // Check for alpha after "/" or as 4th comma-separated value
                if (parts.Length >= 4 && double.TryParse(parts[3].TrimEnd('%'),
                    NumberStyles.Float, CultureInfo.InvariantCulture, out var a))
                {
                    alpha = a <= 1.0 ? (byte)(Math.Clamp(a, 0, 1) * 255) : (byte)Math.Clamp(a, 0, 255);
                }
                return Color.FromArgb(alpha, (byte)rr, (byte)gg, (byte)bb);
            }
        }

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
    /// </summary>
    internal static FontWeight ParseFontWeight(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return FontWeight.Normal;

        return value.Trim().ToLowerInvariant() switch
        {
            "normal" => FontWeight.Normal,
            "bold" => FontWeight.Bold,
            "bolder" => FontWeight.SemiBold,
            "lighter" => FontWeight.Light,
            "100" => FontWeight.Thin,
            "200" => FontWeight.ExtraLight,
            "300" => FontWeight.Light,
            "400" => FontWeight.Normal,
            "500" => FontWeight.Medium,
            "600" => FontWeight.SemiBold,
            "700" => FontWeight.Bold,
            "800" => FontWeight.ExtraBold,
            "900" => FontWeight.Black,
            _ => FontWeight.Normal,
        };
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
    /// Map a CSS font-family string to a bundled Avalonia FontFamily.
    /// Walks the comma-separated font stack and matches known font names.
    /// </summary>
    internal static FontFamily MapToBundledFontFamily(string? cssFontFamily)
    {
        if (string.IsNullOrWhiteSpace(cssFontFamily))
            return BundledNotoSans;

        foreach (var name in cssFontFamily.Split(','))
        {
            var trimmed = name.Trim().Trim('"', '\'').ToLowerInvariant();

            switch (trimmed)
            {
                // Monospace
                case "monospace":
                case "fira mono":
                case "fira code":
                case "courier new":
                case "courier":
                case "consolas":
                case "menlo":
                case "monaco":
                case "source code pro":
                case "jetbrains mono":
                case "sf mono":
                case "ubuntu mono":
                case "dejavu sans mono":
                case "liberation mono":
                case "lucida console":
                case "ui-monospace":
                    return BundledFiraMono;

                // Serif
                case "serif":
                case "noto serif":
                case "times new roman":
                case "times":
                case "georgia":
                case "palatino":
                case "palatino linotype":
                case "book antiqua":
                case "garamond":
                case "cambria":
                case "dejavu serif":
                case "liberation serif":
                case "ui-serif":
                case "cursive":
                case "fantasy":
                    return BundledNotoSerif;

                // Sans-serif (most common on the web)
                case "sans-serif":
                case "noto sans":
                case "arial":
                case "helvetica":
                case "helvetica neue":
                case "segoe ui":
                case "open sans":
                case "roboto":
                case "lato":
                case "inter":
                case "source sans pro":
                case "source sans 3":
                case "ubuntu":
                case "nunito":
                case "poppins":
                case "montserrat":
                case "raleway":
                case "pt sans":
                case "verdana":
                case "tahoma":
                case "trebuchet ms":
                case "lucida grande":
                case "lucida sans":
                case "dejavu sans":
                case "liberation sans":
                case "gill sans":
                case "franklin gothic medium":
                case "-apple-system":
                case "system-ui":
                case "blinkmacsystemfont":
                case "ui-sans-serif":
                case "ui-rounded":
                case "math":
                case "emoji":
                    return BundledNotoSans;
            }
        }

        return BundledNotoSans;
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

    /// <summary>
    /// Try to parse a color channel value (handles both integer 0-255 and percentage 0%-100%).
    /// </summary>
    private static bool TryParseColorChannel(string value, out byte result)
    {
        result = 0;
        value = value.Trim();

        if (value.EndsWith('%'))
        {
            if (double.TryParse(value.AsSpan(0, value.Length - 1),
                NumberStyles.Float, CultureInfo.InvariantCulture, out var pct))
            {
                result = (byte)Math.Clamp(pct / 100.0 * 255, 0, 255);
                return true;
            }
            return false;
        }

        if (double.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out var num))
        {
            result = (byte)Math.Clamp(num, 0, 255);
            return true;
        }

        return false;
    }

    /// <summary>
    /// Convert HSL values to RGB (r, g, b as 0-255).
    /// </summary>
    private static (int r, int g, int b) HslToRgb(double h, double s, double l)
    {
        h = ((h % 360) + 360) % 360;
        double c = (1 - Math.Abs(2 * l - 1)) * s;
        double x = c * (1 - Math.Abs((h / 60.0) % 2 - 1));
        double m = l - c / 2;

        double r1 = 0, g1 = 0, b1 = 0;
        if (h < 60) { r1 = c; g1 = x; }
        else if (h < 120) { r1 = x; g1 = c; }
        else if (h < 180) { g1 = c; b1 = x; }
        else if (h < 240) { g1 = x; b1 = c; }
        else if (h < 300) { r1 = x; b1 = c; }
        else { r1 = c; b1 = x; }

        return (
            (int)Math.Clamp((r1 + m) * 255, 0, 255),
            (int)Math.Clamp((g1 + m) * 255, 0, 255),
            (int)Math.Clamp((b1 + m) * 255, 0, 255)
        );
    }
}
