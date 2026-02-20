using Avalonia;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Tests.Helpers;

namespace ThaloraBrowser.Tests.Rendering;

#region CollapseWhitespace — Pure tests (no Avalonia needed)

public class CollapseWhitespaceTests
{
    [Fact]
    public void CollapseWhitespace_ConsecutiveSpaces_CollapsedToSingle()
    {
        var result = TextLayoutEngine.CollapseWhitespace("hello  world");

        result.Should().Be("hello world");
    }

    [Fact]
    public void CollapseWhitespace_LeadingAndTrailingSpaces_PreservedAsSingleSpaces()
    {
        var result = TextLayoutEngine.CollapseWhitespace("  hello  ");

        result.Should().Be(" hello ");
    }

    [Fact]
    public void CollapseWhitespace_TabsAndNewlines_CollapsedToSingleSpaces()
    {
        var result = TextLayoutEngine.CollapseWhitespace("hello\t\nworld");

        result.Should().Be("hello world");
    }

    [Fact]
    public void CollapseWhitespace_EmptyString_ReturnsEmptyString()
    {
        var result = TextLayoutEngine.CollapseWhitespace("");

        result.Should().BeEmpty();
    }

    [Fact]
    public void CollapseWhitespace_Null_ReturnsNull()
    {
        var result = TextLayoutEngine.CollapseWhitespace(null!);

        result.Should().BeNull();
    }

    [Fact]
    public void CollapseWhitespace_NoWhitespaceToCollapse_ReturnsUnchanged()
    {
        var result = TextLayoutEngine.CollapseWhitespace("hello");

        result.Should().Be("hello");
    }

    [Fact]
    public void CollapseWhitespace_AllWhitespace_CollapsedToSingleSpace()
    {
        var result = TextLayoutEngine.CollapseWhitespace("   ");

        result.Should().Be(" ");
    }

    [Fact]
    public void CollapseWhitespace_MixedWhitespaceTypes_CollapsedToSingleSpace()
    {
        var result = TextLayoutEngine.CollapseWhitespace(" \t \n \r ");

        result.Should().Be(" ");
    }
}

#endregion

#region MeasureText + WrapLine — Avalonia headless tests

[Trait("Category", "AvaloniaHeadless")]
[Collection("Avalonia")]
public class TextLayoutEngineAvaloniaTests
{
    public TextLayoutEngineAvaloniaTests()
    {
        AvaloniaTestApp.EnsureInitialized();
    }

    private static CssComputedStyle DefaultStyle() => new()
    {
        FontSize = 16,
        FontFamily = Avalonia.Media.FontFamily.Default,
        FontWeight = Avalonia.Media.FontWeight.Normal,
        FontStyle = Avalonia.Media.FontStyle.Normal,
        Color = Avalonia.Media.Brushes.White,
        LineHeight = 1.4,
    };

    [Fact]
    public void MeasureText_EmptyString_ReturnsZeroWidthAndFontSizeTimesLineHeight()
    {
        var style = DefaultStyle();

        var size = TextLayoutEngine.MeasureText("", style);

        size.Width.Should().Be(0);
        size.Height.Should().BeApproximately(style.FontSize * style.LineHeight, 0.001);
    }

    [Fact]
    public void MeasureText_NonEmptyString_ReturnsPositiveWidthAndHeight()
    {
        var style = DefaultStyle();

        var size = TextLayoutEngine.MeasureText("Hello, world!", style);

        size.Width.Should().BeGreaterThan(0);
        size.Height.Should().BeGreaterThan(0);
    }

    [Fact]
    public void WrapLine_ShortText_ReturnsSingleLine()
    {
        var style = DefaultStyle();

        var lines = TextLayoutEngine.WrapLine("Hi", style, availableWidth: 500);

        lines.Should().HaveCount(1);
        lines[0].Should().Be("Hi");
    }

    [Fact]
    public void WrapLine_NarrowWidth_WrapsIntoMultipleLines()
    {
        var style = DefaultStyle();

        var lines = TextLayoutEngine.WrapLine("Hello wonderful world", style, availableWidth: 10);

        lines.Should().HaveCountGreaterThan(1);
    }
}

#endregion
