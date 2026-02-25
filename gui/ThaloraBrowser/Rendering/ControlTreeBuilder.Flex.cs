using Avalonia.Controls;
using Avalonia.Layout;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Flex layout and block content building.
/// Contains BuildBlockContent — the main method for building panels/grids
/// from block-level and flex children.
/// </summary>
public partial class ControlTreeBuilder
{
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
                    // CSS spec: whitespace-only text nodes are collapsed in flex
                    .Where(c => !(c.Tag == "#text" && string.IsNullOrWhiteSpace(c.TextContent)))
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
                                    childControl.Margin = new Avalonia.Thickness(gap.Value, 0, 0, 0);
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

            // Skip out-of-flow elements early — position:absolute/fixed are removed
            // from normal flow and should not enter inline buffers or flex grids.
            // BuildControl would also skip them, but catching them here prevents
            // absolute-positioned <input> elements from polluting inline groups.
            if (child.Styles.Position is "absolute" or "fixed")
                continue;

            // CSS spec: In flex containers, whitespace-only text nodes are collapsed
            // (they don't become flex items). Skip them to avoid wasting Auto columns
            // in the horizontal Grid and creating unwanted gaps.
            if ((isFlex || isNavList) && child.Tag == "#text"
                && string.IsNullOrWhiteSpace(child.TextContent))
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
}
