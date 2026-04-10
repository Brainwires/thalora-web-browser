using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Layout;
using Avalonia.Media;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Inline content building — SelectableTextBlock with Inlines (Runs, Spans, Bold, etc.)
/// for elements whose children are all inline.
/// </summary>
public partial class ControlTreeBuilder
{
    /// <summary>
    /// Build a SelectableTextBlock with Inlines for a block element
    /// whose children are all inline.
    /// </summary>
    private Control BuildInlineContent(StyledElement element, double fontSize)
    {
        var textBlock = new SelectableTextBlock();
        // Center vertically so text sits at the middle of its cell in flex/grid rows
        // (matches CSS default where items stretch and text appears visually centered)
        textBlock.VerticalAlignment = VerticalAlignment.Center;
        ApplyTextProperties(textBlock, element.Styles, fontSize);

        foreach (var child in element.Children)
        {
            AddInlineContent(textBlock.Inlines!, child, fontSize);
        }

        return textBlock;
    }

    /// <summary>
    /// Build a SelectableTextBlock from a group of consecutive inline elements.
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

                text = ApplyTextTransform(text, styles.TextTransform);
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
                    run.FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily);
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

                // Text link — render as Run (participates in text layout) unless the
                // element has margin or padding, in which case use InlineUIContainer+Border
                // so the spacing is preserved. (Run has no Margin/Padding support.)
                var linkText = CollectInlineText(element);
                if (string.IsNullOrWhiteSpace(linkText))
                    return;

                var linkColor = StyleParser.ParseBrush(styles.Color)
                    ?? new SolidColorBrush(Avalonia.Media.Color.FromRgb(0, 81, 195)); // #0051C3

                var margin = StyleParser.ParseBoxSides(styles.Margin, fontSize, fontSize, _viewportWidth, _viewportHeight);
                var padding = StyleParser.ParseBoxSides(styles.Padding, fontSize, fontSize, _viewportWidth, _viewportHeight);
                bool hasSpacing = margin != default || padding != default;

                if (hasSpacing)
                {
                    // Use InlineUIContainer+Border so margin/padding apply
                    var linkBlock = new TextBlock
                    {
                        Text = linkText,
                        Foreground = linkColor,
                        FontSize = fontSize,
                    };
                    if (styles.FontWeight != null)
                        linkBlock.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                    if (styles.FontFamily != null)
                        linkBlock.FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily);
                    if (styles.TextDecoration != null && styles.TextDecoration != "none")
                        linkBlock.TextDecorations = TextDecorations.Underline;

                    var linkBorder = new Border
                    {
                        Child = linkBlock,
                        Margin = margin,
                        Padding = padding,
                        Background = Brushes.Transparent,
                        Cursor = new Avalonia.Input.Cursor(Avalonia.Input.StandardCursorType.Hand),
                    };

                    if (!string.IsNullOrEmpty(element.LinkHref))
                    {
                        var href2 = element.LinkHref;
                        linkBorder.PointerPressed += (_, _) =>
                        {
                            _onLinkClicked?.Invoke(href2);
                            DispatchDomEvent("click", element.Id);
                        };
                        linkBorder.PointerEntered += (_, _) => _onHoveredLinkChanged?.Invoke(href2);
                        linkBorder.PointerExited += (_, _) => _onHoveredLinkChanged?.Invoke(null);
                    }

                    inlines.Add(new InlineUIContainer { Child = linkBorder });
                }
                else
                {
                    var linkRun = new Run(linkText);
                    linkRun.Foreground = linkColor;
                    if (styles.FontWeight != null)
                        linkRun.FontWeight = StyleParser.ParseFontWeight(styles.FontWeight);
                    if (styles.FontStyle != null)
                        linkRun.FontStyle = StyleParser.ParseFontStyle(styles.FontStyle);
                    if (styles.FontFamily != null)
                        linkRun.FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily);
                    if (styles.FontSize != null)
                        linkRun.FontSize = fontSize;
                    if (styles.TextDecoration != null && styles.TextDecoration != "none")
                        linkRun.TextDecorations = TextDecorations.Underline;

                    inlines.Add(linkRun);
                }

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

            case "input":
            case "button":
            case "select":
            case "textarea":
            {
                // Form controls can't be rendered as Span/Run — build the real Avalonia
                // control and embed it as an InlineUIContainer so it appears in line.
                var formCtrl = BuildControl(element, parentFontSize);
                if (formCtrl != null)
                    inlines.Add(new InlineUIContainer { Child = formCtrl });
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
                    span.FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily);
                if (styles.FontSize != null)
                    span.FontSize = fontSize;

                foreach (var child in element.Children)
                    AddInlineContent(span.Inlines, child, fontSize);
                if (element.Children.Count == 0 && !string.IsNullOrEmpty(element.TextContent))
                    span.Inlines.Add(new Run(element.TextContent));

                inlines.Add(span);
                return;
            }
        }
    }
}
