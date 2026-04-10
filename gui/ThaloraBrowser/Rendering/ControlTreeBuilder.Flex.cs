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
    private Control BuildBlockContent(StyledElement element, double fontSize, int depth = 0, double availableWidth = 0)
    {
        var styles = element.Styles;
        bool isFlex = styles.Display == "flex" || styles.Display == "inline-flex";
        bool isGrid = styles.Display == "grid" || styles.Display == "inline-grid";

        if (isGrid && (!string.IsNullOrEmpty(styles.GridTemplateColumns)
            || !string.IsNullOrEmpty(styles.GridTemplateAreas)))
        {
            return BuildGridContent(element, fontSize, depth, availableWidth);
        }

        // Float simulation: when block children have float:right or float:left,
        // arrange them in a two/three-column Grid beside normal-flow content.
        // This handles common patterns like Wikipedia's infobox (float:right) and TOC (float:left).
        // Only applies when both floated and normal-flow children coexist.
        if (!isFlex && !isGrid)
        {
            var floatRight = element.Children
                .Where(c => c.Styles.Float == "right" && c.Styles.Display != "none")
                .ToList();
            var floatLeft = element.Children
                .Where(c => c.Styles.Float == "left" && c.Styles.Display != "none")
                .ToList();
            var normalFlow = element.Children
                .Where(c => c.Styles.Float != "right" && c.Styles.Float != "left"
                         && c.Styles.Display != "none")
                .ToList();

            if ((floatRight.Any() || floatLeft.Any()) && normalFlow.Any())
            {
                return BuildFloatLayout(element, normalFlow, floatLeft, floatRight, fontSize, depth, availableWidth);
            }
        }

        // Navigation menu detection: horizontal layout for ul/ol with list-style:none.
        // Only applies when the list is SHORT (≤8 items) — content TOCs like Wikipedia's
        // have 13+ items and must remain vertical. Nav menus are typically concise.
        // Also only applies when display is not explicitly set to block, OR when all li
        // children have float:left (classic CSS horizontal nav pattern: "ul li { float:left }").
        bool allLisFloatLeft = (element.Tag is "ul" or "ol")
            && element.Children.Any(c => c.Tag == "li")
            && element.Children.Where(c => c.Tag == "li").All(c => c.Styles.Float == "left");
        bool isNavList = !isFlex && !isGrid
            && (element.Tag is "ul" or "ol")
            && styles.ListStyleType == "none"
            && (styles.Display != "block" || allLisFloatLeft)
            && element.Children.Count(c => c.Tag == "li") <= 8;

        // <tr> with display:table-row renders its <td>/<th> children horizontally.
        // Without this, table cells would stack vertically instead of side-by-side.
        bool isTableRow = styles.Display == "table-row";

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

                // When flex-wrap is enabled, don't use the single-row Grid shortcut.
                // Wrapping containers need WrapPanel to flow items onto multiple rows
                // when children exceed the container width (e.g., first child width:100%
                // takes a full row, remaining children wrap to the next row).
                if (!isWrap && (hasFlexGrow || isSpaceDist))
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
                        // Exception: children with width >= 100% also get Star columns
                        // (CSS width:100% = fill parent, but Auto columns constrain to content).
                        int col = 0;
                        foreach (var child in visibleChildren)
                        {
                            double grow = 0;
                            if (!string.IsNullOrEmpty(child.Styles.FlexGrow))
                                double.TryParse(child.Styles.FlexGrow, out grow);

                            if (grow > 0)
                                grid.ColumnDefinitions.Add(new ColumnDefinition(grow, GridUnitType.Star));
                            else if (IsChildFullWidth(child))
                                grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));
                            else
                                grid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));

                            var childControl = BuildControl(child, fontSize, depth + 1, availableWidth);
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
                        // Exception: children with width >= 100% get Star columns so they
                        // can stretch to fill the parent (CSS width:100% in Auto columns
                        // constrains to content width, breaking inner justify-content).
                        bool isAround = styles.JustifyContent == "space-around"
                            || styles.JustifyContent == "space-evenly";
                        for (int i = 0; i < visibleChildren.Count; i++)
                        {
                            if (i > 0 || isAround)
                                grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));
                            if (IsChildFullWidth(visibleChildren[i]))
                                grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));
                            else
                                grid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
                        }
                        if (isAround)
                            grid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));

                        int col = isAround ? 1 : 0;
                        foreach (var child in visibleChildren)
                        {
                            var childControl = BuildControl(child, fontSize, depth + 1, availableWidth);
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
        else if (isTableRow)
        {
            // <tr>: render <td>/<th> children as horizontal columns in a Grid.
            // Each cell gets an Auto column — cells size to their content width.
            var grid = new Grid();
            grid.HorizontalAlignment = HorizontalAlignment.Stretch;
            panel = grid;
        }
        else
        {
            panel = new StackPanel { Orientation = Orientation.Vertical };
        }

        // For flex-wrap containers, compute the effective inner width so we can
        // resolve percentage widths on children correctly. In a WrapPanel, children
        // with width:100% need an explicit pixel width (not HorizontalAlignment.Stretch)
        // to ensure they fill the row and force subsequent children to wrap.
        double wrapContainerWidth = 0;
        if (isWrap && isRow)
        {
            var mw = IsPercentage(styles.MaxWidth) ? (double?)null : Len(styles.MaxWidth, fontSize);
            var w = IsPercentage(styles.Width) ? (double?)null : Len(styles.Width, fontSize);
            wrapContainerWidth = mw ?? w ?? _viewportWidth;
            if (styles.Padding != null)
            {
                var padL = Len(styles.Padding.Left, fontSize) ?? 0;
                var padR = Len(styles.Padding.Right, fontSize) ?? 0;
                wrapContainerWidth -= (padL + padR);
            }
            wrapContainerWidth = Math.Max(0, wrapContainerWidth);
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
        bool isHorizFlexGrid = ((isFlex || isNavList) && isRow && panel is Grid && !isWrap) || isTableRow;
        int flexGridCol = 0;
        var flexGap = isHorizFlexGrid ? Len(styles.Gap, fontSize) : null;

        foreach (var child in element.Children)
        {
            if (child.Styles.Display == "none")
                continue;

            // CSS spec: In flex containers, whitespace-only text nodes are collapsed
            // (they don't become flex items). Skip them to avoid wasting Auto columns
            // in the horizontal Grid and creating unwanted gaps.
            if ((isFlex || isNavList) && child.Tag == "#text"
                && string.IsNullOrWhiteSpace(child.TextContent))
                continue;

            if (!isFlex && !isNavList && !isTableRow && IsInlineElement(child))
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
                var childControl = BuildControl(child, fontSize, depth + 1, availableWidth);
                if (childControl != null)
                {
                    // For flex-wrap children, override percentage widths to resolve
                    // against the container's effective width instead of the viewport.
                    // BuildControl resolves width:100% as HorizontalAlignment.Stretch
                    // (no explicit pixel width), but WrapPanel needs explicit widths
                    // to determine which row each child belongs to.
                    if (isWrap && isRow && wrapContainerWidth > 0
                        && child.Styles.Width != null)
                    {
                        var wStr = child.Styles.Width.TrimEnd();
                        if (wStr.EndsWith('%')
                            && double.TryParse(wStr.TrimEnd('%', ' '),
                                System.Globalization.NumberStyles.Float,
                                System.Globalization.CultureInfo.InvariantCulture,
                                out var wpct))
                        {
                            var resolvedWidth = wpct / 100.0 * wrapContainerWidth;
                            if (childControl is Border childBorder)
                            {
                                childBorder.Width = resolvedWidth;
                                // Don't hardcode HorizontalAlignment.Left here — the explicit
                                // Width is sufficient for WrapPanel row-breaking. Leave default
                                // (Stretch) so the child's internal alignment properties
                                // (text-align:right, justify-content:flex-end) work correctly.
                            }
                            else
                            {
                                childControl.Width = resolvedWidth;
                            }
                        }
                    }

                    // Wrap <li> children with list markers
                    if (isList && child.Tag == "li"
                        && child.Styles.ListStyleType != "none")
                    {
                        listItemIndex++;
                        childControl = BuildListItemWithMarker(
                            childControl, child, fontSize, listItemIndex);
                    }

                    // align-self: override parent's align-items for this specific child
                    if (isFlex && child.Styles.AlignSelf is { } selfAlign
                        && selfAlign != "auto")
                    {
                        if (isRow)
                        {
                            childControl.VerticalAlignment = selfAlign switch
                            {
                                "flex-start" or "start" => VerticalAlignment.Top,
                                "flex-end" or "end" => VerticalAlignment.Bottom,
                                "center" => VerticalAlignment.Center,
                                "stretch" => VerticalAlignment.Stretch,
                                _ => childControl.VerticalAlignment,
                            };
                        }
                        else
                        {
                            childControl.HorizontalAlignment = selfAlign switch
                            {
                                "flex-start" or "start" => HorizontalAlignment.Left,
                                "flex-end" or "end" => HorizontalAlignment.Right,
                                "center" => HorizontalAlignment.Center,
                                "stretch" => HorizontalAlignment.Stretch,
                                _ => childControl.HorizontalAlignment,
                            };
                        }
                    }

                    if (isHorizFlexGrid)
                    {
                        // Add gap column before this item (if not first)
                        if (flexGap.HasValue && flexGridCol > 0)
                        {
                            ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(new GridLength(flexGap.Value)));
                            flexGridCol++;
                        }
                        // CSS width:100% on a flex child means "fill the parent".
                        // In an Auto column, HorizontalAlignment.Stretch = content width (wrong).
                        // Use Star column so the child can actually stretch to fill available space.
                        if (IsChildFullWidth(child))
                            ((Grid)panel).ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));
                        else
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
    /// Build a float layout: float:left children in left column, normal flow children in
    /// center column, float:right children in right column. Simulates CSS float behavior
    /// for common patterns like Wikipedia's infobox (float:right) and TOC (float:left).
    /// </summary>
    private Control BuildFloatLayout(
        StyledElement element,
        List<StyledElement> normalFlow,
        List<StyledElement> floatLeft,
        List<StyledElement> floatRight,
        double fontSize,
        int depth,
        double availableWidth = 0)
    {
        var outerGrid = new Grid();
        outerGrid.HorizontalAlignment = HorizontalAlignment.Stretch;

        int contentCol = 0;

        // Left float column
        if (floatLeft.Any())
        {
            outerGrid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
            contentCol = 1;

            var leftPanel = new StackPanel { Orientation = Orientation.Vertical };
            foreach (var fc in floatLeft)
            {
                var ctrl = BuildControl(fc, fontSize, depth + 1, availableWidth);
                if (ctrl != null)
                    leftPanel.Children.Add(ctrl);
            }
            Grid.SetColumn(leftPanel, 0);
            outerGrid.Children.Add(leftPanel);
        }

        // Normal flow children in center (Star column — fills remaining space)
        outerGrid.ColumnDefinitions.Add(new ColumnDefinition(1, GridUnitType.Star));

        var normalPanel = new StackPanel { Orientation = Orientation.Vertical };
        foreach (var child in normalFlow)
        {
            if (child.Styles.Display == "none") continue;
            var ctrl = BuildControl(child, fontSize, depth + 1, availableWidth);
            if (ctrl != null)
                normalPanel.Children.Add(ctrl);
        }
        Grid.SetColumn(normalPanel, contentCol);
        outerGrid.Children.Add(normalPanel);

        // Right float column
        if (floatRight.Any())
        {
            outerGrid.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto));
            int rightCol = contentCol + 1;

            var rightPanel = new StackPanel { Orientation = Orientation.Vertical };
            foreach (var fc in floatRight)
            {
                var ctrl = BuildControl(fc, fontSize, depth + 1, availableWidth);
                if (ctrl != null)
                    rightPanel.Children.Add(ctrl);
            }
            Grid.SetColumn(rightPanel, rightCol);
            outerGrid.Children.Add(rightPanel);
        }

        return outerGrid;
    }
}
