using Avalonia;
using Avalonia.Media;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Resolved CSS properties for a layout box, converted to Avalonia-ready values.
/// </summary>
public class CssComputedStyle
{
    // Box model
    public Thickness Margin { get; set; }
    public Thickness Padding { get; set; }
    public Thickness BorderWidth { get; set; }
    public IBrush? BorderBrush { get; set; }

    // Sizing
    public double? Width { get; set; }
    public double? Height { get; set; }
    public double? MaxWidth { get; set; }
    public double? MinWidth { get; set; }

    // Typography
    public double FontSize { get; set; } = 16;
    public FontWeight FontWeight { get; set; } = FontWeight.Normal;
    public FontFamily FontFamily { get; set; } = FontFamily.Default;
    public FontStyle FontStyle { get; set; } = FontStyle.Normal;
    public IBrush Color { get; set; } = Brushes.White;
    public TextAlignment TextAlign { get; set; } = TextAlignment.Left;
    public TextDecorationCollection? TextDecorations { get; set; }
    public double LineHeight { get; set; } = 1.4;
    public WhiteSpaceMode WhiteSpace { get; set; } = WhiteSpaceMode.Normal;

    // Visual
    public IBrush? BackgroundColor { get; set; }
    public double Opacity { get; set; } = 1.0;
    public DisplayMode Display { get; set; } = DisplayMode.Block;
    public bool IsVisible { get; set; } = true;
    public CornerRadius BorderRadius { get; set; }
    public OverflowMode Overflow { get; set; } = OverflowMode.Visible;

    // List
    public ListStyleType ListStyleType { get; set; } = ListStyleType.None;
}

public enum DisplayMode
{
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
    ListItem,
    Table,
    TableRow,
    TableCell,
}

public enum WhiteSpaceMode
{
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}

public enum OverflowMode
{
    Visible,
    Hidden,
    Scroll,
    Auto,
}

public enum ListStyleType
{
    None,
    Disc,
    Circle,
    Square,
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}
