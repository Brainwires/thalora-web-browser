using System.Linq;
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
    private readonly Action<string, string>? _onDomEvent;
    private double _viewportWidth;
    private double _viewportHeight;

    /// <summary>
    /// CSS canvas background color, determined after building the tree.
    /// Per CSS spec: if the root element (html) has no background, the body's background
    /// propagates to cover the entire canvas/viewport.
    /// WebContentControl should apply this as its own Background.
    /// </summary>
    public IBrush? CanvasBackground { get; private set; }

    /// <summary>
    /// Element ID → CSS selector mapping from the styled tree.
    /// Used by the GUI to dispatch DOM events to the JS engine.
    /// </summary>
    public Dictionary<string, string>? ElementSelectors { get; private set; }

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
        Action<string?>? onHoveredLinkChanged = null,
        Action<string, string>? onDomEvent = null)
    {
        _baseUrl = baseUrl;
        _imageCache = imageCache;
        _onLinkClicked = onLinkClicked;
        _onHoveredLinkChanged = onHoveredLinkChanged;
        _onDomEvent = onDomEvent;
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

        _viewportWidth = result.ViewportWidth;
        _viewportHeight = result.ViewportHeight;

        // CSS background propagation: per spec, if the root element (html) has no
        // explicit background, the body's background covers the canvas/viewport.
        ComputeCanvasBackground(result.Root);

        // Store element selectors for JS event dispatch
        ElementSelectors = result.ElementSelectors;

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
        // Exception: <pre> elements should always render their box (background, padding,
        // border-radius) even if CSS says display:contents — this is typically a CSS
        // specificity artifact and the visual block is always expected.
        if (styles.Display == "contents" && element.Tag == "pre")
            styles.Display = "block";

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

        // Apply hover behavior if this element has :hover style overrides
        if (element.HoverStyles != null)
            AttachHoverBehavior(border, content, styles, element.HoverStyles, fontSize);

        // For <a> elements rendered as blocks, attach click/hover handlers to the Border.
        // Inline <a> links (Span+Run) can't receive pointer events, but block-level links can.
        if (element.Tag == "a" && !string.IsNullOrEmpty(element.LinkHref))
            AttachLinkBehavior(border, element);

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
                var top = Len(styles.Margin.Top, fontSize) ?? 0;
                var bottom = Len(styles.Margin.Bottom, fontSize) ?? 0;
                border.Margin = new Thickness(0, top, 0, bottom);
            }
            else
            {
                border.Margin = Box(styles.Margin, fontSize);
            }
        }

        // Apply max-width (skip percentages — we don't know parent container size)
        var maxWidth = IsPercentage(styles.MaxWidth) ? null : Len(styles.MaxWidth, fontSize);
        if (maxWidth.HasValue)
            border.MaxWidth = maxWidth.Value;

        // Apply explicit width/height
        // Skip percentage widths/heights — Avalonia's default Stretch behavior handles 100%,
        // and we don't track parent sizes for other percentages.
        var width = IsPercentage(styles.Width) ? null : Len(styles.Width, fontSize);
        if (width.HasValue)
            border.Width = width.Value;

        var height = IsPercentage(styles.Height) ? null : Len(styles.Height, fontSize);
        if (height.HasValue)
            border.Height = height.Value;

        // Apply min-width/min-height
        var minWidth = IsPercentage(styles.MinWidth) ? null : Len(styles.MinWidth, fontSize);
        if (minWidth.HasValue)
            border.MinWidth = minWidth.Value;

        var minHeight = IsPercentage(styles.MinHeight) ? null : Len(styles.MinHeight, fontSize);
        if (minHeight.HasValue)
            border.MinHeight = minHeight.Value;

        // Apply max-height
        var maxHeight = IsPercentage(styles.MaxHeight) ? null : Len(styles.MaxHeight, fontSize);
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

        Panel panel;

        if (isFlex)
        {
            bool isRow = styles.FlexDirection != "column" && styles.FlexDirection != "column-reverse";
            bool isWrap = styles.FlexWrap == "wrap" || styles.FlexWrap == "wrap-reverse";

            if (isWrap && isRow)
            {
                // Use WrapPanel for flex-wrap in row direction
                var wrapPanel = new WrapPanel
                {
                    Orientation = Orientation.Horizontal,
                };
                panel = wrapPanel;
            }
            else
            {
                var stackPanel = new StackPanel
                {
                    Orientation = isRow ? Orientation.Horizontal : Orientation.Vertical,
                };
                // Gap
                var gap = Len(styles.Gap, fontSize);
                if (gap.HasValue)
                    stackPanel.Spacing = gap.Value;
                panel = stackPanel;
            }

            // justify-content: center → center items along the main axis
            if (styles.JustifyContent == "center")
            {
                if (isRow)
                    panel.HorizontalAlignment = HorizontalAlignment.Center;
                else
                    panel.VerticalAlignment = VerticalAlignment.Center;
            }
            else if (styles.JustifyContent == "flex-end" || styles.JustifyContent == "end")
            {
                if (isRow)
                    panel.HorizontalAlignment = HorizontalAlignment.Right;
                else
                    panel.VerticalAlignment = VerticalAlignment.Bottom;
            }
            else if (isRow && (styles.JustifyContent == "space-between" || styles.JustifyContent == "space-around"
                || styles.JustifyContent == "space-evenly"))
            {
                // Use Grid to distribute space between items.
                // Build all visible children, place them in Auto columns
                // separated by Star columns that absorb remaining space.
                // Star columns need a concrete width to distribute into, so if the
                // element has a max-width or width, apply it to the Grid directly.
                var grid = new Grid();
                grid.HorizontalAlignment = HorizontalAlignment.Stretch;

                // Give the Grid a concrete width so Star columns can distribute space.
                // Check explicit width first, then max-width, then fall back to viewport.
                var gridWidth = IsPercentage(styles.Width) ? null : Len(styles.Width, fontSize);
                if (!gridWidth.HasValue)
                    gridWidth = IsPercentage(styles.MaxWidth) ? null : Len(styles.MaxWidth, fontSize);
                if (gridWidth.HasValue)
                    grid.Width = gridWidth.Value;

                // Apply cross-axis alignment
                if (styles.AlignItems == "center")
                    grid.VerticalAlignment = VerticalAlignment.Center;

                var visibleChildren = element.Children
                    .Where(c => c.Styles.Display != "none")
                    .ToList();

                // Build column definitions: Auto | Star | Auto | Star | ... | Auto
                // For space-around: Star | Auto | Star | Auto | ... | Star
                bool isAround = styles.JustifyContent == "space-around" || styles.JustifyContent == "space-evenly";
                for (int i = 0; i < visibleChildren.Count; i++)
                {
                    if (i > 0 || isAround)
                        grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));
                    grid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
                }
                if (isAround)
                    grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));

                int col = isAround ? 1 : 0;
                foreach (var child in visibleChildren)
                {
                    var childControl = BuildControl(child, fontSize);
                    if (childControl != null)
                    {
                        // Apply cross-axis alignment to each item
                        if (styles.AlignItems == "center")
                            childControl.VerticalAlignment = VerticalAlignment.Center;
                        Grid.SetColumn(childControl, col);
                        grid.Children.Add(childControl);
                    }
                    col += 2; // skip spacer column
                }

                return grid;
            }

            // align-items → alignment along the cross axis
            if (styles.AlignItems == "center")
            {
                if (isRow)
                    panel.VerticalAlignment = VerticalAlignment.Center;
                else
                    panel.HorizontalAlignment = HorizontalAlignment.Center;
            }
            else if (styles.AlignItems == "flex-end" || styles.AlignItems == "end")
            {
                if (isRow)
                    panel.VerticalAlignment = VerticalAlignment.Bottom;
                else
                    panel.HorizontalAlignment = HorizontalAlignment.Right;
            }
            else if (styles.AlignItems == "flex-start" || styles.AlignItems == "start")
            {
                if (isRow)
                    panel.VerticalAlignment = VerticalAlignment.Top;
                else
                    panel.HorizontalAlignment = HorizontalAlignment.Left;
            }
        }
        else
        {
            panel = new StackPanel { Orientation = Orientation.Vertical };
        }

        // Process children: group consecutive inline children into text blocks,
        // and add block children directly.
        // CSS spec: In flex containers, ALL direct children are blockified (become flex items),
        // regardless of their display property. So skip inline grouping for flex children.
        var inlineBuffer = new List<StyledElement>();

        foreach (var child in element.Children)
        {
            if (child.Styles.Display == "none")
                continue;

            if (!isFlex && IsInlineElement(child))
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

                // Build block child (or blockified flex item)
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

            case "img":
            {
                if (string.IsNullOrEmpty(element.ImgSrc))
                    return;

                var (displayCtrl, imgCtrl) = CreateInlineImageWithControl(element, fontSize);
                _ = LoadImageAsync(imgCtrl, element.ImgSrc);
                inlines.Add(new InlineUIContainer { Child = displayCtrl });
                return;
            }

            case "a":
            {
                // Check if link contains an image (e.g., logo, avatar, icon)
                var imgChild = element.Children.FirstOrDefault(c => c.Tag == "img" && !string.IsNullOrEmpty(c.ImgSrc));
                if (imgChild != null)
                {
                    var imgStyles = imgChild.Styles;
                    var imgFontSize = StyleParser.ParseFontSize(imgStyles.FontSize, fontSize);
                    var (displayCtrl, imgCtrl) = CreateInlineImageWithControl(imgChild, imgFontSize);
                    displayCtrl.Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand);

                    if (!string.IsNullOrEmpty(element.LinkHref))
                    {
                        var href = element.LinkHref;
                        displayCtrl.PointerPressed += (_, _) =>
                        {
                            _onLinkClicked?.Invoke(href);
                            DispatchDomEvent("click", element.Id);
                        };
                        displayCtrl.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(href);
                        displayCtrl.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);
                    }

                    _ = LoadImageAsync(imgCtrl, imgChild.ImgSrc!);
                    inlines.Add(new InlineUIContainer { Child = displayCtrl });
                    return;
                }

                // Text link — use Span+Run for proper inline text flow.
                // InlineUIContainer breaks SelectableTextBlock layout when multiple exist.
                // Click handling is done at the Border level for block-rendered links
                // (see AttachLinkBehavior in BuildControl).
                var linkColor = StyleParser.ParseBrush(styles.Color)
                    ?? new SolidColorBrush(Color.FromRgb(0, 81, 195)); // #0051C3

                var linkSpan = new Span();
                linkSpan.Foreground = linkColor;
                if (styles.TextDecoration != "none")
                    linkSpan.TextDecorations = TextDecorations.Underline;
                if (styles.FontSize != null)
                    linkSpan.FontSize = fontSize;
                linkSpan.FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily);
                linkSpan.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                linkSpan.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);

                // Recursively add child content as proper inline elements
                foreach (var child in element.Children)
                    AddInlineContent(linkSpan.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    linkSpan.Inlines.Add(new Run(element.TextContent));

                // Skip empty links
                if (linkSpan.Inlines.Count == 0)
                    return;

                inlines.Add(linkSpan);
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

        // Handle width
        bool hasExplicitWidth = false;
        if (!string.IsNullOrEmpty(styles.Width) && styles.Width != "auto")
        {
            if (styles.Width.TrimEnd().EndsWith("%"))
            {
                // Percentage width — stretch to fill container
                image.HorizontalAlignment = HorizontalAlignment.Stretch;
                image.Stretch = Stretch.Uniform;
                hasExplicitWidth = true;
            }
            else
            {
                var width = Len(styles.Width, fontSize);
                if (width.HasValue && width.Value > 0)
                {
                    image.Width = width.Value;
                    hasExplicitWidth = true;
                }
            }
        }

        // Handle height
        bool hasExplicitHeight = false;
        if (!string.IsNullOrEmpty(styles.Height) && styles.Height != "auto")
        {
            if (!styles.Height.TrimEnd().EndsWith("%"))
            {
                var height = Len(styles.Height, fontSize);
                if (height.HasValue && height.Value > 0)
                {
                    image.Height = height.Value;
                    hasExplicitHeight = true;
                }
            }
        }

        // Default max size if no dimensions specified
        if (!hasExplicitWidth && !hasExplicitHeight)
        {
            image.MaxWidth = 800;
            image.MaxHeight = 600;
        }

        // Load image asynchronously
        if (!string.IsNullOrEmpty(imgSrc))
        {
            _ = LoadImageAsync(image, imgSrc);
        }

        // Wrap in Border for border-radius clipping
        if (!string.IsNullOrEmpty(styles.BorderRadius))
        {
            var border = new Border
            {
                Child = image,
                ClipToBounds = true,
                CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize),
            };
            if (hasExplicitWidth) border.Width = image.Width;
            if (hasExplicitHeight) border.Height = image.Height;
            return border;
        }

        return image;
    }

    /// <summary>
    /// Create an Image control for use in InlineUIContainer (inside text flow).
    /// Returns the display control (may be Image or Border wrapping Image) and the Image for async loading.
    /// </summary>
    private (Control displayControl, Avalonia.Controls.Image imageControl) CreateInlineImageWithControl(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var imageControl = new Avalonia.Controls.Image
        {
            Stretch = Stretch.Uniform,
            VerticalAlignment = VerticalAlignment.Center,
        };

        // Handle width
        double? imgWidth = null;
        if (!string.IsNullOrEmpty(styles.Width) && styles.Width != "auto"
            && !styles.Width.TrimEnd().EndsWith("%"))
        {
            var w = Len(styles.Width, fontSize);
            if (w.HasValue && w.Value > 0)
            {
                imageControl.Width = w.Value;
                imgWidth = w.Value;
            }
        }

        // Handle height
        double? imgHeight = null;
        if (!string.IsNullOrEmpty(styles.Height) && styles.Height != "auto"
            && !styles.Height.TrimEnd().EndsWith("%"))
        {
            var h = Len(styles.Height, fontSize);
            if (h.HasValue && h.Value > 0)
            {
                imageControl.Height = h.Value;
                imgHeight = h.Value;
            }
        }

        // Wrap in Border for border-radius clipping
        if (!string.IsNullOrEmpty(styles.BorderRadius))
        {
            var border = new Border
            {
                Child = imageControl,
                ClipToBounds = true,
                CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize),
            };
            if (imgWidth.HasValue) border.Width = imgWidth.Value;
            if (imgHeight.HasValue) border.Height = imgHeight.Value;
            return (border, imageControl);
        }

        return (imageControl, imageControl);
    }

    private async Task LoadImageAsync(Avalonia.Controls.Image imageControl, string src)
    {
        try
        {
            // Skip SVG images — Avalonia Bitmap doesn't support SVG format
            if (src.Contains(".svg", StringComparison.OrdinalIgnoreCase))
                return;

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
            Margin = Box(styles.Margin, fontSize),
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
            border.BorderThickness = Box(styles.BorderWidth, fontSize);

        // Border radius
        if (!string.IsNullOrEmpty(styles.BorderRadius))
            border.CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize);

        // Padding
        if (styles.Padding != null)
            border.Padding = Box(styles.Padding, fontSize);

        return border;
    }

    /// <summary>
    /// Attach click and hover handlers to a block-rendered link element.
    /// This handles <a> elements that go through BuildControl (block-level links).
    /// </summary>
    private void AttachLinkBehavior(Border border, StyledElement element)
    {
        var href = element.LinkHref!;
        border.Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand);
        border.PointerPressed += (_, _) =>
        {
            _onLinkClicked?.Invoke(href);
            DispatchDomEvent("click", element.Id);
        };
        border.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(href);
        border.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);
    }

    /// <summary>
    /// Attach hover behavior to a border+content pair.
    /// On PointerEntered: swap to hover styles. On PointerExited: restore originals.
    /// All brushes are precomputed at build time for instant response.
    /// </summary>
    private void AttachHoverBehavior(Border border, Control content, ResolvedStyles normalStyles, ResolvedStyles hoverStyles, double fontSize)
    {
        // Precompute normal state
        var normalBg = StyleParser.ParseBrush(normalStyles.BackgroundColor);
        var normalBorderBrush = StyleParser.ParseBrush(normalStyles.BorderColor);
        var normalOpacity = normalStyles.Opacity ?? 1.0f;

        // Precompute hover state
        var hoverBg = StyleParser.ParseBrush(hoverStyles.BackgroundColor);
        var hoverBorderBrush = StyleParser.ParseBrush(hoverStyles.BorderColor);
        var hoverOpacity = hoverStyles.Opacity;
        var hoverCursor = hoverStyles.Cursor;

        // Text-specific hover properties
        IBrush? normalFg = null;
        IBrush? hoverFg = null;
        TextDecorationCollection? normalTextDeco = null;
        TextDecorationCollection? hoverTextDeco = null;

        if (content is SelectableTextBlock textBlock)
        {
            normalFg = textBlock.Foreground;
            if (hoverStyles.Color != null)
                hoverFg = StyleParser.ParseBrush(hoverStyles.Color);
            if (hoverStyles.TextDecoration != null)
            {
                hoverTextDeco = hoverStyles.TextDecoration.ToLowerInvariant() switch
                {
                    "underline" => TextDecorations.Underline,
                    "none" => null,
                    "line-through" => TextDecorations.Strikethrough,
                    _ => null,
                };
            }
            normalTextDeco = textBlock.TextDecorations;
        }

        border.PointerEntered += (_, _) =>
        {
            if (hoverBg != null)
                border.Background = hoverBg;
            if (hoverBorderBrush != null)
                border.BorderBrush = hoverBorderBrush;
            if (hoverOpacity.HasValue)
                border.Opacity = hoverOpacity.Value;
            if (hoverCursor == "pointer")
                border.Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand);

            if (content is SelectableTextBlock tb)
            {
                if (hoverFg != null)
                    tb.Foreground = hoverFg;
                if (hoverStyles.TextDecoration != null)
                    tb.TextDecorations = hoverTextDeco;
            }
        };

        border.PointerExited += (_, _) =>
        {
            border.Background = normalBg;
            border.BorderBrush = normalBorderBrush;
            border.Opacity = normalOpacity;
            border.Cursor = null;

            if (content is SelectableTextBlock tb)
            {
                if (hoverFg != null)
                    tb.Foreground = normalFg;
                if (hoverStyles.TextDecoration != null)
                    tb.TextDecorations = normalTextDeco;
            }
        };
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
            // Avalonia clips text to LineHeight (unlike CSS where text overflows line boxes).
            // Tight CSS line-heights (e.g., 1.15) would clip descenders/ascenders.
            // Only set LineHeight when it's large enough to prevent clipping.
            // Below ~1.25x, let Avalonia use its natural font metrics.
            if (lhMultiplier >= 1.25)
                textBlock.LineHeight = fontSize * lhMultiplier;
        }

        // White-space handling
        if (styles.WhiteSpace is "pre" or "pre-wrap" or "pre-line" or "break-spaces")
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
    /// <summary>
    /// Check if a CSS value is a percentage (e.g., "100%", "50%").
    /// Percentage widths/heights can't be resolved without parent size,
    /// so we skip them and let Avalonia's layout handle it.
    /// </summary>
    private static bool IsPercentage(string? value)
        => value != null && value.TrimEnd().EndsWith('%');

    /// <summary>
    /// Parse a CSS length with viewport unit support. Shorthand for passing viewport dims.
    /// </summary>
    private double? Len(string? value, double fontSize, double parentSize = 0)
        => StyleParser.ParseLength(value, fontSize, parentSize, _viewportWidth, _viewportHeight);

    /// <summary>
    /// Parse box sides with viewport unit support.
    /// </summary>
    private Thickness Box(StyleBoxSides? sides, double fontSize, double parentSize = 0)
        => StyleParser.ParseBoxSides(sides, fontSize, parentSize, _viewportWidth, _viewportHeight);

    private static bool IsInlineElement(StyledElement element)
    {
        // Images: treat as block when they're likely content images
        if (element.Tag == "img")
        {
            // Percentage width → block (full-width stretching)
            if (IsPercentage(element.Styles.Width))
                return false;
            // No explicit dimensions → likely a content image, not an inline icon
            if (string.IsNullOrEmpty(element.Styles.Width) && string.IsNullOrEmpty(element.Styles.Height))
                return false;
        }

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
    /// Dispatch a DOM event to the JS engine via the callback.
    /// </summary>
    private void DispatchDomEvent(string eventType, string elementId)
    {
        _onDomEvent?.Invoke(eventType, elementId);
    }

    /// <summary>
    /// Compute the canvas background per CSS background propagation rules.
    /// If html has no background, body's background propagates to the canvas.
    /// When propagated, body's background is removed (canvas paints it instead).
    /// </summary>
    private void ComputeCanvasBackground(StyledElement root)
    {
        CanvasBackground = null;

        if (root.Tag != "html")
            return;

        var htmlBg = StyleParser.ParseBrush(root.Styles.BackgroundColor);
        if (htmlBg != null)
        {
            // html has explicit background — use it as canvas, remove from html
            CanvasBackground = htmlBg;
            root.Styles.BackgroundColor = null;
            return;
        }

        // html has no background — propagate body's background to canvas
        var body = root.Children.FirstOrDefault(c => c.Tag == "body");
        if (body != null)
        {
            var bodyBg = StyleParser.ParseBrush(body.Styles.BackgroundColor);
            if (bodyBg != null)
            {
                CanvasBackground = bodyBg;
                // Per spec, body background is consumed by the canvas propagation
                body.Styles.BackgroundColor = null;
            }
        }

        // Default white canvas if nothing set
        CanvasBackground ??= Brushes.White;
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
