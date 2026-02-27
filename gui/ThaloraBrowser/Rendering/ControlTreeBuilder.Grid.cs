using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// CSS Grid layout building and grid template parsing.
/// Contains BuildGridContent and related parsing helpers for
/// grid-template-columns, grid-template-rows, grid-template-areas.
/// </summary>
public partial class ControlTreeBuilder
{
    /// <summary>
    /// Build an Avalonia Grid for a CSS Grid container (display: grid).
    /// Handles grid-template-columns, grid-template-rows, grid-template-areas,
    /// and child grid-area placement.
    /// </summary>
    private Control BuildGridContent(StyledElement element, double fontSize, int depth)
    {
        var styles = element.Styles;

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
                // Insert an invisible placeholder so the grid cell still occupies space.
                // Without this, Avalonia Grid skips the column entirely.
                childControl = new Panel { MinWidth = 0, MinHeight = 0 };
            }

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
}
