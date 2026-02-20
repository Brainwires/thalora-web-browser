using AngleSharp.Css.Dom;
using AngleSharp.Dom;
using Avalonia;
using Avalonia.Media;
using FontWeight = Avalonia.Media.FontWeight;
using FontStyle = Avalonia.Media.FontStyle;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Resolves CSS computed styles from AngleSharp's CSS engine into
/// our CssComputedStyle records that the layout and paint engines understand.
/// </summary>
public class StyleResolver
{
    // Default styles for common HTML elements
    private static readonly Dictionary<string, Action<CssComputedStyle>> ElementDefaults = new(StringComparer.OrdinalIgnoreCase)
    {
        ["h1"] = s => { s.FontSize = 32; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 21.44, 0, 21.44); s.Display = DisplayMode.Block; },
        ["h2"] = s => { s.FontSize = 24; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 19.92, 0, 19.92); s.Display = DisplayMode.Block; },
        ["h3"] = s => { s.FontSize = 18.72; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 18.72, 0, 18.72); s.Display = DisplayMode.Block; },
        ["h4"] = s => { s.FontSize = 16; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 21.28, 0, 21.28); s.Display = DisplayMode.Block; },
        ["h5"] = s => { s.FontSize = 13.28; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 22.18, 0, 22.18); s.Display = DisplayMode.Block; },
        ["h6"] = s => { s.FontSize = 10.72; s.FontWeight = FontWeight.Bold; s.Margin = new Thickness(0, 24.97, 0, 24.97); s.Display = DisplayMode.Block; },
        ["p"] = s => { s.Margin = new Thickness(0, 16, 0, 16); s.Display = DisplayMode.Block; },
        ["div"] = s => { s.Display = DisplayMode.Block; },
        ["section"] = s => { s.Display = DisplayMode.Block; },
        ["article"] = s => { s.Display = DisplayMode.Block; },
        ["header"] = s => { s.Display = DisplayMode.Block; },
        ["footer"] = s => { s.Display = DisplayMode.Block; },
        ["main"] = s => { s.Display = DisplayMode.Block; },
        ["nav"] = s => { s.Display = DisplayMode.Block; },
        ["aside"] = s => { s.Display = DisplayMode.Block; },
        ["blockquote"] = s => { s.Margin = new Thickness(40, 16, 40, 16); s.Display = DisplayMode.Block; },
        ["pre"] = s => { s.FontFamily = new FontFamily("monospace"); s.WhiteSpace = WhiteSpaceMode.Pre; s.Margin = new Thickness(0, 16, 0, 16); s.Display = DisplayMode.Block; },
        ["code"] = s => { s.FontFamily = new FontFamily("monospace"); s.BackgroundColor = new SolidColorBrush(Color.FromArgb(40, 255, 255, 255)); },
        ["hr"] = s => { s.Display = DisplayMode.Block; s.Margin = new Thickness(0, 8, 0, 8); s.BorderWidth = new Thickness(0, 0, 0, 1); s.BorderBrush = Brushes.Gray; },
        ["ul"] = s => { s.Display = DisplayMode.Block; s.Margin = new Thickness(0, 16, 0, 16); s.Padding = new Thickness(40, 0, 0, 0); },
        ["ol"] = s => { s.Display = DisplayMode.Block; s.Margin = new Thickness(0, 16, 0, 16); s.Padding = new Thickness(40, 0, 0, 0); },
        ["li"] = s => { s.Display = DisplayMode.ListItem; s.ListStyleType = ListStyleType.Disc; },
        ["a"] = s => { s.Color = new SolidColorBrush(Color.FromRgb(100, 149, 237)); s.TextDecorations = Avalonia.Media.TextDecorations.Underline; },
        ["strong"] = s => { s.FontWeight = FontWeight.Bold; },
        ["b"] = s => { s.FontWeight = FontWeight.Bold; },
        ["em"] = s => { s.FontStyle = FontStyle.Italic; },
        ["i"] = s => { s.FontStyle = FontStyle.Italic; },
        ["span"] = s => { s.Display = DisplayMode.Inline; },
        ["img"] = s => { s.Display = DisplayMode.InlineBlock; },
        ["br"] = s => { s.Display = DisplayMode.Inline; },
        ["table"] = s => { s.Display = DisplayMode.Table; s.Margin = new Thickness(0, 16, 0, 16); s.BorderWidth = new Thickness(1); s.BorderBrush = Brushes.Gray; },
        ["tr"] = s => { s.Display = DisplayMode.TableRow; },
        ["td"] = s => { s.Display = DisplayMode.TableCell; s.Padding = new Thickness(8, 4, 8, 4); s.BorderWidth = new Thickness(1); s.BorderBrush = new SolidColorBrush(Color.FromArgb(80, 128, 128, 128)); },
        ["th"] = s => { s.Display = DisplayMode.TableCell; s.Padding = new Thickness(8, 4, 8, 4); s.FontWeight = FontWeight.Bold; s.BorderWidth = new Thickness(1); s.BorderBrush = new SolidColorBrush(Color.FromArgb(80, 128, 128, 128)); },
        ["form"] = s => { s.Display = DisplayMode.Block; },
        ["input"] = s => { s.Display = DisplayMode.InlineBlock; },
        ["button"] = s => { s.Display = DisplayMode.InlineBlock; s.Padding = new Thickness(8, 4, 8, 4); s.BackgroundColor = new SolidColorBrush(Color.FromRgb(60, 60, 60)); s.BorderWidth = new Thickness(1); s.BorderBrush = Brushes.Gray; },
        ["textarea"] = s => { s.Display = DisplayMode.InlineBlock; },
        ["select"] = s => { s.Display = DisplayMode.InlineBlock; },
        ["label"] = s => { s.Display = DisplayMode.Inline; },
    };

    // Block-level elements
    private static readonly HashSet<string> BlockElements = new(StringComparer.OrdinalIgnoreCase)
    {
        "div", "p", "h1", "h2", "h3", "h4", "h5", "h6", "section", "article",
        "header", "footer", "main", "nav", "aside", "blockquote", "pre", "hr",
        "ul", "ol", "li", "table", "form", "figure", "figcaption", "details",
        "summary", "dialog", "address", "fieldset", "legend",
    };

    /// <summary>
    /// Compute the style for a DOM element, considering element defaults,
    /// AngleSharp computed CSS, and inherited properties from the parent.
    /// </summary>
    public CssComputedStyle ComputeStyle(IElement element, CssComputedStyle? parentStyle)
    {
        var style = new CssComputedStyle();
        var tagName = element.LocalName.ToLowerInvariant();

        // 1. Apply inherited properties from parent
        if (parentStyle != null)
        {
            style.FontSize = parentStyle.FontSize;
            style.FontWeight = parentStyle.FontWeight;
            style.FontFamily = parentStyle.FontFamily;
            style.FontStyle = parentStyle.FontStyle;
            style.Color = parentStyle.Color;
            style.TextAlign = parentStyle.TextAlign;
            style.LineHeight = parentStyle.LineHeight;
            style.WhiteSpace = parentStyle.WhiteSpace;
        }

        // 2. Determine default display mode
        style.Display = BlockElements.Contains(tagName) ? DisplayMode.Block : DisplayMode.Inline;

        // 3. Apply element-specific defaults
        if (ElementDefaults.TryGetValue(tagName, out var defaults))
            defaults(style);

        // 4. Apply AngleSharp computed CSS (overrides defaults)
        ApplyComputedCss(element, style);

        return style;
    }

    /// <summary>
    /// Create a default style for the root/body element.
    /// </summary>
    public CssComputedStyle CreateRootStyle()
    {
        return new CssComputedStyle
        {
            FontSize = 16,
            FontWeight = FontWeight.Normal,
            FontFamily = FontFamily.Default,
            Color = new SolidColorBrush(Color.FromRgb(220, 220, 220)),
            BackgroundColor = new SolidColorBrush(Color.FromRgb(30, 30, 30)),
            Display = DisplayMode.Block,
            LineHeight = 1.4,
        };
    }

    private void ApplyComputedCss(IElement element, CssComputedStyle style)
    {
        var cssStyle = element.ComputeCurrentStyle();
        if (cssStyle == null) return;

        // Display
        var display = cssStyle.GetPropertyValue("display");
        if (!string.IsNullOrEmpty(display))
            style.Display = ParseDisplay(display);

        // Visibility
        var visibility = cssStyle.GetPropertyValue("visibility");
        if (visibility == "hidden" || visibility == "collapse")
            style.IsVisible = false;

        if (style.Display == DisplayMode.None)
            style.IsVisible = false;

        // Font size
        var fontSize = cssStyle.GetPropertyValue("font-size");
        if (!string.IsNullOrEmpty(fontSize))
        {
            var parsed = ParseLength(fontSize, style.FontSize);
            if (parsed.HasValue)
                style.FontSize = parsed.Value;
        }

        // Font weight
        var fontWeight = cssStyle.GetPropertyValue("font-weight");
        if (!string.IsNullOrEmpty(fontWeight))
            style.FontWeight = ParseFontWeight(fontWeight);

        // Font style
        var fontStyleVal = cssStyle.GetPropertyValue("font-style");
        if (fontStyleVal == "italic" || fontStyleVal == "oblique")
            style.FontStyle = FontStyle.Italic;

        // Font family
        var fontFamily = cssStyle.GetPropertyValue("font-family");
        if (!string.IsNullOrEmpty(fontFamily))
            style.FontFamily = new FontFamily(fontFamily.Split(',')[0].Trim().Trim('"', '\''));

        // Color
        var color = cssStyle.GetPropertyValue("color");
        if (!string.IsNullOrEmpty(color))
        {
            var parsed = ParseColor(color);
            if (parsed.HasValue)
                style.Color = new SolidColorBrush(parsed.Value);
        }

        // Background color
        var bgColor = cssStyle.GetPropertyValue("background-color");
        if (!string.IsNullOrEmpty(bgColor) && bgColor != "transparent" && bgColor != "rgba(0, 0, 0, 0)")
        {
            var parsed = ParseColor(bgColor);
            if (parsed.HasValue)
                style.BackgroundColor = new SolidColorBrush(parsed.Value);
        }

        // Text alignment
        var textAlign = cssStyle.GetPropertyValue("text-align");
        if (!string.IsNullOrEmpty(textAlign))
            style.TextAlign = ParseTextAlign(textAlign);

        // Text decoration
        var textDecoration = cssStyle.GetPropertyValue("text-decoration");
        if (!string.IsNullOrEmpty(textDecoration))
        {
            if (textDecoration.Contains("underline"))
                style.TextDecorations = Avalonia.Media.TextDecorations.Underline;
            else if (textDecoration.Contains("line-through"))
                style.TextDecorations = Avalonia.Media.TextDecorations.Strikethrough;
            else if (textDecoration == "none")
                style.TextDecorations = null;
        }

        // Line height
        var lineHeight = cssStyle.GetPropertyValue("line-height");
        if (!string.IsNullOrEmpty(lineHeight) && lineHeight != "normal")
        {
            if (double.TryParse(lineHeight, System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var lh))
                style.LineHeight = lh;
            else
            {
                var parsed = ParseLength(lineHeight, style.FontSize);
                if (parsed.HasValue)
                    style.LineHeight = parsed.Value / style.FontSize;
            }
        }

        // White space
        var whiteSpace = cssStyle.GetPropertyValue("white-space");
        if (!string.IsNullOrEmpty(whiteSpace))
            style.WhiteSpace = ParseWhiteSpace(whiteSpace);

        // Opacity
        var opacity = cssStyle.GetPropertyValue("opacity");
        if (!string.IsNullOrEmpty(opacity) && double.TryParse(opacity,
            System.Globalization.NumberStyles.Float,
            System.Globalization.CultureInfo.InvariantCulture, out var op))
            style.Opacity = Math.Clamp(op, 0, 1);

        // Box model: margin
        style.Margin = ParseBoxEdges(cssStyle, "margin", style.Margin, style.FontSize);

        // Box model: padding
        style.Padding = ParseBoxEdges(cssStyle, "padding", style.Padding, style.FontSize);

        // Box model: border-width
        style.BorderWidth = ParseBoxEdges(cssStyle, "border", style.BorderWidth, style.FontSize, isBorder: true);

        // Border color
        var borderColor = cssStyle.GetPropertyValue("border-color");
        if (!string.IsNullOrEmpty(borderColor))
        {
            var parsed = ParseColor(borderColor);
            if (parsed.HasValue)
                style.BorderBrush = new SolidColorBrush(parsed.Value);
        }

        // Border radius
        var borderRadius = cssStyle.GetPropertyValue("border-radius");
        if (!string.IsNullOrEmpty(borderRadius))
        {
            var parsed = ParseLength(borderRadius, style.FontSize);
            if (parsed.HasValue)
                style.BorderRadius = new CornerRadius(parsed.Value);
        }

        // Sizing
        var width = cssStyle.GetPropertyValue("width");
        if (!string.IsNullOrEmpty(width) && width != "auto")
        {
            style.Width = ParseLength(width, style.FontSize);
        }

        var height = cssStyle.GetPropertyValue("height");
        if (!string.IsNullOrEmpty(height) && height != "auto")
        {
            style.Height = ParseLength(height, style.FontSize);
        }

        var maxWidth = cssStyle.GetPropertyValue("max-width");
        if (!string.IsNullOrEmpty(maxWidth) && maxWidth != "none")
        {
            style.MaxWidth = ParseLength(maxWidth, style.FontSize);
        }

        var minWidth = cssStyle.GetPropertyValue("min-width");
        if (!string.IsNullOrEmpty(minWidth))
        {
            style.MinWidth = ParseLength(minWidth, style.FontSize);
        }

        // Overflow
        var overflow = cssStyle.GetPropertyValue("overflow");
        if (!string.IsNullOrEmpty(overflow))
            style.Overflow = ParseOverflow(overflow);
    }

    // --- Parsing helpers ---

    private static DisplayMode ParseDisplay(string value) => value.Trim().ToLowerInvariant() switch
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

    private static FontWeight ParseFontWeight(string value) => value.Trim().ToLowerInvariant() switch
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

    private static TextAlignment ParseTextAlign(string value) => value.Trim().ToLowerInvariant() switch
    {
        "left" => TextAlignment.Left,
        "right" => TextAlignment.Right,
        "center" => TextAlignment.Center,
        "justify" => TextAlignment.Justify,
        _ => TextAlignment.Left,
    };

    private static WhiteSpaceMode ParseWhiteSpace(string value) => value.Trim().ToLowerInvariant() switch
    {
        "normal" => WhiteSpaceMode.Normal,
        "nowrap" => WhiteSpaceMode.NoWrap,
        "pre" => WhiteSpaceMode.Pre,
        "pre-wrap" => WhiteSpaceMode.PreWrap,
        "pre-line" => WhiteSpaceMode.PreLine,
        _ => WhiteSpaceMode.Normal,
    };

    private static OverflowMode ParseOverflow(string value) => value.Trim().ToLowerInvariant() switch
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
        else if (value.EndsWith("em"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var em))
                return em * parentFontSize;
        }
        else if (value.EndsWith("rem"))
        {
            if (double.TryParse(value[..^3], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var rem))
                return rem * 16; // root font size is 16px
        }
        else if (value.EndsWith('%'))
        {
            // Percentage — caller context determines base value; here we use parent font size for font-related
            if (double.TryParse(value[..^1], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var pct))
                return pct / 100.0 * parentFontSize;
        }
        else if (value.EndsWith("pt"))
        {
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var pt))
                return pt * 4.0 / 3.0; // 1pt = 4/3 px
        }
        else if (value.EndsWith("vh") || value.EndsWith("vw"))
        {
            // Viewport units - approximate with fixed viewport
            if (double.TryParse(value[..^2], System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var v))
                return v * 10; // rough approximation
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

    private static Thickness ParseBoxEdges(ICssStyleDeclaration css, string property, Thickness fallback, double fontSize, bool isBorder = false)
    {
        string suffix = isBorder ? "-width" : "";

        var top = ParseLength(css.GetPropertyValue($"{property}-top{suffix}"), fontSize) ?? fallback.Top;
        var right = ParseLength(css.GetPropertyValue($"{property}-right{suffix}"), fontSize) ?? fallback.Right;
        var bottom = ParseLength(css.GetPropertyValue($"{property}-bottom{suffix}"), fontSize) ?? fallback.Bottom;
        var left = ParseLength(css.GetPropertyValue($"{property}-left{suffix}"), fontSize) ?? fallback.Left;

        // Also check shorthand
        var shorthand = css.GetPropertyValue(isBorder ? $"{property}-width" : property);
        if (!string.IsNullOrEmpty(shorthand))
        {
            var parts = shorthand.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length == 1)
            {
                var v = ParseLength(parts[0], fontSize) ?? 0;
                return new Thickness(v);
            }
            if (parts.Length == 2)
            {
                var v = ParseLength(parts[0], fontSize) ?? 0;
                var h = ParseLength(parts[1], fontSize) ?? 0;
                return new Thickness(h, v, h, v);
            }
            if (parts.Length == 3)
            {
                var t = ParseLength(parts[0], fontSize) ?? 0;
                var h = ParseLength(parts[1], fontSize) ?? 0;
                var b = ParseLength(parts[2], fontSize) ?? 0;
                return new Thickness(h, t, h, b);
            }
            if (parts.Length >= 4)
            {
                var t2 = ParseLength(parts[0], fontSize) ?? 0;
                var r2 = ParseLength(parts[1], fontSize) ?? 0;
                var b2 = ParseLength(parts[2], fontSize) ?? 0;
                var l2 = ParseLength(parts[3], fontSize) ?? 0;
                return new Thickness(l2, t2, r2, b2);
            }
        }

        return new Thickness(left, top, right, bottom);
    }
}
