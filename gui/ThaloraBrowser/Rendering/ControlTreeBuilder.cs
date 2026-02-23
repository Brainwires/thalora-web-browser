using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Converts a styled element tree (from Rust) into an Avalonia control tree.
/// Rust resolves CSS (lightningcss); this class builds native Avalonia controls
/// for layout and rendering — no manual painting, no PaintContext.
///
/// Control mapping:
///   block container → Border (bg/border) wrapping StackPanel (children)
///   flex container  → Border wrapping StackPanel (horizontal/vertical)
///   paragraph/heading → Border wrapping SelectableTextBlock with Inlines
///   #text → Run inside parent's SelectableTextBlock
///   img → Image control with async bitmap loading
///   pre/code block → Border with monospace TextBlock
///   list → StackPanel with list item panels
///   display:none → skipped
/// </summary>
public class ControlTreeBuilder
{
    private readonly string? _baseUrl;
    private readonly ImageCache _imageCache;
    private readonly Action<string>? _onLinkClicked;
    private readonly Action<string?>? _onHoveredLinkChanged;

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = null, // We use explicit [JsonPropertyName] attributes
        DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull,
    };

    // Tags that are always inline (their text gets concatenated into parent's TextBlock)
    private static readonly HashSet<string> InlineTags = new(StringComparer.OrdinalIgnoreCase)
    {
        "#text", "span", "strong", "b", "em", "i", "a", "code",
        "sub", "sup", "small", "abbr", "mark", "del", "ins",
        "u", "s", "q", "cite", "br", "wbr", "time", "data",
        "kbd", "samp", "var", "dfn", "bdi", "bdo",
    };

    // Tags that should be treated as block even if inline
    private static readonly HashSet<string> AlwaysBlockTags = new(StringComparer.OrdinalIgnoreCase)
    {
        "div", "section", "article", "main", "header", "footer", "nav", "aside",
        "p", "h1", "h2", "h3", "h4", "h5", "h6",
        "ul", "ol", "li", "dl", "dt", "dd",
        "blockquote", "pre", "figure", "figcaption",
        "table", "thead", "tbody", "tfoot", "tr", "td", "th",
        "form", "fieldset", "legend",
        "details", "summary", "dialog",
        "hr", "address",
    };

    public ControlTreeBuilder(
        string? baseUrl,
        ImageCache imageCache,
        Action<string>? onLinkClicked = null,
        Action<string?>? onHoveredLinkChanged = null)
    {
        _baseUrl = baseUrl;
        _imageCache = imageCache;
        _onLinkClicked = onLinkClicked;
        _onHoveredLinkChanged = onHoveredLinkChanged;
    }

    /// <summary>
    /// Deserialize JSON from Rust and build an Avalonia control tree.
    /// </summary>
    public Control? BuildFromJson(string json)
    {
        StyledTreeResult? result;
        try
        {
            result = JsonSerializer.Deserialize<StyledTreeResult>(json, JsonOptions);
        }
        catch (JsonException ex)
        {
            System.Diagnostics.Debug.WriteLine($"[ControlTreeBuilder] JSON parse error: {ex.Message}");
            return CreateErrorControl($"JSON parse error: {ex.Message}");
        }

        if (result?.Root == null)
            return CreateErrorControl("Empty styled tree from Rust");

        return BuildControl(result.Root, 16.0);
    }

    /// <summary>
    /// Recursively convert a StyledElement into an Avalonia Control.
    /// parentFontSize is the inherited font size for em/% resolution.
    /// </summary>
    private Control? BuildControl(StyledElement element, double parentFontSize)
    {
        var styles = element.Styles;

        // Skip display:none and visibility:hidden
        if (styles.Display == "none")
            return null;
        if (styles.Visibility == "hidden")
            return null;

        // Resolve font size for this element (used for em units in children)
        var fontSize = StyleParser.ParseFontSize(styles.FontSize, parentFontSize);

        // display:contents — don't generate a box, pass children to parent
        // This is used by frameworks like Astro (<astro-island>) and CSS grid layouts.
        if (styles.Display == "contents" && element.Children.Count > 0)
        {
            if (element.Children.Count == 1)
                return BuildControl(element.Children[0], fontSize);

            // Multiple children: wrap in a transparent StackPanel
            var panel = new StackPanel { Orientation = Orientation.Vertical };
            foreach (var child in element.Children)
            {
                var childControl = BuildControl(child, fontSize);
                if (childControl != null)
                    panel.Children.Add(childControl);
            }
            return panel;
        }

        // Special handling by tag
        switch (element.Tag)
        {
            case "#text":
                // Text nodes are handled by the parent's inline builder
                // If we reach here, it's a standalone text node — wrap in TextBlock
                return BuildTextBlock(element, fontSize);

            case "img":
                return BuildImage(element, fontSize);

            case "hr":
                return BuildHorizontalRule(element, fontSize);

            case "br":
                return null; // Handled inline as line breaks
        }

        // Determine if this element has only inline children
        bool hasOnlyInlineChildren = element.Children.Count > 0
            && element.Children.All(c => IsInlineElement(c));

        // Build the appropriate control
        Control content;
        if (hasOnlyInlineChildren)
        {
            // Build a SelectableTextBlock with Inlines for inline content
            content = BuildInlineContent(element, fontSize);
        }
        else if (element.Children.Count > 0)
        {
            // Build a panel with child controls
            content = BuildBlockContent(element, fontSize);
        }
        else if (!string.IsNullOrEmpty(element.TextContent))
        {
            // Leaf element with text content
            content = BuildTextBlock(element, fontSize);
        }
        else
        {
            // Empty element — still render the border/background
            content = new Panel();
        }

        // Wrap in Border for background, border, border-radius, padding
        var border = WrapInBorder(content, styles, fontSize);

        // Apply margin
        if (styles.Margin != null)
        {
            bool leftAuto = StyleParser.IsAutoMargin(styles.Margin.Left);
            bool rightAuto = StyleParser.IsAutoMargin(styles.Margin.Right);

            if (leftAuto && rightAuto)
            {
                // margin: auto centering
                border.HorizontalAlignment = HorizontalAlignment.Center;
                // Apply top/bottom margin only
                var top = StyleParser.ParseLength(styles.Margin.Top, fontSize) ?? 0;
                var bottom = StyleParser.ParseLength(styles.Margin.Bottom, fontSize) ?? 0;
                border.Margin = new Thickness(0, top, 0, bottom);
            }
            else
            {
                border.Margin = StyleParser.ParseBoxSides(styles.Margin, fontSize);
            }
        }

        // Apply max-width
        var maxWidth = StyleParser.ParseLength(styles.MaxWidth, fontSize);
        if (maxWidth.HasValue)
            border.MaxWidth = maxWidth.Value;

        // Apply explicit width/height
        var width = StyleParser.ParseLength(styles.Width, fontSize);
        if (width.HasValue)
            border.Width = width.Value;

        var height = StyleParser.ParseLength(styles.Height, fontSize);
        if (height.HasValue)
            border.Height = height.Value;

        // Apply min-width/min-height
        var minWidth = StyleParser.ParseLength(styles.MinWidth, fontSize);
        if (minWidth.HasValue)
            border.MinWidth = minWidth.Value;

        var minHeight = StyleParser.ParseLength(styles.MinHeight, fontSize);
        if (minHeight.HasValue)
            border.MinHeight = minHeight.Value;

        // Apply max-height
        var maxHeight = StyleParser.ParseLength(styles.MaxHeight, fontSize);
        if (maxHeight.HasValue)
            border.MaxHeight = maxHeight.Value;

        // Apply opacity
        if (styles.Opacity.HasValue && styles.Opacity.Value < 1.0f)
            border.Opacity = styles.Opacity.Value;

        // Apply overflow:hidden as clipping
        if (styles.Overflow == "hidden" || styles.Overflow == "scroll" || styles.Overflow == "auto")
            border.ClipToBounds = true;

        return border;
    }

    /// <summary>
    /// Build a StackPanel (or WrapPanel) containing block-level child controls.
    /// </summary>
    private Control BuildBlockContent(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        bool isFlex = styles.Display == "flex" || styles.Display == "inline-flex";

        var panel = new StackPanel();

        if (isFlex)
        {
            // Flex direction
            bool isRow = styles.FlexDirection != "column" && styles.FlexDirection != "column-reverse";
            panel.Orientation = isRow ? Orientation.Horizontal : Orientation.Vertical;

            // Gap
            var gap = StyleParser.ParseLength(styles.Gap, fontSize);
            if (gap.HasValue)
                panel.Spacing = gap.Value;
        }
        else
        {
            panel.Orientation = Orientation.Vertical;
        }

        // Process children: group consecutive inline children into text blocks,
        // and add block children directly
        var inlineBuffer = new List<StyledElement>();

        foreach (var child in element.Children)
        {
            if (child.Styles.Display == "none")
                continue;

            if (IsInlineElement(child))
            {
                inlineBuffer.Add(child);
            }
            else
            {
                // Flush any accumulated inline children
                if (inlineBuffer.Count > 0)
                {
                    var textBlock = BuildInlineGroup(inlineBuffer, fontSize, element);
                    if (textBlock != null)
                        panel.Children.Add(textBlock);
                    inlineBuffer.Clear();
                }

                // Build block child
                var childControl = BuildControl(child, fontSize);
                if (childControl != null)
                    panel.Children.Add(childControl);
            }
        }

        // Flush remaining inline children
        if (inlineBuffer.Count > 0)
        {
            var textBlock = BuildInlineGroup(inlineBuffer, fontSize, element);
            if (textBlock != null)
                panel.Children.Add(textBlock);
        }

        return panel;
    }

    /// <summary>
    /// Build a SelectableTextBlock with Inlines for a block element
    /// whose children are all inline.
    /// </summary>
    private Control BuildInlineContent(StyledElement element, double fontSize)
    {
        var textBlock = new SelectableTextBlock();
        ApplyTextProperties(textBlock, element.Styles, fontSize);

        foreach (var child in element.Children)
        {
            AddInlineContent(textBlock.Inlines!, child, fontSize);
        }

        return textBlock;
    }

    /// <summary>
    /// Build a SelectableTextBlock from a group of inline StyledElements
    /// (used when block content has a mix of inline and block children).
    /// </summary>
    private Control? BuildInlineGroup(List<StyledElement> inlineElements, double fontSize, StyledElement parent)
    {
        if (inlineElements.Count == 0)
            return null;

        // Check if all elements are empty text
        bool allEmpty = inlineElements.All(e =>
            e.Tag == "#text" && string.IsNullOrWhiteSpace(e.TextContent));
        if (allEmpty)
            return null;

        var textBlock = new SelectableTextBlock();
        ApplyTextProperties(textBlock, parent.Styles, fontSize);

        foreach (var child in inlineElements)
        {
            AddInlineContent(textBlock.Inlines!, child, fontSize);
        }

        return textBlock;
    }

    /// <summary>
    /// Recursively add inline content to an InlineCollection.
    /// </summary>
    private void AddInlineContent(InlineCollection inlines, StyledElement element, double parentFontSize)
    {
        var styles = element.Styles;
        var fontSize = StyleParser.ParseFontSize(styles.FontSize, parentFontSize);

        switch (element.Tag)
        {
            case "#text":
            {
                var text = element.TextContent ?? "";
                if (string.IsNullOrEmpty(text))
                    return;

                var run = new Run(text);

                // Apply text styling from parent/inherited styles
                if (styles.Color != null)
                {
                    var brush = StyleParser.ParseBrush(styles.Color);
                    if (brush != null)
                        run.Foreground = brush;
                }
                if (styles.FontWeight != null)
                    run.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                if (styles.FontStyle != null)
                    run.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);
                if (styles.FontFamily != null)
                    run.FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily);
                if (styles.FontSize != null)
                    run.FontSize = fontSize;

                inlines.Add(run);
                return;
            }

            case "br":
                inlines.Add(new LineBreak());
                return;

            case "strong":
            case "b":
            {
                var bold = new Bold();
                foreach (var child in element.Children)
                    AddInlineContent(bold.Inlines, child, fontSize);
                // If no children but has text_content
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    bold.Inlines.Add(new Run(element.TextContent));
                inlines.Add(bold);
                return;
            }

            case "em":
            case "i":
            {
                var italic = new Italic();
                foreach (var child in element.Children)
                    AddInlineContent(italic.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    italic.Inlines.Add(new Run(element.TextContent));
                inlines.Add(italic);
                return;
            }

            case "a":
            {
                // Build link text content
                var linkText = "";
                if (element.Children.Count > 0)
                {
                    // Collect text from children
                    linkText = CollectInlineText(element);
                }
                else if (!string.IsNullOrEmpty(element.TextContent))
                {
                    linkText = element.TextContent;
                }

                if (string.IsNullOrEmpty(linkText))
                    return;

                var linkColor = StyleParser.ParseBrush(styles.Color)
                    ?? new SolidColorBrush(Color.FromRgb(0, 81, 195)); // #0051C3

                // Use InlineUIContainer with a clickable TextBlock for link behavior
                var linkTextBlock = new TextBlock
                {
                    Text = linkText,
                    Foreground = linkColor,
                    FontSize = fontSize,
                    FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily),
                    FontWeight = StyleParser.ParseFontWeight(styles.FontWeight),
                    FontStyle = StyleParser.ParseFontStyle(styles.FontStyle),
                    TextDecorations = TextDecorations.Underline,
                    Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand),
                };

                if (!string.IsNullOrEmpty(element.LinkHref))
                {
                    var href = element.LinkHref;
                    linkTextBlock.PointerPressed += (_, _) => _onLinkClicked?.Invoke(href);
                    linkTextBlock.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(href);
                    linkTextBlock.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);
                }

                inlines.Add(new InlineUIContainer { Child = linkTextBlock });
                return;
            }

            case "code":
            {
                // Inline code: monospace with background
                var span = new Span();
                span.FontFamily = StyleParser.BundledFiraMono;
                if (styles.FontSize != null)
                    span.FontSize = fontSize;

                var bgColor = StyleParser.ParseColor(styles.BackgroundColor);
                if (bgColor.HasValue)
                    span.Background = new SolidColorBrush(bgColor.Value);

                foreach (var child in element.Children)
                    AddInlineContent(span.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    span.Inlines.Add(new Run(element.TextContent));

                inlines.Add(span);
                return;
            }

            case "u":
            case "ins":
            {
                var underline = new Underline();
                foreach (var child in element.Children)
                    AddInlineContent(underline.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    underline.Inlines.Add(new Run(element.TextContent));
                inlines.Add(underline);
                return;
            }

            case "del":
            case "s":
            {
                var span = new Span();
                span.TextDecorations = TextDecorations.Strikethrough;
                foreach (var child in element.Children)
                    AddInlineContent(span.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    span.Inlines.Add(new Run(element.TextContent));
                inlines.Add(span);
                return;
            }

            default:
            {
                // Generic inline: wrap in Span with styling
                var span = new Span();
                if (styles.Color != null)
                {
                    var brush = StyleParser.ParseBrush(styles.Color);
                    if (brush != null) span.Foreground = brush;
                }
                if (styles.FontWeight != null)
                    span.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                if (styles.FontStyle != null)
                    span.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);
                if (styles.FontFamily != null)
                    span.FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily);

                foreach (var child in element.Children)
                    AddInlineContent(span.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    span.Inlines.Add(new Run(element.TextContent));

                inlines.Add(span);
                return;
            }
        }
    }

    /// <summary>
    /// Build a simple TextBlock for a text-only element.
    /// </summary>
    private Control BuildTextBlock(StyledElement element, double fontSize)
    {
        var text = element.TextContent ?? "";
        if (string.IsNullOrWhiteSpace(text))
            return new Panel(); // Empty placeholder

        var textBlock = new SelectableTextBlock
        {
            Text = text,
        };
        ApplyTextProperties(textBlock, element.Styles, fontSize);
        return textBlock;
    }

    /// <summary>
    /// Build an Image control for an &lt;img&gt; element.
    /// Loads the image asynchronously via ImageCache.
    /// </summary>
    private Control BuildImage(StyledElement element, double fontSize)
    {
        var imgSrc = element.ImgSrc;
        var styles = element.Styles;

        var image = new Avalonia.Controls.Image
        {
            Stretch = Stretch.Uniform,
            HorizontalAlignment = HorizontalAlignment.Left,
        };

        // Set explicit dimensions if available
        var width = StyleParser.ParseLength(styles.Width, fontSize);
        var height = StyleParser.ParseLength(styles.Height, fontSize);
        if (width.HasValue) image.Width = width.Value;
        if (height.HasValue) image.Height = height.Value;

        // Default size if none specified
        if (!width.HasValue && !height.HasValue)
        {
            image.MaxWidth = 800;
            image.MaxHeight = 600;
        }

        // Load image asynchronously
        if (!string.IsNullOrEmpty(imgSrc))
        {
            _ = LoadImageAsync(image, imgSrc);
        }

        return image;
    }

    private async Task LoadImageAsync(Avalonia.Controls.Image imageControl, string src)
    {
        try
        {
            var bitmap = await _imageCache.GetImageAsync(src, _baseUrl);
            if (bitmap != null)
            {
                await Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(() =>
                {
                    imageControl.Source = bitmap;
                });
            }
        }
        catch
        {
            // Image load failed — leave blank
        }
    }

    /// <summary>
    /// Build an HR element — a thin horizontal line.
    /// </summary>
    private Control BuildHorizontalRule(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var bgColor = StyleParser.ParseBrush(styles.BackgroundColor)
            ?? StyleParser.ParseBrush(styles.BorderColor)
            ?? new SolidColorBrush(Color.FromRgb(200, 200, 200));

        return new Border
        {
            Height = 1,
            Background = bgColor,
            Margin = StyleParser.ParseBoxSides(styles.Margin, fontSize),
            HorizontalAlignment = HorizontalAlignment.Stretch,
        };
    }

    /// <summary>
    /// Wrap a content control in a Border for background, border, radius, padding.
    /// </summary>
    private Border WrapInBorder(Control content, ResolvedStyles styles, double fontSize)
    {
        var border = new Border
        {
            Child = content,
        };

        // Background
        var bg = StyleParser.ParseBrush(styles.BackgroundColor);
        if (bg != null)
            border.Background = bg;

        // Border color + width
        var borderBrush = StyleParser.ParseBrush(styles.BorderColor);
        if (borderBrush != null)
            border.BorderBrush = borderBrush;

        if (styles.BorderWidth != null)
            border.BorderThickness = StyleParser.ParseBoxSides(styles.BorderWidth, fontSize);

        // Border radius
        if (!string.IsNullOrEmpty(styles.BorderRadius))
            border.CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize);

        // Padding
        if (styles.Padding != null)
            border.Padding = StyleParser.ParseBoxSides(styles.Padding, fontSize);

        return border;
    }

    /// <summary>
    /// Apply text-related CSS properties to a TextBlock.
    /// </summary>
    private static void ApplyTextProperties(SelectableTextBlock textBlock, ResolvedStyles styles, double fontSize)
    {
        textBlock.FontSize = fontSize;
        textBlock.FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily);
        textBlock.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
        textBlock.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);
        textBlock.TextAlignment = StyleParser.ParseTextAlignment(styles.TextAlign);
        textBlock.TextWrapping = TextWrapping.Wrap;

        if (styles.Color != null)
        {
            var brush = StyleParser.ParseBrush(styles.Color);
            if (brush != null)
                textBlock.Foreground = brush;
        }

        if (styles.LineHeight != null)
        {
            var lhMultiplier = StyleParser.ParseLineHeight(styles.LineHeight, fontSize);
            textBlock.LineHeight = fontSize * lhMultiplier;
        }

        // White-space handling
        if (styles.WhiteSpace == "pre" || styles.WhiteSpace == "pre-wrap" || styles.WhiteSpace == "pre-line")
        {
            textBlock.TextWrapping = styles.WhiteSpace == "pre" ? TextWrapping.NoWrap : TextWrapping.Wrap;
        }

        // Text decoration
        if (!string.IsNullOrEmpty(styles.TextDecoration))
        {
            textBlock.TextDecorations = styles.TextDecoration.ToLowerInvariant() switch
            {
                "underline" => TextDecorations.Underline,
                "line-through" => TextDecorations.Strikethrough,
                _ => null,
            };
        }
    }

    /// <summary>
    /// Determine if a StyledElement should be treated as inline content.
    /// </summary>
    private static bool IsInlineElement(StyledElement element)
    {
        // Explicit display overrides
        if (element.Styles.Display == "block" || element.Styles.Display == "flex"
            || element.Styles.Display == "grid" || element.Styles.Display == "list-item"
            || element.Styles.Display == "table")
            return false;

        if (element.Styles.Display == "inline" || element.Styles.Display == "inline-block")
            return true;

        // Tag-based classification
        if (AlwaysBlockTags.Contains(element.Tag))
            return false;

        return InlineTags.Contains(element.Tag);
    }

    /// <summary>
    /// Recursively collect all text content from an inline element tree.
    /// Used for building link text from nested inline elements.
    /// </summary>
    private static string CollectInlineText(StyledElement element)
    {
        if (!string.IsNullOrEmpty(element.TextContent))
            return element.TextContent;

        var sb = new System.Text.StringBuilder();
        foreach (var child in element.Children)
        {
            sb.Append(CollectInlineText(child));
        }
        return sb.ToString();
    }

    /// <summary>
    /// Create an error display control.
    /// </summary>
    private static Control CreateErrorControl(string message)
    {
        return new TextBlock
        {
            Text = $"Render error: {message}",
            Foreground = Brushes.Red,
            FontSize = 14,
            Margin = new Thickness(8),
            TextWrapping = TextWrapping.Wrap,
        };
    }
}
