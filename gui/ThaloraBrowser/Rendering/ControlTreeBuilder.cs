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
///
/// Split into partial class files:
///   ControlTreeBuilder.Flex.cs     — BuildBlockContent (flex layout, nav list detection)
///   ControlTreeBuilder.Grid.cs     — BuildGridContent, grid template parsing
///   ControlTreeBuilder.Inline.cs   — BuildInlineContent, AddInlineContent (inline rendering)
///   ControlTreeBuilder.Elements.cs — BuildTextBlock, BuildImage, BuildListItemWithMarker, etc.
///   ControlTreeBuilder.Styles.cs   — WrapInBorder, AttachHoverBehavior, helpers, canvas background
/// </summary>
public partial class ControlTreeBuilder
{
    private readonly string? _baseUrl;
    private readonly ImageCache _imageCache;
    private readonly Action<string>? _onLinkClicked;
    private readonly Action<string?>? _onHoveredLinkChanged;
    private readonly Action<string, string>? _onDomEvent;
    private readonly ElementActionRegistry _elementActions = new();
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

    /// <summary>
    /// Registry of interactive elements (links, hover targets) and their programmatic actions.
    /// Populated during BuildFromJson(). Used by BrowserControlServer for /click-element, /hover-element, etc.
    /// </summary>
    public ElementActionRegistry ElementActions => _elementActions;

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
        var swDeserialize = System.Diagnostics.Stopwatch.StartNew();
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
        swDeserialize.Stop();
        Console.Error.WriteLine($"[TIMING] C# JSON deserialization: {swDeserialize.ElapsedMilliseconds}ms ({json.Length} chars)");

        if (result?.Root == null)
            return CreateErrorControl("Empty styled tree from Rust");

        _viewportWidth = result.ViewportWidth;
        _viewportHeight = result.ViewportHeight;

        // CSS background propagation: per spec, if the root element (html) has no
        // explicit background, the body's background covers the canvas/viewport.
        ComputeCanvasBackground(result.Root);

        // Store element selectors for JS event dispatch
        ElementSelectors = result.ElementSelectors;

        var swBuild = System.Diagnostics.Stopwatch.StartNew();
        var control = BuildControl(result.Root, 16.0, 0);
        swBuild.Stop();
        Console.Error.WriteLine($"[TIMING] C# BuildControl (tree construction): {swBuild.ElapsedMilliseconds}ms");
        Console.Error.WriteLine($"[TIMING] C# Total BuildFromJson: {swDeserialize.ElapsedMilliseconds + swBuild.ElapsedMilliseconds}ms");

        return control;
    }

    /// <summary>
    /// Maximum recursion depth for building controls.
    /// Wikipedia's DOM can nest 200+ levels deep, which blows the stack.
    /// </summary>
    private const int MaxBuildDepth = 120;

    /// <summary>
    /// Recursively convert a StyledElement into an Avalonia Control.
    /// parentFontSize is the inherited font size for em/% resolution.
    /// </summary>
    private Control? BuildControl(StyledElement element, double parentFontSize, int depth = 0)
    {
        // Bail out if recursion is too deep to avoid stack overflow on deeply nested DOMs
        if (depth > MaxBuildDepth)
            return null;

        var styles = element.Styles;

        // Skip display:none and visibility:hidden
        if (styles.Display == "none")
            return null;
        if (styles.Visibility == "hidden")
            return null;

        // position: fixed/absolute elements are out-of-flow overlays.
        // Without a proper positioning engine we can't position them correctly,
        // but dropping them entirely loses important content (e.g. site headers).
        // Render them in normal flow instead — at scroll=0 this is approximately
        // correct, and elements that should truly be hidden already have
        // display:none or visibility:hidden.
        if (styles.Position == "fixed" || styles.Position == "absolute")
            styles.Position = null;
        // sticky elements participate in normal flow — render them normally
        if (styles.Position == "sticky")
            styles.Position = null;

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
                return BuildControl(element.Children[0], fontSize, depth + 1);

            // Multiple children: wrap in a transparent StackPanel
            var panel = new StackPanel { Orientation = Orientation.Vertical };
            foreach (var child in element.Children)
            {
                var childControl = BuildControl(child, fontSize, depth + 1);
                if (childControl != null)
                    panel.Children.Add(childControl);
            }
            return panel;
        }

        // Special handling by tag — self-contained elements return early,
        // form elements set specialContent and fall through to Border wrapping
        // so they get CSS border, padding, margin, dimensions applied.
        Control? specialContent = null;
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

            case "input":
                specialContent = BuildInputElement(element, fontSize);
                if (specialContent == null) return null; // hidden input
                break;

            case "button":
                specialContent = BuildButtonElement(element, fontSize);
                break;

            case "select":
                specialContent = BuildSelectElement(element, fontSize);
                break;

            case "textarea":
                specialContent = BuildTextareaElement(element, fontSize);
                break;

            case "svg":
                // Inline SVG: create a sized placeholder panel
                return BuildInlineSvgPlaceholder(element, fontSize);

            case "audio":
                // Audio element: render a placeholder play button
                return BuildAudioPlaceholder(element, fontSize);
        }

        // Determine if this element has only inline children
        bool hasOnlyInlineChildren = element.Children.Count > 0
            && element.Children.All(c => IsInlineElement(c));

        // Build the appropriate control
        Control content;
        if (specialContent != null)
        {
            content = specialContent;
        }
        else if (hasOnlyInlineChildren)
        {
            // Check if this is a "simple link wrapper" — an element whose only non-whitespace
            // content is a single <a> link (e.g., <li><a>Donations</a></li>).
            // SelectableTextBlock with only Inlines (no .Text) doesn't measure properly
            // in horizontal layout contexts (flex items, horizontal StackPanels).
            // Build these as a plain TextBlock with .Text set directly.
            var nonWhitespaceChildren = element.Children
                .Where(c => !(c.Tag == "#text" && string.IsNullOrWhiteSpace(c.TextContent)))
                .ToList();
            var linkChild = (nonWhitespaceChildren.Count == 1 && nonWhitespaceChildren[0].Tag == "a")
                ? nonWhitespaceChildren[0] : null;
            if (linkChild != null)
            {
                var linkText = CollectInlineText(element).Trim();
                if (!string.IsNullOrWhiteSpace(linkText))
                {
                    var linkStyles = linkChild.Styles;
                    var linkColor = StyleParser.ParseBrush(linkStyles.Color)
                        ?? new SolidColorBrush(Color.FromRgb(0, 81, 195));
                    var tb = new TextBlock
                    {
                        Text = linkText,
                        Foreground = linkColor,
                        FontSize = fontSize,
                        FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily ?? linkStyles.FontFamily),
                        FontWeight = StyleParser.ParseFontWeight(styles.FontWeight ?? linkStyles.FontWeight),
                        FontStyle = StyleParser.ParseFontStyle(styles.FontStyle ?? linkStyles.FontStyle),
                        TextWrapping = TextWrapping.NoWrap,
                        VerticalAlignment = VerticalAlignment.Center,
                    };
                    if (linkStyles.TextDecoration != null && linkStyles.TextDecoration != "none")
                        tb.TextDecorations = TextDecorations.Underline;

                    // Register link action for programmatic interaction
                    if (!string.IsNullOrEmpty(linkChild.LinkHref))
                    {
                        var href = linkChild.LinkHref;
                        _elementActions.Register(new ElementActionRegistry.ElementActions
                        {
                            ElementId = linkChild.Id,
                            Tag = linkChild.Tag,
                            TextContent = linkText,
                            Href = href,
                            HasHoverStyles = linkChild.HoverStyles != null,
                            IsLink = true,
                            OnHover = () => _onHoveredLinkChanged?.Invoke(href),
                            OnUnhover = () => _onHoveredLinkChanged?.Invoke(null),
                            OnClick = () =>
                            {
                                _onLinkClicked?.Invoke(href);
                                DispatchDomEvent("click", linkChild.Id);
                            },
                        });
                    }

                    // Return the TextBlock directly — skip the Border wrapper entirely.
                    // Avalonia's RenderTargetBitmap has a rendering issue where TextBlock
                    // text inside deeply nested Borders (10+ levels) doesn't get painted.
                    // By returning the TextBlock directly, we eliminate one layer of nesting
                    // and bypass the Border rendering path. CSS margin is applied directly
                    // to the TextBlock.
                    if (styles.Margin != null)
                    {
                        bool leftAuto = StyleParser.IsAutoMargin(styles.Margin.Left);
                        bool rightAuto = StyleParser.IsAutoMargin(styles.Margin.Right);
                        if (leftAuto && rightAuto)
                        {
                            tb.HorizontalAlignment = HorizontalAlignment.Center;
                            var top = Len(styles.Margin.Top, fontSize) ?? 0;
                            var bottom = Len(styles.Margin.Bottom, fontSize) ?? 0;
                            tb.Margin = new Thickness(0, top, 0, bottom);
                        }
                        else
                        {
                            tb.Margin = Box(styles.Margin, fontSize);
                        }
                    }

                    // Apply link click via pointer events directly on the TextBlock
                    if (!string.IsNullOrEmpty(linkChild.LinkHref))
                    {
                        var href = linkChild.LinkHref;
                        tb.Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand);
                        tb.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(href);
                        tb.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);
                        tb.PointerPressed += (_, e) =>
                        {
                            _onLinkClicked?.Invoke(href);
                            DispatchDomEvent("click", linkChild.Id);
                            e.Handled = true;
                        };
                    }

                    return tb;
                }
                else
                {
                    content = BuildInlineContent(element, fontSize);
                }
            }
            else
            {
                // Standard inline content — SelectableTextBlock with Inlines
                content = BuildInlineContent(element, fontSize);
            }
        }
        else if (element.Children.Count > 0)
        {
            // Skip elements that have no visible content (no text, images, etc.)
            // These are empty structural wrappers (Wiktionary TOC placeholder, etc.)
            // that would create unwanted vertical gaps if rendered.
            if (!HasVisibleContent(element))
                return null;

            // Build a panel with child controls
            content = BuildBlockContent(element, fontSize, depth);
        }
        else if (!string.IsNullOrEmpty(element.TextContent))
        {
            // Leaf element with text content
            content = BuildTextBlock(element, fontSize);
        }
        else
        {
            // Empty element — only render if it has a visible background, border, or explicit dimensions.
            // Many empty divs are structural containers, spacers, or placeholders that shouldn't
            // create visual space when they have no content.
            bool hasVisualBackground = styles.BackgroundColor != null
                && styles.BackgroundColor != "transparent"
                && styles.BackgroundColor != "rgba(0, 0, 0, 0)";
            bool hasBorder = styles.BorderWidth != null
                && (styles.BorderStyle != null && styles.BorderStyle != "none");
            bool hasExplicitSize = styles.Width != null || styles.Height != null;

            if (!hasVisualBackground && !hasBorder && !hasExplicitSize)
                return null;

            content = new Panel();
        }

        // Wrap in Border for background, border, border-radius, padding
        var border = WrapInBorder(content, styles, fontSize);

        // Track actions for registry
        Action? regOnHover = null;
        Action? regOnUnhover = null;
        Action? regOnClick = null;
        bool hasHoverStyles = element.HoverStyles != null;
        bool isLink = element.Tag == "a" && !string.IsNullOrEmpty(element.LinkHref);

        // Apply hover behavior if this element has :hover style overrides
        if (element.HoverStyles != null)
        {
            var (hoverAction, unhoverAction) = AttachHoverBehavior(border, content, styles, element.HoverStyles, fontSize);
            regOnHover = hoverAction;
            regOnUnhover = unhoverAction;
        }

        // For <a> elements rendered as blocks, attach click/hover handlers to the Border.
        // Inline <a> links (Span+Run) can't receive pointer events, but block-level links can.
        if (isLink)
        {
            var (linkHover, linkUnhover, linkClick) = AttachLinkBehavior(border, element);
            // If this element also had hover styles, compose the actions
            if (regOnHover != null)
            {
                var existingHover = regOnHover;
                var existingUnhover = regOnUnhover!;
                regOnHover = () => { existingHover(); linkHover(); };
                regOnUnhover = () => { existingUnhover(); linkUnhover(); };
            }
            else
            {
                regOnHover = linkHover;
                regOnUnhover = linkUnhover;
            }
            regOnClick = linkClick;
        }
        else if (hasHoverStyles)
        {
            // Non-link with hover styles: register a click that dispatches the DOM event
            regOnClick = () => DispatchDomEvent("click", element.Id);
        }

        // Register in the element action registry if interactive
        if (hasHoverStyles || isLink)
        {
            var textContent = CollectInlineText(element);
            _elementActions.Register(new ElementActionRegistry.ElementActions
            {
                ElementId = element.Id,
                Tag = element.Tag,
                TextContent = string.IsNullOrWhiteSpace(textContent) ? null : textContent.Trim(),
                Href = element.LinkHref,
                HasHoverStyles = hasHoverStyles,
                IsLink = isLink,
                OnHover = regOnHover,
                OnUnhover = regOnUnhover,
                OnClick = regOnClick,
            });
        }

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

        // Apply max-width — resolve percentages against viewport width as approximation
        double? maxWidth;
        if (IsPercentage(styles.MaxWidth))
        {
            if (double.TryParse(styles.MaxWidth!.TrimEnd('%', ' '),
                System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var pct))
                maxWidth = pct / 100.0 * _viewportWidth;
            else
                maxWidth = null;
        }
        else
        {
            maxWidth = Len(styles.MaxWidth, fontSize);
        }
        if (maxWidth.HasValue)
            border.MaxWidth = maxWidth.Value;

        // Apply explicit width/height — resolve percentages against viewport width as approximation.
        // Special case: width:100% means "fill parent" in CSS. Setting an explicit pixel
        // value breaks nested elements (e.g., search input getting viewport width instead
        // of flex item width). Use HorizontalAlignment.Stretch for 100%, which lets
        // Avalonia resolve the width relative to the actual parent container.
        double? width;
        if (IsPercentage(styles.Width))
        {
            if (double.TryParse(styles.Width!.TrimEnd('%', ' '),
                System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var wpct))
            {
                if (wpct >= 100)
                {
                    // 100% → stretch to fill parent (don't set explicit pixel value)
                    border.HorizontalAlignment = HorizontalAlignment.Stretch;
                    width = null;
                }
                else
                {
                    // Partial percentages → approximate against viewport
                    width = wpct / 100.0 * _viewportWidth;
                }
            }
            else
                width = null;
        }
        else
        {
            width = Len(styles.Width, fontSize);
        }
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
}
