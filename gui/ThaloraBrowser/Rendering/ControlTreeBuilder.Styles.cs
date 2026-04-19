using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Style application, hover/link behavior, helper methods, and canvas background.
/// Contains WrapInBorder, AttachHoverBehavior, ApplyTextProperties, and utility methods.
/// </summary>
public partial class ControlTreeBuilder
{
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
            border.Background = Brushes.Transparent;

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
            border.Background = Brushes.Transparent;

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
        textBlock.FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily);
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
    private static bool IsInlineElement(StyledElement element)
    {
        // Form controls: always treat as block — they require special rendering
        // (TextBox, CheckBox, Button, etc.) that can't be done as Span/Run inside
        // SelectableTextBlock. Even when CSS says display:inline-block, these must
        // go through BuildControl's tag-specific handlers.
        if (element.Tag is "input" or "button" or "select" or "textarea")
            return false;

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

        // Tag-based classification takes priority over display — AlwaysBlockTags
        // includes tags like "svg" that have inline display by default but require
        // the BuildControl tag-specific path to render correctly.
        if (AlwaysBlockTags.Contains(element.Tag))
            return false;

        // Explicit display overrides (after tag check so svg/etc. aren't redirected)
        if (element.Styles.Display == "block" || element.Styles.Display == "flex"
            || element.Styles.Display == "grid" || element.Styles.Display == "list-item"
            || element.Styles.Display == "table" || element.Styles.Display == "flow-root"
            || element.Styles.Display == "table-row" || element.Styles.Display == "table-cell"
            || element.Styles.Display == "table-row-group" || element.Styles.Display == "table-header-group"
            || element.Styles.Display == "table-footer-group" || element.Styles.Display == "table-caption")
            return false;

        if (element.Styles.Display == "inline" || element.Styles.Display == "inline-block")
            return true;

        return InlineTags.Contains(element.Tag);
    }

    /// <summary>
    /// Check if a CSS value is a percentage (e.g., "100%", "50%").
    /// Percentage widths/heights can't be resolved without parent size,
    /// so we skip them and let Avalonia's layout handle it.
    /// </summary>
    private static bool IsPercentage(string? value)
        => value != null && value.TrimEnd().EndsWith('%');

    /// <summary>
    /// Check if a flex child has CSS width >= 100%, meaning it wants to fill its parent.
    /// In horizontal flex Grids, these children need Star columns instead of Auto
    /// because BuildControl translates width:100% to HorizontalAlignment.Stretch
    /// (no explicit pixel width), and Stretch in an Auto column = content width.
    /// </summary>
    private static bool IsChildFullWidth(StyledElement child)
    {
        var w = child.Styles.Width?.TrimEnd();
        if (w == null || !w.EndsWith('%'))
            return false;
        if (double.TryParse(w.TrimEnd('%', ' '),
            System.Globalization.NumberStyles.Float,
            System.Globalization.CultureInfo.InvariantCulture,
            out var pct))
        {
            return pct >= 100;
        }
        return false;
    }

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

    /// <summary>
    /// If an inline element tree contains exactly one image element with an img_src
    /// (and no meaningful text), return that image element. Otherwise return null.
    /// Used to render image-only table cells as block Image controls instead of
    /// InlineUIContainer (which fails to render in Avalonia).
    /// </summary>
    private static StyledElement? FindSingleImageElement(StyledElement element, int depth = 0)
    {
        if (depth > 8) return null;
        if (element.Styles.Display == "none") return null;

        // If this IS an img with src, return it
        if (element.Tag == "img" && !string.IsNullOrEmpty(element.ImgSrc))
            return element;

        // If this element has text content (not just whitespace), it's not image-only
        if (!string.IsNullOrWhiteSpace(element.TextContent))
            return null;

        StyledElement? foundImg = null;
        foreach (var child in element.Children)
        {
            if (child.Styles.Display == "none") continue;
            // Skip pure whitespace text nodes
            if (child.Tag == "#text" && string.IsNullOrWhiteSpace(child.TextContent)) continue;
            // Non-whitespace text means it's not image-only
            if (child.Tag == "#text" && !string.IsNullOrWhiteSpace(child.TextContent)) return null;

            var childImg = FindSingleImageElement(child, depth + 1);
            if (childImg == null) return null; // non-image content found
            if (foundImg != null) return null;  // multiple images
            foundImg = childImg;
        }
        return foundImg; // may be null if no children
    }

    /// <summary>
    /// Check if an element has any visible content (non-whitespace text, images,
    /// visible backgrounds, explicit dimensions). Used to skip empty structural containers
    /// that would otherwise create unwanted vertical gaps.
    /// </summary>
    private static bool HasVisibleContent(StyledElement element, int depth = 0)
    {
        if (depth > 60) return false; // guard against deep recursion (GitHub content sits at depth 17+)

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
    /// Override the Fluent theme's per-state resources on a TextBox so focus/pointer-over
    /// don't swap in the theme's dark default background (the app runs under the Dark
    /// variant, which would otherwise paint focused inputs black on light web pages).
    /// </summary>
    private static void ApplyTextBoxStateResources(TextBox textBox, IBrush background, IBrush foreground)
    {
        var r = textBox.Resources;
        r["TextControlBackground"] = background;
        r["TextControlBackgroundPointerOver"] = background;
        r["TextControlBackgroundFocused"] = background;
        r["TextControlBackgroundDisabled"] = background;
        r["TextControlForeground"] = foreground;
        r["TextControlForegroundPointerOver"] = foreground;
        r["TextControlForegroundFocused"] = foreground;
        r["TextControlForegroundDisabled"] = foreground;
        r["TextControlBorderBrush"] = Brushes.Transparent;
        r["TextControlBorderBrushPointerOver"] = Brushes.Transparent;
        r["TextControlBorderBrushFocused"] = Brushes.Transparent;
        r["TextControlBorderBrushDisabled"] = Brushes.Transparent;
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
