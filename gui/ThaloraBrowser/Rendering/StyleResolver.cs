using Avalonia;
using Avalonia.Media;
using FontWeight = Avalonia.Media.FontWeight;
using FontStyle = Avalonia.Media.FontStyle;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Static parsing helpers for converting CSS string values (from Rust JSON)
/// into Avalonia types. No longer performs CSS computation — that's done on the Rust side.
/// </summary>
public class StyleResolver
{
    // Viewport size for resolving vh/vw units
    public static Size Viewport { get; set; } = new Size(1024, 768);

    // --- Parsing helpers (used by HtmlRenderer when converting Rust JSON values) ---

    internal static DisplayMode ParseDisplay(string value) => value.Trim().ToLowerInvariant() switch
    {
        "block" => DisplayMode.Block,
        "inline" => DisplayMode.Inline,
        "inline-block" => DisplayMode.InlineBlock,
        "none" => DisplayMode.None,
        "flex" => DisplayMode.Flex,
        "list-item" => DisplayMode.ListItem,
        "table" => DisplayMode.Table,
        "table-row" => DisplayMode.TableRow,
        "table-cell" => DisplayMode.TableCell,
        _ => DisplayMode.Block,
    };

    internal static FontWeight ParseFontWeight(string value) => value.Trim().ToLowerInvariant() switch
    {
        "bold" => FontWeight.Bold,
        "bolder" => FontWeight.Bold,
        "lighter" => FontWeight.Light,
        "normal" => FontWeight.Normal,
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

    internal static TextAlignment ParseTextAlign(string value) => value.Trim().ToLowerInvariant() switch
    {
        "left" => TextAlignment.Left,
        "right" => TextAlignment.Right,
        "center" => TextAlignment.Center,
        "justify" => TextAlignment.Justify,
        _ => TextAlignment.Left,
    };

    internal static WhiteSpaceMode ParseWhiteSpace(string value) => value.Trim().ToLowerInvariant() switch
    {
        "normal" => WhiteSpaceMode.Normal,
        "nowrap" => WhiteSpaceMode.NoWrap,
        "pre" => WhiteSpaceMode.Pre,
        "pre-wrap" => WhiteSpaceMode.PreWrap,
        "pre-line" => WhiteSpaceMode.PreLine,
        _ => WhiteSpaceMode.Normal,
    };

    internal static OverflowMode ParseOverflow(string value) => value.Trim().ToLowerInvariant() switch
    {
        "visible" => OverflowMode.Visible,
        "hidden" => OverflowMode.Hidden,
        "scroll" => OverflowMode.Scroll,
        "auto" => OverflowMode.Auto,
        _ => OverflowMode.Visible,
    };

    public static double? ParseLength(string value, double parentFontSize)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "auto" || value == "none")
            return null;

        value = value.Trim().ToLowerInvariant();

        if (value.EndsWith("px"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var px))
                return px;
        }
        else if (value.EndsWith("rem"))
        {
            if (double.TryParse(value[..^3], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var rem))
                return rem * 16; // root font size is 16px
        }
        else if (value.EndsWith("em"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var em))
                return em * parentFontSize;
        }
        else if (value.EndsWith('%'))
        {
            if (double.TryParse(value[..^1], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var pct))
                return pct / 100.0 * parentFontSize;
        }
        else if (value.EndsWith("pt"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var pt))
                return pt * 4.0 / 3.0;
        }
        else if (value.EndsWith("vh"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var v))
                return v / 100.0 * Viewport.Height;
        }
        else if (value.EndsWith("vw"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var v))
                return v / 100.0 * Viewport.Width;
        }
        else if (double.TryParse(value, System.Globalization.NumberStyles.Float,
            System.Globalization.CultureInfo.InvariantCulture, out var raw))
        {
            return raw;
        }

        return null;
    }

    public static Color? ParseColor(string value)
    {
        if (string.IsNullOrWhiteSpace(value) || value == "transparent")
            return null;

        value = value.Trim();

        // Try Avalonia's built-in color parser first
        if (Avalonia.Media.Color.TryParse(value, out var color))
            return color;

        // Handle rgb()/rgba()
        if (value.StartsWith("rgb"))
        {
            var inner = value.Substring(value.IndexOf('(') + 1).TrimEnd(')');
            var parts = inner.Split(',').Select(p => p.Trim()).ToArray();

            if (parts.Length >= 3)
            {
                if (byte.TryParse(parts[0], out var r) &&
                    byte.TryParse(parts[1], out var g) &&
                    byte.TryParse(parts[2], out var b))
                {
                    byte a = 255;
                    if (parts.Length >= 4 && double.TryParse(parts[3],
                        System.Globalization.NumberStyles.Float,
                        System.Globalization.CultureInfo.InvariantCulture, out var alpha))
                        a = (byte)(Math.Clamp(alpha, 0, 1) * 255);

                    return Avalonia.Media.Color.FromArgb(a, r, g, b);
                }
            }
        }

        return null;
    }
}
