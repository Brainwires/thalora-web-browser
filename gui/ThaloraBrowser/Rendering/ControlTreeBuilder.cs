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
        // Without a proper positioning engine, rendering them in the normal flow
        // creates incorrect spacing. Skip them — their content is usually
        // duplicated in the normal flow or is non-essential (fixed nav, tooltips).
        // Exception: grid-placed items (grid-area set) with position:absolute are
        // still part of the grid layout — they use absolute positioning relative to
        // their grid cell, not the viewport. Don't skip these.
        if (styles.Position == "fixed" || styles.Position == "absolute")
        {
            if (!string.IsNullOrEmpty(styles.GridArea))
            {
                // Grid-placed element — keep it, just clear the absolute positioning
                // so it renders normally within its grid cell
                styles.Position = null;
            }
            else
            {
                return null;
            }
        }
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

        // Apply explicit width/height — resolve percentages against viewport width as approximation
        double? width;
        if (IsPercentage(styles.Width))
        {
            if (double.TryParse(styles.Width!.TrimEnd('%', ' '),
                System.Globalization.NumberStyles.Float,
                System.Globalization.CultureInfo.InvariantCulture, out var wpct))
            {
                // Resolve all percentage widths against viewport width, including 100%.
                // Setting explicit width ensures child StackPanels get constrained
                // available space instead of infinity, which fixes text rendering in
                // Avalonia's RenderTargetBitmap for deeply nested horizontal layouts.
                width = wpct / 100.0 * _viewportWidth;
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

    /// <summary>
    /// Build a StackPanel (or WrapPanel) containing block-level child controls.
    /// </summary>
    private Control BuildBlockContent(StyledElement element, double fontSize, int depth = 0)
    {
        var styles = element.Styles;
        bool isFlex = styles.Display == "flex" || styles.Display == "inline-flex";
        bool isGrid = styles.Display == "grid" || styles.Display == "inline-grid";

        if (isGrid && (!string.IsNullOrEmpty(styles.GridTemplateColumns)
            || !string.IsNullOrEmpty(styles.GridTemplateAreas)))
        {
            return BuildGridContent(element, fontSize, depth);
        }

        // Fix 5: Navigation menu detection — when a <ul>/<ol> has list-style-type:none
        // AND is a direct child of <nav> (or the element itself is <nav> wrapping a list),
        // it's almost certainly a horizontal navigation menu. Many sites rely on CSS rules
        // like `.nav-list { display: flex; list-style: none }` that our selector engine
        // may not match. Detect this pattern and treat as horizontal flex.
        bool isNavList = !isFlex && !isGrid
            && (element.Tag is "ul" or "ol")
            && styles.ListStyleType == "none";

        Panel panel;

        bool isRow = (isFlex || isNavList) && styles.FlexDirection != "column" && styles.FlexDirection != "column-reverse";
        bool isWrap = (isFlex || isNavList) && (styles.FlexWrap == "wrap" || styles.FlexWrap == "wrap-reverse");

        if (isFlex || isNavList)
        {

            if (isWrap && isRow)
            {
                // Use WrapPanel for flex-wrap in row direction
                var wrapPanel = new WrapPanel
                {
                    Orientation = Orientation.Horizontal,
                };
                panel = wrapPanel;
            }
            else if (isRow)
            {
                // Use Grid with Auto columns for horizontal flex. StackPanel gives
                // children infinite available width which causes Avalonia's
                // RenderTargetBitmap to skip rendering text in deeply nested layouts.
                // Grid with Auto columns arranges children within a concrete width.
                var grid = new Grid();
                grid.HorizontalAlignment = HorizontalAlignment.Stretch;
                panel = grid;
            }
            else
            {
                var stackPanel = new StackPanel
                {
                    Orientation = Orientation.Vertical,
                };
                // Gap
                var gap = Len(styles.Gap, fontSize);
                if (gap.HasValue)
                    stackPanel.Spacing = gap.Value;
                panel = stackPanel;
            }

            // flex-grow: When any child has flex-grow, use Grid for flexible sizing.
            // flex-grow takes priority over justify-content because growing children
            // consume all available space, leaving nothing for justify-content to distribute.
            if (isRow)
            {
                var visibleChildren = element.Children
                    .Where(c => c.Styles.Display != "none")
                    .ToList();

                bool hasFlexGrow = visibleChildren.Any(c =>
                    !string.IsNullOrEmpty(c.Styles.FlexGrow) && c.Styles.FlexGrow != "0");
                bool isSpaceDist = styles.JustifyContent == "space-between"
                    || styles.JustifyContent == "space-around"
                    || styles.JustifyContent == "space-evenly";

                if (hasFlexGrow || isSpaceDist)
                {
                    // Use Grid to handle flex-grow and space distribution.
                    // flex-grow children get Star columns; others get Auto columns.
                    var grid = new Grid();
                    grid.HorizontalAlignment = HorizontalAlignment.Stretch;

                    // Give the Grid a concrete width so Star columns can distribute space.
                    // Resolve percentage widths against viewport (best approximation
                    // without parent layout info). 100% → viewport width.
                    // Subtract padding since the Grid sits inside the Border's padding.
                    double? gridWidth;
                    if (IsPercentage(styles.Width))
                    {
                        if (double.TryParse(styles.Width!.TrimEnd('%', ' '),
                            System.Globalization.NumberStyles.Float,
                            System.Globalization.CultureInfo.InvariantCulture, out var wpct))
                        {
                            var totalW = wpct / 100.0 * _viewportWidth;
                            // Subtract horizontal padding (Grid is inside Border padding)
                            if (styles.Padding != null)
                            {
                                var padL = Len(styles.Padding.Left, fontSize) ?? 0;
                                var padR = Len(styles.Padding.Right, fontSize) ?? 0;
                                totalW -= (padL + padR);
                            }
                            gridWidth = Math.Max(0, totalW);
                        }
                        else
                            gridWidth = null;
                    }
                    else
                    {
                        gridWidth = Len(styles.Width, fontSize);
                    }
                    if (!gridWidth.HasValue)
                        gridWidth = IsPercentage(styles.MaxWidth) ? null : Len(styles.MaxWidth, fontSize);
                    if (gridWidth.HasValue)
                        grid.Width = gridWidth.Value;

                    // Apply cross-axis alignment
                    if (styles.AlignItems == "center")
                        grid.VerticalAlignment = VerticalAlignment.Center;

                    // Gap between items
                    var gap = Len(styles.Gap, fontSize);

                    if (hasFlexGrow)
                    {
                        // flex-grow mode: each child gets a column.
                        // Children with flex-grow > 0 get Star(N) columns.
                        // Children without flex-grow get Auto columns.
                        int col = 0;
                        foreach (var child in visibleChildren)
                        {
                            double grow = 0;
                            if (!string.IsNullOrEmpty(child.Styles.FlexGrow))
                                double.TryParse(child.Styles.FlexGrow, out grow);

                            if (grow > 0)
                                grid.ColumnDefinitions.Add(new ColumnDefinition(grow, GridUnitType.Star));
                            else
                                grid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));

                            var childControl = BuildControl(child, fontSize, depth + 1);
                            if (childControl != null)
                            {
                                if (styles.AlignItems == "center")
                                    childControl.VerticalAlignment = VerticalAlignment.Center;
                                // Apply gap as left margin (except first child)
                                if (col > 0 && gap.HasValue)
                                    childControl.Margin = new Thickness(gap.Value, 0, 0, 0);
                                Grid.SetColumn(childControl, col);
                                grid.Children.Add(childControl);
                            }
                            col++;
                        }
                    }
                    else
                    {
                        // space-between/around/evenly mode (no flex-grow):
                        // Auto columns for children, Star columns for spacers.
                        bool isAround = styles.JustifyContent == "space-around"
                            || styles.JustifyContent == "space-evenly";
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
                            var childControl = BuildControl(child, fontSize, depth + 1);
                            if (childControl != null)
                            {
                                if (styles.AlignItems == "center")
                                    childControl.VerticalAlignment = VerticalAlignment.Center;
                                Grid.SetColumn(childControl, col);
                                grid.Children.Add(childControl);
                            }
                            col += 2;
                        }
                    }

                    return grid;
                }
            }

            // justify-content (only reached when no flex-grow and no space distribution)
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

        // Track list item counter for ordered lists
        bool isList = element.Tag is "ul" or "ol";
        int listItemIndex = 0;

        // Track column index for horizontal flex Grid (default row flex without flex-grow)
        bool isHorizFlexGrid = (isFlex || isNavList) && isRow && panel is Grid && !isWrap;
        int flexGridCol = 0;
        var flexGap = isHorizFlexGrid ? Len(styles.Gap, fontSize) : null;

        foreach (var child in element.Children)
        {
            if (child.Styles.Display == "none")
                continue;

            if (!isFlex && !isNavList && IsInlineElement(child))
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
                    {
                        if (isHorizFlexGrid)
                        {
                            ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
                            Grid.SetColumn(textBlock, flexGridCol++);
                        }
                        panel.Children.Add(textBlock);
                    }
                    inlineBuffer.Clear();
                }

                // Build block child (or blockified flex item)
                var childControl = BuildControl(child, fontSize, depth + 1);
                if (childControl != null)
                {
                    // Wrap <li> children with list markers
                    if (isList && child.Tag == "li"
                        && child.Styles.ListStyleType != "none")
                    {
                        listItemIndex++;
                        childControl = BuildListItemWithMarker(
                            childControl, child, fontSize, listItemIndex);
                    }

                    if (isHorizFlexGrid)
                    {
                        // Add gap column before this item (if not first)
                        if (flexGap.HasValue && flexGridCol > 0)
                        {
                            ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(new GridLength(flexGap.Value)));
                            flexGridCol++;
                        }
                        ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
                        Grid.SetColumn(childControl, flexGridCol++);
                    }
                    panel.Children.Add(childControl);
                }
            }
        }

        // Flush remaining inline children
        if (inlineBuffer.Count > 0)
        {
            var textBlock = BuildInlineGroup(inlineBuffer, fontSize, element);
            if (textBlock != null)
            {
                if (isHorizFlexGrid)
                {
                    ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
                    Grid.SetColumn(textBlock, flexGridCol++);
                }
                panel.Children.Add(textBlock);
            }
        }

        return panel;
    }

    /// <summary>
    /// Build an Avalonia Grid for a CSS Grid container (display: grid).
    /// Handles grid-template-columns, grid-template-rows, grid-template-areas,
    /// and child grid-area placement.
    /// </summary>
    private Control BuildGridContent(StyledElement element, double fontSize, int depth)
    {
        var styles = element.Styles;

        Console.Error.WriteLine($"[GRID] id={element.Id} tag={element.Tag} cols={styles.GridTemplateColumns} rows={styles.GridTemplateRows} areas={styles.GridTemplateAreas?.Substring(0, Math.Min(80, styles.GridTemplateAreas?.Length ?? 0))}");

        var grid = new Grid();
        grid.HorizontalAlignment = HorizontalAlignment.Stretch;

        // Apply width/max-width from CSS so Star columns can distribute space
        var gridWidth = IsPercentage(styles.Width) ? null : Len(styles.Width, fontSize);
        if (gridWidth.HasValue)
            grid.Width = gridWidth.Value;
        var gridMaxW = IsPercentage(styles.MaxWidth) ? null : Len(styles.MaxWidth, fontSize);
        if (gridMaxW.HasValue)
            grid.MaxWidth = gridMaxW.Value;

        // Parse grid-template-columns into Avalonia ColumnDefinitions
        if (!string.IsNullOrEmpty(styles.GridTemplateColumns))
        {
            var columnDefs = ParseGridTemplateColumns(styles.GridTemplateColumns, fontSize);
            foreach (var colDef in columnDefs)
                grid.ColumnDefinitions.Add(colDef);
        }

        // Parse grid-template-rows into Avalonia RowDefinitions
        if (!string.IsNullOrEmpty(styles.GridTemplateRows))
        {
            var rowDefs = ParseGridTemplateRows(styles.GridTemplateRows, fontSize);
            foreach (var rowDef in rowDefs)
                grid.RowDefinitions.Add(rowDef);
        }

        int numCols = grid.ColumnDefinitions.Count;
        if (numCols == 0)
            numCols = 1; // fallback: single column

        // Parse grid-template-areas into an area placement map
        Dictionary<string, (int row, int col, int rowSpan, int colSpan)>? areaMap = null;
        if (!string.IsNullOrEmpty(styles.GridTemplateAreas))
        {
            areaMap = ParseGridTemplateAreas(styles.GridTemplateAreas);
            if (areaMap != null)
                Console.Error.WriteLine($"[GRID] areaMap: {string.Join(", ", areaMap.Select(kv => $"{kv.Key}=({kv.Value.row},{kv.Value.col},{kv.Value.rowSpan},{kv.Value.colSpan})"))}");
        }

        // Apply gap via margin on children (Avalonia Grid doesn't have Spacing)
        var gap = Len(styles.Gap, fontSize);

        var visibleChildren = element.Children
            .Where(c => c.Styles.Display != "none")
            // Skip whitespace-only text nodes — they consume grid cells but render nothing
            .Where(c => !(c.Tag == "#text" && string.IsNullOrWhiteSpace(c.TextContent)))
            .ToList();

        // Track next sequential position for children without grid-area
        int seqCol = 0;
        int seqRow = 0;

        // Ensure we have at least one row if no row definitions were parsed
        if (grid.RowDefinitions.Count == 0)
            grid.RowDefinitions.Add(new RowDefinition(GridLength.Auto));

        foreach (var child in visibleChildren)
        {
            int placedRow, placedCol, rowSpan = 1, colSpan = 1;
            bool hasAreaPlacement = areaMap != null && !string.IsNullOrEmpty(child.Styles.GridArea)
                && areaMap.TryGetValue(child.Styles.GridArea, out var placement);

            // Determine placement BEFORE building — so sequential counter advances
            // even if BuildControl returns null (position:absolute, etc.)
            if (hasAreaPlacement)
            {
                areaMap!.TryGetValue(child.Styles.GridArea!, out var pl);
                placedRow = pl.row;
                placedCol = pl.col;
                rowSpan = pl.rowSpan;
                colSpan = pl.colSpan;
            }
            else
            {
                // Sequential placement: fill columns left-to-right, then wrap to next row
                if (seqCol >= numCols)
                {
                    seqCol = 0;
                    seqRow++;
                }
                placedRow = seqRow;
                placedCol = seqCol;
                seqCol++;
            }

            var childControl = BuildControl(child, fontSize, depth + 1);
            if (childControl == null)
            {
                Console.Error.WriteLine($"[GRID-CHILD-SKIP] id={child.Id} tag={child.Tag} grid_area={child.Styles.GridArea} display={child.Styles.Display} position={child.Styles.Position} visibility={child.Styles.Visibility}");
                continue;
            }

            Console.Error.WriteLine($"[GRID-CHILD] id={child.Id} tag={child.Tag} grid_area={child.Styles.GridArea} placed=({placedRow},{placedCol}) span=({rowSpan},{colSpan})");

            // Ensure enough RowDefinitions exist for the placement
            while (grid.RowDefinitions.Count <= placedRow + rowSpan - 1)
                grid.RowDefinitions.Add(new RowDefinition(GridLength.Auto));

            // Ensure enough ColumnDefinitions exist for the placement
            while (grid.ColumnDefinitions.Count <= placedCol + colSpan - 1)
                grid.ColumnDefinitions.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));

            Grid.SetColumn(childControl, placedCol);
            Grid.SetRow(childControl, placedRow);
            if (colSpan > 1)
                Grid.SetColumnSpan(childControl, colSpan);
            if (rowSpan > 1)
                Grid.SetRowSpan(childControl, rowSpan);

            // Apply gap as margin on children
            if (gap.HasValue)
            {
                var gapHalf = gap.Value / 2.0;
                childControl.Margin = new Thickness(
                    placedCol > 0 ? gapHalf : 0,
                    placedRow > 0 ? gapHalf : 0,
                    placedCol + colSpan < numCols ? gapHalf : 0,
                    0
                );
            }

            grid.Children.Add(childControl);
        }


        return grid;
    }

    /// <summary>
    /// Parse a CSS grid-template-areas value into a map of area name → (row, col, rowSpan, colSpan).
    /// Input format (single quotes): "'siteNotice siteNotice' 'columnStart pageContent' 'footer footer'"
    /// Input format (double quotes from lightningcss): "\"siteNotice siteNotice\"\n\"columnStart pageContent\"\n\"footer footer\""
    /// </summary>
    private static Dictionary<string, (int row, int col, int rowSpan, int colSpan)> ParseGridTemplateAreas(string value)
    {
        var areaMap = new Dictionary<string, (int row, int col, int rowSpan, int colSpan)>();

        // Parse rows: each row is enclosed in single OR double quotes
        var rows = new List<string[]>();
        var rowStart = -1;
        char quoteChar = '\0';
        for (int i = 0; i < value.Length; i++)
        {
            var ch = value[i];
            if ((ch == '\'' || ch == '"') && (rowStart < 0 || ch == quoteChar))
            {
                if (rowStart < 0)
                {
                    rowStart = i + 1; // start of area names
                    quoteChar = ch;
                }
                else
                {
                    // end of area names for this row
                    var rowContent = value[rowStart..i].Trim();
                    rows.Add(rowContent.Split(' ', StringSplitOptions.RemoveEmptyEntries));
                    rowStart = -1;
                    quoteChar = '\0';
                }
            }
        }

        if (rows.Count == 0)
            return areaMap;

        // Build area map: for each unique name, find its bounding rectangle
        var areaNames = new HashSet<string>();
        foreach (var row in rows)
            foreach (var name in row)
                if (name != ".")
                    areaNames.Add(name);

        foreach (var name in areaNames)
        {
            int minRow = int.MaxValue, maxRow = -1;
            int minCol = int.MaxValue, maxCol = -1;

            for (int r = 0; r < rows.Count; r++)
            {
                for (int c = 0; c < rows[r].Length; c++)
                {
                    if (rows[r][c] == name)
                    {
                        if (r < minRow) minRow = r;
                        if (r > maxRow) maxRow = r;
                        if (c < minCol) minCol = c;
                        if (c > maxCol) maxCol = c;
                    }
                }
            }

            if (maxRow >= 0)
            {
                areaMap[name] = (minRow, minCol, maxRow - minRow + 1, maxCol - minCol + 1);
            }
        }

        return areaMap;
    }

    /// <summary>
    /// Parse a CSS grid-template-rows value into Avalonia RowDefinitions.
    /// Handles: min-content → Auto, Nfr → Star, Npx → Pixel, auto → Auto
    /// </summary>
    private List<RowDefinition> ParseGridTemplateRows(string value, double fontSize)
    {
        var defs = new List<RowDefinition>();
        var tokens = TokenizeGridTemplate(value);

        foreach (var token in tokens)
        {
            var trimmed = token.Trim();
            if (string.IsNullOrEmpty(trimmed))
                continue;

            if (trimmed == "auto" || trimmed == "min-content" || trimmed == "max-content")
            {
                defs.Add(new RowDefinition(GridLength.Auto));
            }
            else if (trimmed.EndsWith("fr"))
            {
                var numStr = trimmed[..^2];
                if (double.TryParse(numStr, System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var fr))
                    defs.Add(new RowDefinition(new GridLength(fr, GridUnitType.Star)));
                else
                    defs.Add(new RowDefinition(new GridLength(1, GridUnitType.Star)));
            }
            else if (trimmed.StartsWith("minmax(", StringComparison.OrdinalIgnoreCase))
            {
                // Parse minmax(min, max) — use the max part
                var inner = trimmed[7..].TrimEnd(')');
                var parts = inner.Split(',');
                if (parts.Length == 2)
                {
                    var maxPart = parts[1].Trim();
                    if (maxPart.EndsWith("fr"))
                    {
                        var frStr = maxPart[..^2];
                        if (double.TryParse(frStr, System.Globalization.NumberStyles.Float,
                            System.Globalization.CultureInfo.InvariantCulture, out var fr))
                            defs.Add(new RowDefinition(new GridLength(fr, GridUnitType.Star)));
                        else
                            defs.Add(new RowDefinition(new GridLength(1, GridUnitType.Star)));
                    }
                    else if (maxPart == "auto" || maxPart == "min-content" || maxPart == "max-content")
                    {
                        defs.Add(new RowDefinition(GridLength.Auto));
                    }
                    else
                    {
                        var px = Len(maxPart, fontSize);
                        if (px.HasValue)
                            defs.Add(new RowDefinition(new GridLength(px.Value, GridUnitType.Pixel)));
                        else
                            defs.Add(new RowDefinition(new GridLength(1, GridUnitType.Star)));
                    }
                }
                else
                {
                    defs.Add(new RowDefinition(new GridLength(1, GridUnitType.Star)));
                }
            }
            else
            {
                // Fixed length: px, rem, em, etc.
                var px = Len(trimmed, fontSize);
                if (px.HasValue)
                    defs.Add(new RowDefinition(new GridLength(px.Value, GridUnitType.Pixel)));
                else
                    defs.Add(new RowDefinition(GridLength.Auto));
            }
        }

        return defs;
    }

    /// <summary>
    /// Parse a CSS grid-template-columns value into Avalonia ColumnDefinitions.
    /// Handles: Npx, Nrem, Nem, Nfr, auto, minmax(min, max), and percentage values.
    /// </summary>
    private List<ColumnDefinition> ParseGridTemplateColumns(string value, double fontSize)
    {
        var defs = new List<ColumnDefinition>();
        var tokens = TokenizeGridTemplate(value);

        foreach (var token in tokens)
        {
            var trimmed = token.Trim();
            if (string.IsNullOrEmpty(trimmed))
                continue;

            if (trimmed is "auto" or "min-content" or "max-content")
            {
                defs.Add(new ColumnDefinition(GridLength.Auto));
            }
            else if (trimmed.EndsWith("fr"))
            {
                var numStr = trimmed[..^2];
                if (double.TryParse(numStr, System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var fr))
                    defs.Add(new ColumnDefinition(new GridLength(fr, GridUnitType.Star)));
                else
                    defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
            }
            else if (trimmed.StartsWith("minmax(", StringComparison.OrdinalIgnoreCase))
            {
                // Parse minmax(min, max) — extract the max part
                var inner = trimmed[7..].TrimEnd(')');
                var parts = inner.Split(',');
                if (parts.Length == 2)
                {
                    var maxPart = parts[1].Trim();
                    if (maxPart.EndsWith("fr"))
                    {
                        // minmax(X, Nfr) → Star column
                        var frStr = maxPart[..^2];
                        if (double.TryParse(frStr, System.Globalization.NumberStyles.Float,
                            System.Globalization.CultureInfo.InvariantCulture, out var fr))
                            defs.Add(new ColumnDefinition(new GridLength(fr, GridUnitType.Star)));
                        else
                            defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
                    }
                    else if (maxPart is "auto" or "min-content" or "max-content")
                    {
                        defs.Add(new ColumnDefinition(GridLength.Auto));
                    }
                    else
                    {
                        // minmax(X, Npx) → fixed pixel width from max
                        var px = Len(maxPart, fontSize);
                        if (px.HasValue)
                            defs.Add(new ColumnDefinition(new GridLength(px.Value, GridUnitType.Pixel)));
                        else
                            defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
                    }
                }
                else
                {
                    defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
                }
            }
            else if (trimmed.EndsWith("%"))
            {
                // Percentage → resolve against viewport width
                if (double.TryParse(trimmed.TrimEnd('%', ' '),
                    System.Globalization.NumberStyles.Float,
                    System.Globalization.CultureInfo.InvariantCulture, out var pct))
                    defs.Add(new ColumnDefinition(new GridLength(pct / 100.0 * _viewportWidth, GridUnitType.Pixel)));
                else
                    defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
            }
            else
            {
                // Fixed length: px, rem, em, etc.
                var px = Len(trimmed, fontSize);
                if (px.HasValue)
                    defs.Add(new ColumnDefinition(new GridLength(px.Value, GridUnitType.Pixel)));
                else
                    defs.Add(new ColumnDefinition(new GridLength(1, GridUnitType.Star)));
            }
        }

        return defs;
    }

    /// <summary>
    /// Tokenize a grid-template-columns value into individual column definitions.
    /// Handles minmax() as a single token (doesn't split on commas inside parens).
    /// </summary>
    private static List<string> TokenizeGridTemplate(string value)
    {
        var tokens = new List<string>();
        var current = new System.Text.StringBuilder();
        int parenDepth = 0;

        foreach (var ch in value)
        {
            if (ch == '(')
            {
                parenDepth++;
                current.Append(ch);
            }
            else if (ch == ')')
            {
                parenDepth--;
                current.Append(ch);
                if (parenDepth == 0)
                {
                    tokens.Add(current.ToString().Trim());
                    current.Clear();
                }
            }
            else if (ch == ' ' && parenDepth == 0)
            {
                if (current.Length > 0)
                {
                    tokens.Add(current.ToString().Trim());
                    current.Clear();
                }
            }
            else
            {
                current.Append(ch);
            }
        }

        if (current.Length > 0)
            tokens.Add(current.ToString().Trim());

        return tokens;
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
                        var imgHref = element.LinkHref;
                        displayCtrl.PointerPressed += (_, _) =>
                        {
                            _onLinkClicked?.Invoke(imgHref);
                            DispatchDomEvent("click", element.Id);
                        };
                        displayCtrl.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(imgHref);
                        displayCtrl.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);

                        // Register image link in action registry for programmatic interaction
                        _elementActions.Register(new ElementActionRegistry.ElementActions
                        {
                            ElementId = element.Id,
                            Tag = element.Tag,
                            TextContent = element.ImgAlt ?? CollectInlineText(element),
                            Href = imgHref,
                            HasHoverStyles = element.HoverStyles != null,
                            IsLink = true,
                            OnHover = () => _onHoveredLinkChanged?.Invoke(imgHref),
                            OnUnhover = () => _onHoveredLinkChanged?.Invoke(null),
                            OnClick = () =>
                            {
                                _onLinkClicked?.Invoke(imgHref);
                                DispatchDomEvent("click", element.Id);
                            },
                        });
                    }

                    _ = LoadImageAsync(imgCtrl, imgChild.ImgSrc!);
                    inlines.Add(new InlineUIContainer { Child = displayCtrl });
                    return;
                }

                // Text link — add a styled Run directly. This ensures the text participates
                // in the SelectableTextBlock's text layout and measures correctly.
                // Span wrapping or InlineUIContainer both cause measurement failures
                // in certain contexts (horizontal StackPanels, flex items).
                var linkText = CollectInlineText(element);
                if (string.IsNullOrWhiteSpace(linkText))
                    return;

                var linkColor = StyleParser.ParseBrush(styles.Color)
                    ?? new SolidColorBrush(Color.FromRgb(0, 81, 195)); // #0051C3

                var linkRun = new Run(linkText);
                linkRun.Foreground = linkColor;
                if (styles.FontWeight != null)
                    linkRun.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                if (styles.FontStyle != null)
                    linkRun.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);
                if (styles.FontFamily != null)
                    linkRun.FontFamily = StyleParser.MapToBundledFontFamily(styles.FontFamily);
                if (styles.FontSize != null)
                    linkRun.FontSize = fontSize;
                if (styles.TextDecoration != null && styles.TextDecoration != "none")
                    linkRun.TextDecorations = TextDecorations.Underline;

                inlines.Add(linkRun);

                // Register in action registry for programmatic interaction
                // (click events are dispatched by the parent Border's handler)
                if (!string.IsNullOrEmpty(element.LinkHref))
                {
                    var href = element.LinkHref;
                    _elementActions.Register(new ElementActionRegistry.ElementActions
                    {
                        ElementId = element.Id,
                        Tag = element.Tag,
                        TextContent = linkText.Trim(),
                        Href = href,
                        HasHoverStyles = element.HoverStyles != null,
                        IsLink = true,
                        OnHover = () => _onHoveredLinkChanged?.Invoke(href),
                        OnUnhover = () => _onHoveredLinkChanged?.Invoke(null),
                        OnClick = () =>
                        {
                            _onLinkClicked?.Invoke(href);
                            DispatchDomEvent("click", element.Id);
                        },
                    });
                }

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
    /// Wrap list item content with a marker (bullet, number, etc.)
    /// Creates a horizontal DockPanel with the marker on the left and content filling the rest.
    /// </summary>
    private static Control BuildListItemWithMarker(Control content, StyledElement element, double fontSize, int ordinalIndex)
    {
        var listStyleType = element.Styles.ListStyleType ?? "disc";

        // Determine marker text
        string markerText;
        switch (listStyleType)
        {
            case "disc":
                markerText = "\u2022"; // bullet •
                break;
            case "circle":
                markerText = "\u25E6"; // white bullet ◦
                break;
            case "square":
                markerText = "\u25AA"; // black small square ▪
                break;
            case "decimal":
                markerText = $"{ordinalIndex}.";
                break;
            case "decimal-leading-zero":
                markerText = $"{ordinalIndex:D2}.";
                break;
            case "lower-alpha":
            case "lower-latin":
                markerText = $"{(char)('a' + (ordinalIndex - 1) % 26)}.";
                break;
            case "upper-alpha":
            case "upper-latin":
                markerText = $"{(char)('A' + (ordinalIndex - 1) % 26)}.";
                break;
            case "lower-roman":
                markerText = $"{ToRoman(ordinalIndex).ToLowerInvariant()}.";
                break;
            case "upper-roman":
                markerText = $"{ToRoman(ordinalIndex)}.";
                break;
            default:
                markerText = "\u2022"; // fallback to bullet
                break;
        }

        var marker = new TextBlock
        {
            Text = markerText,
            FontSize = fontSize,
            VerticalAlignment = VerticalAlignment.Top,
            HorizontalAlignment = HorizontalAlignment.Right,
            MinWidth = fontSize * 1.5,
            TextAlignment = TextAlignment.Right,
            Margin = new Thickness(0, 0, 6, 0),
        };

        if (element.Styles.Color != null)
        {
            var brush = StyleParser.ParseBrush(element.Styles.Color);
            if (brush != null)
                marker.Foreground = brush;
        }

        var dock = new DockPanel();
        DockPanel.SetDock(marker, Dock.Left);
        dock.Children.Add(marker);
        dock.Children.Add(content); // fills remaining space (last child in DockPanel)

        return dock;
    }

    /// <summary>
    /// Convert an integer to a Roman numeral string.
    /// </summary>
    private static string ToRoman(int number)
    {
        if (number <= 0 || number > 3999)
            return number.ToString();

        string[] thousands = { "", "M", "MM", "MMM" };
        string[] hundreds = { "", "C", "CC", "CCC", "CD", "D", "DC", "DCC", "DCCC", "CM" };
        string[] tens = { "", "X", "XX", "XXX", "XL", "L", "LX", "LXX", "LXXX", "XC" };
        string[] ones = { "", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX" };

        return thousands[number / 1000]
            + hundreds[(number % 1000) / 100]
            + tens[(number % 100) / 10]
            + ones[number % 10];
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
    /// This handles &lt;a&gt; elements that go through BuildControl (block-level links).
    /// Returns (onHover, onUnhover, onClick) actions for the ElementActionRegistry.
    /// </summary>
    private (Action onHover, Action onUnhover, Action onClick) AttachLinkBehavior(Border border, StyledElement element)
    {
        // Ensure hit-testable: Avalonia skips controls with null Background from pointer hit-testing
        if (border.Background == null)
            border.Background = Avalonia.Media.Brushes.Transparent;

        var href = element.LinkHref!;
        border.Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand);

        Action onClick = () =>
        {
            _onLinkClicked?.Invoke(href);
            DispatchDomEvent("click", element.Id);
        };
        Action onHover = () => _onHoveredLinkChanged?.Invoke(href);
        Action onUnhover = () => _onHoveredLinkChanged?.Invoke(null);

        border.PointerPressed += (_, _) => onClick();
        border.PointerEntered += (_, _) => onHover();
        border.PointerExited += (_, _) => onUnhover();

        return (onHover, onUnhover, onClick);
    }

    /// <summary>
    /// Attach hover behavior to a border+content pair.
    /// On PointerEntered: swap to hover styles. On PointerExited: restore originals.
    /// All brushes are precomputed at build time for instant response.
    /// Returns (onHover, onUnhover) actions for the ElementActionRegistry.
    /// </summary>
    private (Action onHover, Action onUnhover) AttachHoverBehavior(Border border, Control content, ResolvedStyles normalStyles, ResolvedStyles hoverStyles, double fontSize)
    {
        // Ensure hit-testable: Avalonia skips controls with null Background from pointer hit-testing
        if (border.Background == null)
            border.Background = Avalonia.Media.Brushes.Transparent;

        // Precompute normal state — capture from the border itself (which now has Transparent fallback)
        var normalBg = border.Background;
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

        Action onHover = () =>
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

        Action onUnhover = () =>
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

        border.PointerEntered += (_, _) => onHover();
        border.PointerExited += (_, _) => onUnhover();

        return (onHover, onUnhover);
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
            // Guard: Avalonia throws if LineHeight <= 0.
            var computedLh = fontSize * lhMultiplier;
            if (lhMultiplier >= 1.25 && computedLh > 0)
                textBlock.LineHeight = computedLh;
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
            || element.Styles.Display == "table" || element.Styles.Display == "flow-root"
            || element.Styles.Display == "table-row" || element.Styles.Display == "table-cell"
            || element.Styles.Display == "table-row-group" || element.Styles.Display == "table-header-group"
            || element.Styles.Display == "table-footer-group" || element.Styles.Display == "table-caption")
            return false;

        if (element.Styles.Display == "inline" || element.Styles.Display == "inline-block")
            return true;

        // Tag-based classification
        if (AlwaysBlockTags.Contains(element.Tag))
            return false;

        return InlineTags.Contains(element.Tag);
    }

    /// <summary>
    /// Check if an element has any visible content (non-whitespace text, images,
    /// visible backgrounds, explicit dimensions). Used to skip empty structural containers
    /// that would otherwise create unwanted vertical gaps.
    /// </summary>
    private static bool HasVisibleContent(StyledElement element, int depth = 0)
    {
        if (depth > 10) return false; // guard against deep recursion

        // display:none is never visible
        if (element.Styles.Display == "none" || element.Styles.Visibility == "hidden")
            return false;

        // Images are visible
        if (element.Tag == "img" || element.Tag == "svg" || element.Tag == "video" || element.Tag == "canvas")
            return true;

        // Form elements are visible
        if (element.Tag is "input" or "textarea" or "select" or "button")
            return true;

        // Non-whitespace text is visible
        if (!string.IsNullOrWhiteSpace(element.TextContent))
            return true;

        // Element with visible background, border, or explicit dimensions
        var s = element.Styles;
        if (s.BackgroundColor != null && s.BackgroundColor != "transparent" && s.BackgroundColor != "rgba(0, 0, 0, 0)")
            return true;
        if (s.BorderWidth != null && s.BorderStyle != null && s.BorderStyle != "none")
            return true;
        if (s.Width != null || s.Height != null)
            return true;

        // Recursively check children
        foreach (var child in element.Children)
        {
            if (HasVisibleContent(child, depth + 1))
                return true;
        }

        return false;
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
