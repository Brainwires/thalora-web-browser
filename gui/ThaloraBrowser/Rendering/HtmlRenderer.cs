using System.Text.Json;
using System.Text.Json.Serialization;
using Avalonia;
using Avalonia.Media;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Orchestrates the rendering pipeline using layout data computed on the Rust side:
/// Rust (HTML → CSS → Layout) → JSON → Deserialize → LayoutBox tree → Paint (Avalonia)
/// </summary>
public class HtmlRenderer : IDisposable
{
    private readonly PaintContext _paintContext;
    private readonly HitTester _hitTester;
    private readonly ImageCache _imageCache;

    private LayoutBox? _currentLayout;
    private string? _baseUrl;

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    public HtmlRenderer()
    {
        _hitTester = new HitTester();
        _imageCache = new ImageCache();
        _paintContext = new PaintContext(_imageCache);
    }

    /// <summary>
    /// The current layout tree (null if no page has been rendered).
    /// </summary>
    public LayoutBox? CurrentLayout => _currentLayout;

    /// <summary>
    /// The paint context for drawing.
    /// </summary>
    public PaintContext PaintContext => _paintContext;

    /// <summary>
    /// The hit tester for coordinate-to-element mapping.
    /// </summary>
    public HitTester HitTester => _hitTester;

    /// <summary>
    /// The image cache for downloaded images.
    /// </summary>
    public ImageCache ImageCache => _imageCache;

    /// <summary>
    /// Total content height of the current layout (for scrolling).
    /// </summary>
    public double ContentHeight
    {
        get
        {
            if (_currentLayout == null) return 0;
            return CalculateTotalHeight(_currentLayout);
        }
    }

    /// <summary>
    /// Render from JSON layout data computed by the Rust engine.
    /// This replaces the old RenderPageAsync that used AngleSharp.
    /// </summary>
    public LayoutBox? RenderFromLayoutJson(string json, string? baseUrl)
    {
        _baseUrl = baseUrl;

        try
        {
            var rustLayout = JsonSerializer.Deserialize<RustLayoutResult>(json, JsonOptions);
            if (rustLayout == null)
                return null;

            _currentLayout = ConvertToLayoutBox(rustLayout);
            return _currentLayout;
        }
        catch (Exception ex)
        {
            // On deserialization failure, render an error box
            _currentLayout = CreateErrorBox(ex.Message);
            return _currentLayout;
        }
    }

    /// <summary>
    /// Resolve a potentially relative URL against the current base URL.
    /// </summary>
    public string? ResolveUrl(string? href)
    {
        if (string.IsNullOrEmpty(href))
            return null;

        // Already absolute
        if (Uri.TryCreate(href, UriKind.Absolute, out var absolute))
            return absolute.ToString();

        // Resolve relative to base URL
        if (!string.IsNullOrEmpty(_baseUrl) && Uri.TryCreate(_baseUrl, UriKind.Absolute, out var baseUri))
        {
            if (Uri.TryCreate(baseUri, href, out var resolved))
                return resolved.ToString();
        }

        return href;
    }

    /// <summary>
    /// Convert the Rust LayoutResult JSON structure into our LayoutBox tree for painting.
    /// </summary>
    private static LayoutBox ConvertToLayoutBox(RustLayoutResult rustResult)
    {
        if (rustResult.Elements == null || rustResult.Elements.Count == 0)
        {
            return new LayoutBox
            {
                Type = BoxType.Block,
                Style = CreateDefaultRootStyle(),
                ContentRect = new Rect(0, 0, rustResult.Width, rustResult.Height),
            };
        }

        // The root element is usually <html>
        return ConvertElement(rustResult.Elements[0], null);
    }

    /// <summary>
    /// Recursively convert a Rust ElementLayout to a C# LayoutBox.
    /// </summary>
    private static LayoutBox ConvertElement(RustElementLayout el, CssComputedStyle? parentStyle)
    {
        var style = BuildComputedStyle(el, parentStyle);

        // Use content_box (content area excluding border+padding) when available.
        // The Rust side returns x/y/width/height as border-box coords, but LayoutBox
        // computed properties (PaddingBox, BorderBox, MarginBox) expand outward from
        // ContentRect — so ContentRect must be the innermost content-box.
        var contentRect = el.ContentBox != null
            ? new Rect(el.ContentBox.X, el.ContentBox.Y, Math.Max(0, el.ContentBox.Width), Math.Max(0, el.ContentBox.Height))
            : new Rect(el.X, el.Y, el.Width, el.Height);

        var box = new LayoutBox
        {
            Type = MapDisplayToBoxType(el.Display, el.Tag),
            Style = style,
            ContentRect = contentRect,
            Margin = el.Margin != null
                ? new Thickness(el.Margin.Left, el.Margin.Top, el.Margin.Right, el.Margin.Bottom)
                : style.Margin,
            Padding = el.Padding != null
                ? new Thickness(el.Padding.Left, el.Padding.Top, el.Padding.Right, el.Padding.Bottom)
                : style.Padding,
            Border = el.BorderSides != null
                ? new Thickness(el.BorderSides.Left, el.BorderSides.Top, el.BorderSides.Right, el.BorderSides.Bottom)
                : style.BorderWidth,
            LinkHref = el.LinkHref,
            ImageSource = el.ImgSrc,
        };

        // Handle text content — create TextRuns
        if (!string.IsNullOrEmpty(el.TextContent))
        {
            box.TextRuns = new List<TextRun>
            {
                new TextRun
                {
                    Text = el.TextContent,
                    Style = style,
                    LinkHref = el.LinkHref,
                    Bounds = contentRect,
                }
            };

            // Text nodes are anonymous boxes
            if (el.Tag == "#text")
                box.Type = BoxType.Anonymous;
        }

        // Convert children recursively
        if (el.Children != null)
        {
            foreach (var child in el.Children)
            {
                var childBox = ConvertElement(child, style);
                box.Children.Add(childBox);
            }

            // Assign sequential 1-based indices to ListItem children
            int listIndex = 1;
            foreach (var child in box.Children)
            {
                if (child.Type == BoxType.ListItem)
                {
                    child.ListItemIndex = listIndex++;
                }
            }
        }

        return box;
    }

    /// <summary>
    /// Build a CssComputedStyle from the Rust element's visual properties.
    /// </summary>
    private static CssComputedStyle BuildComputedStyle(RustElementLayout el, CssComputedStyle? parentStyle)
    {
        var style = new CssComputedStyle();

        // Inherit from parent
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

        // Display
        if (!string.IsNullOrEmpty(el.Display))
            style.Display = StyleResolver.ParseDisplay(el.Display);

        // Visibility
        style.IsVisible = el.IsVisible;
        if (style.Display == DisplayMode.None)
            style.IsVisible = false;

        // Font size
        if (el.FontSize.HasValue)
            style.FontSize = el.FontSize.Value;

        // Font weight
        if (!string.IsNullOrEmpty(el.FontWeight))
            style.FontWeight = StyleResolver.ParseFontWeight(el.FontWeight);

        // Font style
        if (!string.IsNullOrEmpty(el.FontStyle))
        {
            if (el.FontStyle == "italic" || el.FontStyle == "oblique")
                style.FontStyle = Avalonia.Media.FontStyle.Italic;
        }

        // Font family — map to bundled fonts for width agreement with Rust engine
        if (!string.IsNullOrEmpty(el.FontFamily))
            style.FontFamily = MapToBundledFontFamily(el.FontFamily);

        // Color
        if (!string.IsNullOrEmpty(el.Color))
        {
            var parsed = StyleResolver.ParseColor(el.Color);
            if (parsed.HasValue)
                style.Color = new SolidColorBrush(parsed.Value);
        }

        // Background color
        if (!string.IsNullOrEmpty(el.BackgroundColor) && el.BackgroundColor != "transparent")
        {
            var parsed = StyleResolver.ParseColor(el.BackgroundColor);
            if (parsed.HasValue)
                style.BackgroundColor = new SolidColorBrush(parsed.Value);
        }

        // Text alignment
        if (!string.IsNullOrEmpty(el.TextAlign))
            style.TextAlign = StyleResolver.ParseTextAlign(el.TextAlign);

        // Text decoration
        if (!string.IsNullOrEmpty(el.TextDecoration))
        {
            if (el.TextDecoration.Contains("underline"))
                style.TextDecorations = Avalonia.Media.TextDecorations.Underline;
            else if (el.TextDecoration.Contains("line-through"))
                style.TextDecorations = Avalonia.Media.TextDecorations.Strikethrough;
            else if (el.TextDecoration == "none")
                style.TextDecorations = null;
        }

        // Line height
        if (el.LineHeight.HasValue)
            style.LineHeight = el.LineHeight.Value;

        // White space
        if (!string.IsNullOrEmpty(el.WhiteSpace))
            style.WhiteSpace = StyleResolver.ParseWhiteSpace(el.WhiteSpace);

        // Opacity
        if (el.Opacity.HasValue)
            style.Opacity = Math.Clamp(el.Opacity.Value, 0, 1);

        // Margin
        if (el.Margin != null)
            style.Margin = new Thickness(el.Margin.Left, el.Margin.Top, el.Margin.Right, el.Margin.Bottom);
        style.HasAutoMarginLeft = el.MarginLeftAuto;
        style.HasAutoMarginRight = el.MarginRightAuto;

        // Padding
        if (el.Padding != null)
            style.Padding = new Thickness(el.Padding.Left, el.Padding.Top, el.Padding.Right, el.Padding.Bottom);

        // Border
        if (el.BorderSides != null)
            style.BorderWidth = new Thickness(el.BorderSides.Left, el.BorderSides.Top, el.BorderSides.Right, el.BorderSides.Bottom);
        else if (el.BorderWidth.HasValue)
            style.BorderWidth = new Thickness(el.BorderWidth.Value);

        if (!string.IsNullOrEmpty(el.BorderColor))
        {
            var parsed = StyleResolver.ParseColor(el.BorderColor);
            if (parsed.HasValue)
                style.BorderBrush = new SolidColorBrush(parsed.Value);
        }

        // Border radius
        if (el.BorderRadius.HasValue)
            style.BorderRadius = new CornerRadius(el.BorderRadius.Value);

        // Sizing — width/height come from taffy layout, not CSS
        // But we store explicit CSS values if the Rust side passed them
        if (el.Width > 0)
            style.Width = el.Width;
        if (el.Height > 0)
            style.Height = el.Height;

        // Overflow
        if (!string.IsNullOrEmpty(el.Overflow))
            style.Overflow = StyleResolver.ParseOverflow(el.Overflow);

        // List style
        if (!string.IsNullOrEmpty(el.ListStyleType))
            style.ListStyleType = ParseListStyleType(el.ListStyleType);

        return style;
    }

    private static ListStyleType ParseListStyleType(string value) => value.ToLowerInvariant() switch
    {
        "disc" => ListStyleType.Disc,
        "circle" => ListStyleType.Circle,
        "square" => ListStyleType.Square,
        "decimal" => ListStyleType.Decimal,
        "lower-alpha" => ListStyleType.LowerAlpha,
        "upper-alpha" => ListStyleType.UpperAlpha,
        "lower-roman" => ListStyleType.LowerRoman,
        "upper-roman" => ListStyleType.UpperRoman,
        _ => ListStyleType.None,
    };

    private static BoxType MapDisplayToBoxType(string? display, string? tag) => display?.ToLowerInvariant() switch
    {
        "block" => BoxType.Block,
        "inline" => BoxType.Inline,
        "inline-block" => BoxType.InlineBlock,
        "flex" => BoxType.Block,
        "list-item" => BoxType.ListItem,
        "table" => BoxType.TableBox,
        "table-row" => BoxType.TableRowBox,
        "table-cell" => BoxType.TableCellBox,
        _ => tag == "#text" ? BoxType.Anonymous : BoxType.Block,
    };

    private static CssComputedStyle CreateDefaultRootStyle()
    {
        return new CssComputedStyle
        {
            FontSize = 16,
            FontWeight = Avalonia.Media.FontWeight.Normal,
            FontFamily = new FontFamily("avares://ThaloraBrowser/Fonts#Noto Sans"),
            Color = new SolidColorBrush(Color.FromRgb(0, 0, 0)),
            BackgroundColor = new SolidColorBrush(Color.FromRgb(255, 255, 255)),
            Display = DisplayMode.Block,
            LineHeight = 1.4,
        };
    }

    // Bundled font family constants — these match the font files in src/gui/fonts/
    private static readonly FontFamily BundledNotoSans = new("avares://ThaloraBrowser/Fonts#Noto Sans");
    private static readonly FontFamily BundledNotoSerif = new("avares://ThaloraBrowser/Fonts#Noto Serif");
    private static readonly FontFamily BundledFiraMono = new("avares://ThaloraBrowser/Fonts#Fira Mono");

    /// <summary>
    /// Map a CSS font-family string to a bundled font for width agreement with Rust.
    /// Walks the comma-separated font stack and returns the first match.
    /// Supports CSS generic families: sans-serif, serif, monospace, system-ui, cursive, fantasy.
    /// </summary>
    private static FontFamily MapToBundledFontFamily(string cssFontFamily)
    {
        foreach (var name in cssFontFamily.Split(','))
        {
            var trimmed = name.Trim().Trim('"', '\'').ToLowerInvariant();

            // Monospace family
            switch (trimmed)
            {
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
                    return BundledFiraMono;

                // Serif family
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
                    return BundledNotoSerif;

                // Sans-serif family (most common on the web)
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
                    return BundledNotoSans;

                // CSS generic families that map to sans-serif
                case "cursive":
                case "fantasy":
                case "ui-serif":
                    return BundledNotoSerif;
                case "ui-monospace":
                    return BundledFiraMono;
                case "ui-rounded":
                case "math":
                case "emoji":
                    return BundledNotoSans;
            }
        }
        // Default to Noto Sans for any unrecognized font
        return BundledNotoSans;
    }

    private static LayoutBox CreateErrorBox(string message)
    {
        var style = CreateDefaultRootStyle();
        style.Color = new SolidColorBrush(Colors.Red);

        return new LayoutBox
        {
            Type = BoxType.Block,
            Style = style,
            ContentRect = new Rect(16, 16, 800, 40),
            TextRuns = new List<TextRun>
            {
                new TextRun
                {
                    Text = $"Layout Error: {message}",
                    Style = style,
                    Bounds = new Rect(16, 16, 800, 40),
                }
            },
        };
    }

    /// <summary>
    /// Calculate total height of a layout tree for scrolling.
    /// </summary>
    private static double CalculateTotalHeight(LayoutBox box)
    {
        double maxBottom = box.MarginBox.Bottom;

        foreach (var child in box.Children)
        {
            maxBottom = Math.Max(maxBottom, CalculateTotalHeight(child));
        }

        if (box.TextRuns != null)
        {
            foreach (var run in box.TextRuns)
            {
                maxBottom = Math.Max(maxBottom, run.Bounds.Bottom);
            }
        }

        return maxBottom;
    }

    public void Dispose()
    {
        _imageCache.Dispose();
    }
}

// --- JSON deserialization models matching Rust LayoutResult/ElementLayout ---

/// <summary>
/// Mirrors Rust LayoutResult struct.
/// </summary>
internal class RustLayoutResult
{
    [JsonPropertyName("width")]
    public double Width { get; set; }

    [JsonPropertyName("height")]
    public double Height { get; set; }

    [JsonPropertyName("elements")]
    public List<RustElementLayout>? Elements { get; set; }
}

/// <summary>
/// Mirrors Rust ElementLayout struct with all visual properties.
/// </summary>
internal class RustElementLayout
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = "";

    [JsonPropertyName("tag")]
    public string Tag { get; set; } = "";

    [JsonPropertyName("x")]
    public double X { get; set; }

    [JsonPropertyName("y")]
    public double Y { get; set; }

    [JsonPropertyName("width")]
    public double Width { get; set; }

    [JsonPropertyName("height")]
    public double Height { get; set; }

    [JsonPropertyName("content_box")]
    public RustContentBox? ContentBox { get; set; }

    [JsonPropertyName("children")]
    public List<RustElementLayout>? Children { get; set; }

    // Visual properties
    [JsonPropertyName("text_content")]
    public string? TextContent { get; set; }

    [JsonPropertyName("link_href")]
    public string? LinkHref { get; set; }

    [JsonPropertyName("img_src")]
    public string? ImgSrc { get; set; }

    [JsonPropertyName("background_color")]
    public string? BackgroundColor { get; set; }

    [JsonPropertyName("color")]
    public string? Color { get; set; }

    [JsonPropertyName("font_size")]
    public double? FontSize { get; set; }

    [JsonPropertyName("font_family")]
    public string? FontFamily { get; set; }

    [JsonPropertyName("font_weight")]
    public string? FontWeight { get; set; }

    [JsonPropertyName("font_style")]
    public string? FontStyle { get; set; }

    [JsonPropertyName("text_align")]
    public string? TextAlign { get; set; }

    [JsonPropertyName("text_decoration")]
    public string? TextDecoration { get; set; }

    [JsonPropertyName("line_height")]
    public double? LineHeight { get; set; }

    [JsonPropertyName("white_space")]
    public string? WhiteSpace { get; set; }

    [JsonPropertyName("border_radius")]
    public double? BorderRadius { get; set; }

    [JsonPropertyName("border_width")]
    public double? BorderWidth { get; set; }

    [JsonPropertyName("border_color")]
    public string? BorderColor { get; set; }

    [JsonPropertyName("opacity")]
    public double? Opacity { get; set; }

    [JsonPropertyName("overflow")]
    public string? Overflow { get; set; }

    [JsonPropertyName("list_style_type")]
    public string? ListStyleType { get; set; }

    [JsonPropertyName("margin_left_auto")]
    public bool MarginLeftAuto { get; set; }

    [JsonPropertyName("margin_right_auto")]
    public bool MarginRightAuto { get; set; }

    [JsonPropertyName("padding")]
    public RustBoxModelSides? Padding { get; set; }

    [JsonPropertyName("margin")]
    public RustBoxModelSides? Margin { get; set; }

    [JsonPropertyName("border_sides")]
    public RustBoxModelSides? BorderSides { get; set; }

    [JsonPropertyName("display")]
    public string? Display { get; set; }

    [JsonPropertyName("is_visible")]
    public bool IsVisible { get; set; } = true;

}

internal class RustContentBox
{
    [JsonPropertyName("x")]
    public double X { get; set; }

    [JsonPropertyName("y")]
    public double Y { get; set; }

    [JsonPropertyName("width")]
    public double Width { get; set; }

    [JsonPropertyName("height")]
    public double Height { get; set; }
}

internal class RustBoxModelSides
{
    [JsonPropertyName("top")]
    public double Top { get; set; }

    [JsonPropertyName("right")]
    public double Right { get; set; }

    [JsonPropertyName("bottom")]
    public double Bottom { get; set; }

    [JsonPropertyName("left")]
    public double Left { get; set; }
}
