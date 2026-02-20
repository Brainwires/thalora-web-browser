using AngleSharp.Dom;
using Avalonia;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// CSS box-model layout engine. Converts a styled DOM tree into a positioned
/// LayoutBox tree ready for painting.
///
/// Supports:
/// - Block formatting context (vertical stacking)
/// - Inline formatting context (horizontal text flow with word wrapping)
/// - Box model (margin, border, padding, content)
/// - Tables (basic grid layout)
/// - Lists (disc/decimal markers)
/// </summary>
public class LayoutEngine
{
    private readonly StyleResolver _styleResolver;

    // Tags that should be skipped entirely during layout
    private static readonly HashSet<string> SkipTags = new(StringComparer.OrdinalIgnoreCase)
    {
        "script", "style", "link", "meta", "head", "title", "noscript", "template",
    };

    public LayoutEngine(StyleResolver styleResolver)
    {
        _styleResolver = styleResolver;
    }

    /// <summary>
    /// Build a full layout tree from a DOM document for the given viewport.
    /// </summary>
    public LayoutBox BuildLayoutTree(IDocument document, Size viewport)
    {
        var body = document.Body ?? document.DocumentElement;
        if (body == null)
        {
            return new LayoutBox
            {
                Type = BoxType.Block,
                Style = _styleResolver.CreateRootStyle(),
                ContentRect = new Rect(0, 0, viewport.Width, viewport.Height),
            };
        }

        var rootStyle = _styleResolver.CreateRootStyle();
        var rootBox = new LayoutBox
        {
            Element = body as IElement,
            Type = BoxType.Block,
            Style = rootStyle,
        };

        // Build children from DOM
        BuildChildren(body, rootBox, rootStyle);

        // Perform layout pass
        PerformLayout(rootBox, viewport.Width, 0, 0);

        return rootBox;
    }

    /// <summary>
    /// Recursively build LayoutBox children from DOM child nodes.
    /// </summary>
    private void BuildChildren(INode parentNode, LayoutBox parentBox, CssComputedStyle parentStyle)
    {
        foreach (var child in parentNode.ChildNodes)
        {
            if (child is IText textNode)
            {
                var text = textNode.Data;
                if (string.IsNullOrWhiteSpace(text) && parentStyle.WhiteSpace == WhiteSpaceMode.Normal)
                    continue;

                // Create an anonymous inline box for text
                var textBox = new LayoutBox
                {
                    Type = BoxType.Anonymous,
                    Style = parentStyle,
                    TextRuns = new List<TextRun>(),
                };

                // Actual text runs will be created during layout when we know the available width
                textBox.TextRuns.Add(new TextRun
                {
                    Text = text,
                    Style = parentStyle,
                    LinkHref = FindAncestorHref(parentNode),
                });

                parentBox.Children.Add(textBox);
            }
            else if (child is IElement element)
            {
                var tagName = element.LocalName.ToLowerInvariant();

                // Skip invisible/metadata elements
                if (SkipTags.Contains(tagName))
                    continue;

                var style = _styleResolver.ComputeStyle(element, parentStyle);

                // Skip display:none elements
                if (!style.IsVisible || style.Display == DisplayMode.None)
                    continue;

                var boxType = MapDisplayToBoxType(style.Display);
                var box = new LayoutBox
                {
                    Element = element,
                    Type = boxType,
                    Style = style,
                    Margin = style.Margin,
                    Border = style.BorderWidth,
                    Padding = style.Padding,
                };

                // Handle special elements
                if (tagName == "img")
                {
                    box.ImageSource = element.GetAttribute("src");
                    box.Type = BoxType.InlineBlock;
                }

                if (tagName == "a")
                {
                    box.LinkHref = element.GetAttribute("href");
                }

                if (tagName == "br")
                {
                    // <br> is a line break — create a special text run
                    box.Type = BoxType.Anonymous;
                    box.TextRuns = new List<TextRun>
                    {
                        new TextRun { Text = "\n", Style = parentStyle }
                    };
                    parentBox.Children.Add(box);
                    continue;
                }

                // Propagate link href to children
                if (box.LinkHref != null)
                    PropagateLink(element, box, style, box.LinkHref);
                else
                    BuildChildren(element, box, style);

                parentBox.Children.Add(box);
            }
        }
    }

    /// <summary>
    /// Build children with a link href propagated through all text content.
    /// </summary>
    private void PropagateLink(INode parentNode, LayoutBox parentBox, CssComputedStyle parentStyle, string href)
    {
        foreach (var child in parentNode.ChildNodes)
        {
            if (child is IText textNode)
            {
                var text = textNode.Data;
                if (string.IsNullOrWhiteSpace(text) && parentStyle.WhiteSpace == WhiteSpaceMode.Normal)
                    continue;

                var textBox = new LayoutBox
                {
                    Type = BoxType.Anonymous,
                    Style = parentStyle,
                    LinkHref = href,
                    TextRuns = new List<TextRun>
                    {
                        new TextRun { Text = text, Style = parentStyle, LinkHref = href }
                    },
                };
                parentBox.Children.Add(textBox);
            }
            else if (child is IElement element)
            {
                var tagName = element.LocalName.ToLowerInvariant();
                if (SkipTags.Contains(tagName)) continue;

                var style = _styleResolver.ComputeStyle(element, parentStyle);
                if (!style.IsVisible || style.Display == DisplayMode.None) continue;

                var boxType = MapDisplayToBoxType(style.Display);
                var box = new LayoutBox
                {
                    Element = element,
                    Type = boxType,
                    Style = style,
                    LinkHref = href,
                    Margin = style.Margin,
                    Border = style.BorderWidth,
                    Padding = style.Padding,
                };

                PropagateLink(element, box, style, href);
                parentBox.Children.Add(box);
            }
        }
    }

    /// <summary>
    /// Perform the layout pass: compute positions and sizes for all boxes.
    /// </summary>
    private void PerformLayout(LayoutBox box, double containingWidth, double x, double y)
    {
        // Compute content width
        double contentWidth = containingWidth - box.Margin.Left - box.Margin.Right
            - box.Border.Left - box.Border.Right
            - box.Padding.Left - box.Padding.Right;

        if (box.Style.Width.HasValue)
            contentWidth = Math.Min(contentWidth, box.Style.Width.Value);
        if (box.Style.MaxWidth.HasValue)
            contentWidth = Math.Min(contentWidth, box.Style.MaxWidth.Value);
        if (box.Style.MinWidth.HasValue)
            contentWidth = Math.Max(contentWidth, box.Style.MinWidth.Value);

        contentWidth = Math.Max(0, contentWidth);

        // Content start position
        double contentX = x + box.Margin.Left + box.Border.Left + box.Padding.Left;
        double contentY = y + box.Margin.Top + box.Border.Top + box.Padding.Top;

        if (box.Type == BoxType.TableBox)
        {
            LayoutTable(box, contentWidth, contentX, contentY);
        }
        else if (box.Type == BoxType.Block || box.Type == BoxType.ListItem || box.Type == BoxType.Inline)
        {
            LayoutBlockOrInline(box, contentWidth, contentX, contentY);
        }
        else if (box.Type == BoxType.Anonymous && box.TextRuns?.Count > 0)
        {
            LayoutTextBox(box, contentWidth, contentX, contentY);
        }
        else
        {
            // Default: lay out children as block
            LayoutBlockOrInline(box, contentWidth, contentX, contentY);
        }
    }

    /// <summary>
    /// Layout children in block or inline formatting context.
    /// </summary>
    private void LayoutBlockOrInline(LayoutBox box, double contentWidth, double contentX, double contentY)
    {
        double currentY = contentY;
        double maxChildWidth = 0;

        // Determine if this box establishes a block or inline formatting context
        bool hasBlockChildren = box.Children.Any(c =>
            c.Type == BoxType.Block || c.Type == BoxType.ListItem || c.Type == BoxType.TableBox);

        if (hasBlockChildren)
        {
            // Block formatting context: stack children vertically
            foreach (var child in box.Children)
            {
                PerformLayout(child, contentWidth, contentX, currentY);

                var childHeight = child.MarginBox.Height;
                currentY += childHeight;
                maxChildWidth = Math.Max(maxChildWidth, child.MarginBox.Width);
            }
        }
        else
        {
            // Inline formatting context: flow children horizontally with wrapping
            double lineX = contentX;
            double lineHeight = 0;
            double lineY = currentY;

            foreach (var child in box.Children)
            {
                if (child.Type == BoxType.Anonymous && child.TextRuns?.Count > 0)
                {
                    // Text node — break into positioned runs
                    var originalRun = child.TextRuns[0];
                    var text = originalRun.Text;

                    // Check for line break
                    if (text == "\n")
                    {
                        lineX = contentX;
                        lineY += Math.Max(lineHeight, child.Style.FontSize * child.Style.LineHeight);
                        lineHeight = 0;
                        continue;
                    }

                    var runs = TextLayoutEngine.BreakIntoLines(
                        text,
                        child.Style,
                        contentWidth,
                        lineX - contentX,
                        lineY,
                        originalRun.LinkHref
                    );

                    child.TextRuns = runs;

                    foreach (var run in runs)
                    {
                        var runRight = run.Bounds.X + run.Bounds.Width;
                        maxChildWidth = Math.Max(maxChildWidth, runRight - contentX);

                        // Track line position
                        if (run.Bounds.X <= contentX + 1) // New line started
                        {
                            if (lineHeight > 0)
                            {
                                // Finished previous line
                            }
                            lineHeight = run.Bounds.Height;
                        }
                        else
                        {
                            lineHeight = Math.Max(lineHeight, run.Bounds.Height);
                        }

                        lineX = run.Bounds.X + run.Bounds.Width;
                    }

                    if (runs.Count > 0)
                    {
                        var lastRun = runs[^1];
                        lineY = lastRun.Bounds.Y;
                        lineX = lastRun.Bounds.X + lastRun.Bounds.Width;
                        lineHeight = lastRun.Bounds.Height;

                        child.ContentRect = new Rect(
                            runs[0].Bounds.X,
                            runs[0].Bounds.Y,
                            runs.Max(r => r.Bounds.Right) - runs[0].Bounds.X,
                            lastRun.Bounds.Bottom - runs[0].Bounds.Y
                        );
                    }
                }
                else
                {
                    // Inline element — lay out recursively
                    PerformLayout(child, contentWidth - (lineX - contentX), lineX, lineY);

                    var childWidth = child.MarginBox.Width;
                    if (lineX + childWidth > contentX + contentWidth && lineX > contentX)
                    {
                        // Wrap to next line
                        lineX = contentX;
                        lineY += lineHeight;
                        lineHeight = 0;
                        PerformLayout(child, contentWidth, lineX, lineY);
                    }

                    lineX += child.MarginBox.Width;
                    lineHeight = Math.Max(lineHeight, child.MarginBox.Height);
                    maxChildWidth = Math.Max(maxChildWidth, lineX - contentX);
                }
            }

            currentY = lineY + lineHeight;
        }

        // Set the content rect
        double totalHeight = box.Style.Height ?? (currentY - contentY);
        box.ContentRect = new Rect(contentX, contentY, contentWidth, Math.Max(0, totalHeight));
    }

    /// <summary>
    /// Layout a text-only box (anonymous box with text runs).
    /// </summary>
    private void LayoutTextBox(LayoutBox box, double contentWidth, double contentX, double contentY)
    {
        if (box.TextRuns == null || box.TextRuns.Count == 0) return;

        var firstRun = box.TextRuns[0];
        var size = TextLayoutEngine.MeasureText(firstRun.Text, firstRun.Style);
        var lineHeight = firstRun.Style.FontSize * firstRun.Style.LineHeight;

        box.ContentRect = new Rect(contentX, contentY, size.Width, lineHeight);
        firstRun.Bounds = box.ContentRect;
    }

    /// <summary>
    /// Basic table layout — distributes column widths evenly.
    /// </summary>
    private void LayoutTable(LayoutBox table, double contentWidth, double contentX, double contentY)
    {
        // Count max columns
        int maxCols = 0;
        foreach (var row in table.Children)
        {
            if (row.Type == BoxType.TableRowBox)
                maxCols = Math.Max(maxCols, row.Children.Count);
        }

        if (maxCols == 0) maxCols = 1;
        double colWidth = contentWidth / maxCols;
        double currentY = contentY;

        foreach (var row in table.Children)
        {
            if (row.Type != BoxType.TableRowBox)
            {
                PerformLayout(row, contentWidth, contentX, currentY);
                currentY += row.MarginBox.Height;
                continue;
            }

            double rowHeight = 0;
            double cellX = contentX;

            for (int i = 0; i < row.Children.Count; i++)
            {
                var cell = row.Children[i];
                PerformLayout(cell, colWidth, cellX, currentY);
                rowHeight = Math.Max(rowHeight, cell.MarginBox.Height);
                cellX += colWidth;
            }

            row.ContentRect = new Rect(contentX, currentY, contentWidth, rowHeight);
            currentY += rowHeight;
        }

        table.ContentRect = new Rect(contentX, contentY, contentWidth, currentY - contentY);
    }

    internal static BoxType MapDisplayToBoxType(DisplayMode display) => display switch
    {
        DisplayMode.Block => BoxType.Block,
        DisplayMode.Inline => BoxType.Inline,
        DisplayMode.InlineBlock => BoxType.InlineBlock,
        DisplayMode.Flex => BoxType.Block, // Treat flex as block for now
        DisplayMode.ListItem => BoxType.ListItem,
        DisplayMode.Table => BoxType.TableBox,
        DisplayMode.TableRow => BoxType.TableRowBox,
        DisplayMode.TableCell => BoxType.TableCellBox,
        _ => BoxType.Block,
    };

    private static string? FindAncestorHref(INode? node)
    {
        while (node != null)
        {
            if (node is IElement el && el.LocalName.Equals("a", StringComparison.OrdinalIgnoreCase))
                return el.GetAttribute("href");
            node = node.Parent;
        }
        return null;
    }
}
