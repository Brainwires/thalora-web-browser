using Avalonia;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// A node in the layout tree. Each LayoutBox corresponds to a positioned element
/// (or an anonymous box for text runs) with computed position and dimensions.
/// Layout data comes from the Rust engine via JSON.
/// </summary>
public class LayoutBox
{
    /// <summary>The type of this box in the formatting context.</summary>
    public BoxType Type { get; set; }

    /// <summary>Content area rectangle (position + size after layout).</summary>
    public Rect ContentRect { get; set; }

    /// <summary>Box model edges.</summary>
    public Thickness Margin { get; set; }
    public Thickness Border { get; set; }
    public Thickness Padding { get; set; }

    /// <summary>Resolved CSS properties for painting.</summary>
    public CssComputedStyle Style { get; set; } = new();

    /// <summary>Child layout boxes.</summary>
    public List<LayoutBox> Children { get; set; } = new();

    /// <summary>Text runs for inline/text boxes.</summary>
    public List<TextRun>? TextRuns { get; set; }

    /// <summary>Image source URL (for img elements).</summary>
    public string? ImageSource { get; set; }

    /// <summary>Link href (for clickable elements).</summary>
    public string? LinkHref { get; set; }

    /// <summary>1-based index for list items within their parent list.</summary>
    public int ListItemIndex { get; set; } = 1;

    /// <summary>The full border box rectangle including margin.</summary>
    public Rect MarginBox => new(
        ContentRect.X - Padding.Left - Border.Left - Margin.Left,
        ContentRect.Y - Padding.Top - Border.Top - Margin.Top,
        ContentRect.Width + Padding.Left + Padding.Right + Border.Left + Border.Right + Margin.Left + Margin.Right,
        ContentRect.Height + Padding.Top + Padding.Bottom + Border.Top + Border.Bottom + Margin.Top + Margin.Bottom
    );

    /// <summary>The border box rectangle (content + padding + border).</summary>
    public Rect BorderBox => new(
        ContentRect.X - Padding.Left - Border.Left,
        ContentRect.Y - Padding.Top - Border.Top,
        ContentRect.Width + Padding.Left + Padding.Right + Border.Left + Border.Right,
        ContentRect.Height + Padding.Top + Padding.Bottom + Border.Top + Border.Bottom
    );

    /// <summary>The padding box rectangle (content + padding).</summary>
    public Rect PaddingBox => new(
        ContentRect.X - Padding.Left,
        ContentRect.Y - Padding.Top,
        ContentRect.Width + Padding.Left + Padding.Right,
        ContentRect.Height + Padding.Top + Padding.Bottom
    );
}

public enum BoxType
{
    Block,
    Inline,
    InlineBlock,
    Anonymous,
    ListItem,
    TableBox,
    TableRowBox,
    TableCellBox,
}

/// <summary>
/// A positioned run of text within an inline formatting context.
/// </summary>
public class TextRun
{
    public string Text { get; set; } = "";
    public CssComputedStyle Style { get; set; } = new();
    public Rect Bounds { get; set; }
    public string? LinkHref { get; set; }
}
