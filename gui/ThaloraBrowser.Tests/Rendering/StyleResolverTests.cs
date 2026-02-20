using Avalonia;
using Avalonia.Media;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Tests.Helpers;

namespace ThaloraBrowser.Tests.Rendering;

#region ParseLength Tests

public class StyleResolver_ParseLengthTests
{
    [Fact]
    public void ParseLength_Px_ReturnsPixelValue()
    {
        StyleResolver.ParseLength("100px", 16).Should().Be(100);
    }

    [Fact]
    public void ParseLength_Em_MultipliesByParentFontSize()
    {
        StyleResolver.ParseLength("2em", 16).Should().Be(32);
    }

    [Fact]
    public void ParseLength_Em_UsesActualParentFontSize()
    {
        StyleResolver.ParseLength("1.5em", 20).Should().Be(30);
    }

    [Fact]
    public void ParseLength_Rem_AlwaysMultipliesBy16()
    {
        StyleResolver.ParseLength("2rem", 20).Should().Be(32);
    }

    [Fact]
    public void ParseLength_Percent_UsesParentFontSize()
    {
        // 150% of 16 = 24
        StyleResolver.ParseLength("150%", 16).Should().Be(24);
    }

    [Fact]
    public void ParseLength_Pt_MultipliesByFourThirds()
    {
        // 12pt = 12 * 4/3 = 16
        StyleResolver.ParseLength("12pt", 16).Should().Be(16);
    }

    [Fact]
    public void ParseLength_Vh_MultipliesBy10()
    {
        StyleResolver.ParseLength("50vh", 16).Should().Be(500);
    }

    [Fact]
    public void ParseLength_Vw_MultipliesBy10()
    {
        StyleResolver.ParseLength("10vw", 16).Should().Be(100);
    }

    [Fact]
    public void ParseLength_Auto_ReturnsNull()
    {
        StyleResolver.ParseLength("auto", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_None_ReturnsNull()
    {
        StyleResolver.ParseLength("none", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_EmptyString_ReturnsNull()
    {
        StyleResolver.ParseLength("", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_Whitespace_ReturnsNull()
    {
        StyleResolver.ParseLength("   ", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_BareNumber_ReturnsRawValue()
    {
        StyleResolver.ParseLength("42", 16).Should().Be(42);
    }

    [Fact]
    public void ParseLength_NegativeValue_ReturnsNegative()
    {
        StyleResolver.ParseLength("-10px", 16).Should().Be(-10);
    }

    [Fact]
    public void ParseLength_DecimalValue_ReturnsDecimal()
    {
        StyleResolver.ParseLength("1.5px", 16).Should().Be(1.5);
    }
}

#endregion

#region ParseColor Tests

public class StyleResolver_ParseColorTests
{
    [Fact]
    public void ParseColor_Hex6Digit_ReturnsCorrectColor()
    {
        var result = StyleResolver.ParseColor("#ff0000");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_Hex3Digit_ReturnsCorrectColor()
    {
        var result = StyleResolver.ParseColor("#f00");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_Rgb_ReturnsCorrectColor()
    {
        var result = StyleResolver.ParseColor("rgb(100, 200, 50)");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(100);
        result.Value.G.Should().Be(200);
        result.Value.B.Should().Be(50);
        result.Value.A.Should().Be(255);
    }

    [Fact]
    public void ParseColor_Rgba_ReturnsCorrectColorWithAlpha()
    {
        var result = StyleResolver.ParseColor("rgba(100, 200, 50, 0.5)");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(100);
        result.Value.G.Should().Be(200);
        result.Value.B.Should().Be(50);
        result.Value.A.Should().BeInRange((byte)127, (byte)128);
    }

    [Fact]
    public void ParseColor_Transparent_ReturnsNull()
    {
        StyleResolver.ParseColor("transparent").Should().BeNull();
    }

    [Fact]
    public void ParseColor_NamedColor_Red_ReturnsCorrectColor()
    {
        var result = StyleResolver.ParseColor("red");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_NamedColor_Blue_ReturnsCorrectColor()
    {
        var result = StyleResolver.ParseColor("blue");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(0);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(255);
    }

    [Fact]
    public void ParseColor_Null_ReturnsNull()
    {
        StyleResolver.ParseColor(null!).Should().BeNull();
    }

    [Fact]
    public void ParseColor_Empty_ReturnsNull()
    {
        StyleResolver.ParseColor("").Should().BeNull();
    }

    [Fact]
    public void ParseColor_Rgba_AlphaAboveOne_ClampsToMax()
    {
        var result = StyleResolver.ParseColor("rgba(100, 100, 100, 2.0)");
        result.Should().NotBeNull();
        result!.Value.A.Should().BeGreaterOrEqualTo(254);
    }

    [Fact]
    public void ParseColor_Rgba_AlphaBelowZero_ClampsToMin()
    {
        var result = StyleResolver.ParseColor("rgba(100, 100, 100, -1.0)");
        result.Should().NotBeNull();
        result!.Value.A.Should().BeLessOrEqualTo(1);
    }
}

#endregion

#region ParseDisplay Tests

public class StyleResolver_ParseDisplayTests
{
    [Fact]
    public void ParseDisplay_Block_ReturnsBlock()
    {
        StyleResolver.ParseDisplay("block").Should().Be(DisplayMode.Block);
    }

    [Fact]
    public void ParseDisplay_Inline_ReturnsInline()
    {
        StyleResolver.ParseDisplay("inline").Should().Be(DisplayMode.Inline);
    }

    [Fact]
    public void ParseDisplay_InlineBlock_ReturnsInlineBlock()
    {
        StyleResolver.ParseDisplay("inline-block").Should().Be(DisplayMode.InlineBlock);
    }

    [Fact]
    public void ParseDisplay_None_ReturnsNone()
    {
        StyleResolver.ParseDisplay("none").Should().Be(DisplayMode.None);
    }

    [Fact]
    public void ParseDisplay_Flex_ReturnsFlex()
    {
        StyleResolver.ParseDisplay("flex").Should().Be(DisplayMode.Flex);
    }

    [Fact]
    public void ParseDisplay_ListItem_ReturnsListItem()
    {
        StyleResolver.ParseDisplay("list-item").Should().Be(DisplayMode.ListItem);
    }

    [Fact]
    public void ParseDisplay_Table_ReturnsTable()
    {
        StyleResolver.ParseDisplay("table").Should().Be(DisplayMode.Table);
    }

    [Fact]
    public void ParseDisplay_TableRow_ReturnsTableRow()
    {
        StyleResolver.ParseDisplay("table-row").Should().Be(DisplayMode.TableRow);
    }

    [Fact]
    public void ParseDisplay_TableCell_ReturnsTableCell()
    {
        StyleResolver.ParseDisplay("table-cell").Should().Be(DisplayMode.TableCell);
    }

    [Fact]
    public void ParseDisplay_CaseInsensitive_ReturnsCorrectValue()
    {
        StyleResolver.ParseDisplay("BLOCK").Should().Be(DisplayMode.Block);
        StyleResolver.ParseDisplay("Inline").Should().Be(DisplayMode.Inline);
        StyleResolver.ParseDisplay("FLEX").Should().Be(DisplayMode.Flex);
    }

    [Fact]
    public void ParseDisplay_Unknown_DefaultsToBlock()
    {
        StyleResolver.ParseDisplay("grid").Should().Be(DisplayMode.Block);
        StyleResolver.ParseDisplay("nonsense").Should().Be(DisplayMode.Block);
    }
}

#endregion

#region ParseFontWeight Tests

public class StyleResolver_ParseFontWeightTests
{
    [Fact]
    public void ParseFontWeight_Bold_ReturnsBold()
    {
        StyleResolver.ParseFontWeight("bold").Should().Be(FontWeight.Bold);
    }

    [Fact]
    public void ParseFontWeight_Normal_ReturnsNormal()
    {
        StyleResolver.ParseFontWeight("normal").Should().Be(FontWeight.Normal);
    }

    [Fact]
    public void ParseFontWeight_Bolder_ReturnsBold()
    {
        StyleResolver.ParseFontWeight("bolder").Should().Be(FontWeight.Bold);
    }

    [Fact]
    public void ParseFontWeight_Lighter_ReturnsLight()
    {
        StyleResolver.ParseFontWeight("lighter").Should().Be(FontWeight.Light);
    }

    [Fact]
    public void ParseFontWeight_100_ReturnsThin()
    {
        StyleResolver.ParseFontWeight("100").Should().Be(FontWeight.Thin);
    }

    [Fact]
    public void ParseFontWeight_200_ReturnsExtraLight()
    {
        StyleResolver.ParseFontWeight("200").Should().Be(FontWeight.ExtraLight);
    }

    [Fact]
    public void ParseFontWeight_300_ReturnsLight()
    {
        StyleResolver.ParseFontWeight("300").Should().Be(FontWeight.Light);
    }

    [Fact]
    public void ParseFontWeight_400_ReturnsNormal()
    {
        StyleResolver.ParseFontWeight("400").Should().Be(FontWeight.Normal);
    }

    [Fact]
    public void ParseFontWeight_500_ReturnsMedium()
    {
        StyleResolver.ParseFontWeight("500").Should().Be(FontWeight.Medium);
    }

    [Fact]
    public void ParseFontWeight_600_ReturnsSemiBold()
    {
        StyleResolver.ParseFontWeight("600").Should().Be(FontWeight.SemiBold);
    }

    [Fact]
    public void ParseFontWeight_700_ReturnsBold()
    {
        StyleResolver.ParseFontWeight("700").Should().Be(FontWeight.Bold);
    }

    [Fact]
    public void ParseFontWeight_800_ReturnsExtraBold()
    {
        StyleResolver.ParseFontWeight("800").Should().Be(FontWeight.ExtraBold);
    }

    [Fact]
    public void ParseFontWeight_900_ReturnsBlack()
    {
        StyleResolver.ParseFontWeight("900").Should().Be(FontWeight.Black);
    }

    [Fact]
    public void ParseFontWeight_Unknown_DefaultsToNormal()
    {
        StyleResolver.ParseFontWeight("ultralight").Should().Be(FontWeight.Normal);
        StyleResolver.ParseFontWeight("xyz").Should().Be(FontWeight.Normal);
    }
}

#endregion

#region ParseTextAlign Tests

public class StyleResolver_ParseTextAlignTests
{
    [Fact]
    public void ParseTextAlign_Left_ReturnsLeft()
    {
        StyleResolver.ParseTextAlign("left").Should().Be(TextAlignment.Left);
    }

    [Fact]
    public void ParseTextAlign_Right_ReturnsRight()
    {
        StyleResolver.ParseTextAlign("right").Should().Be(TextAlignment.Right);
    }

    [Fact]
    public void ParseTextAlign_Center_ReturnsCenter()
    {
        StyleResolver.ParseTextAlign("center").Should().Be(TextAlignment.Center);
    }

    [Fact]
    public void ParseTextAlign_Justify_ReturnsJustify()
    {
        StyleResolver.ParseTextAlign("justify").Should().Be(TextAlignment.Justify);
    }

    [Fact]
    public void ParseTextAlign_Unknown_DefaultsToLeft()
    {
        StyleResolver.ParseTextAlign("start").Should().Be(TextAlignment.Left);
        StyleResolver.ParseTextAlign("nonsense").Should().Be(TextAlignment.Left);
    }
}

#endregion

#region ParseWhiteSpace Tests

public class StyleResolver_ParseWhiteSpaceTests
{
    [Fact]
    public void ParseWhiteSpace_Normal_ReturnsNormal()
    {
        StyleResolver.ParseWhiteSpace("normal").Should().Be(WhiteSpaceMode.Normal);
    }

    [Fact]
    public void ParseWhiteSpace_NoWrap_ReturnsNoWrap()
    {
        StyleResolver.ParseWhiteSpace("nowrap").Should().Be(WhiteSpaceMode.NoWrap);
    }

    [Fact]
    public void ParseWhiteSpace_Pre_ReturnsPre()
    {
        StyleResolver.ParseWhiteSpace("pre").Should().Be(WhiteSpaceMode.Pre);
    }

    [Fact]
    public void ParseWhiteSpace_PreWrap_ReturnsPreWrap()
    {
        StyleResolver.ParseWhiteSpace("pre-wrap").Should().Be(WhiteSpaceMode.PreWrap);
    }

    [Fact]
    public void ParseWhiteSpace_PreLine_ReturnsPreLine()
    {
        StyleResolver.ParseWhiteSpace("pre-line").Should().Be(WhiteSpaceMode.PreLine);
    }

    [Fact]
    public void ParseWhiteSpace_Unknown_DefaultsToNormal()
    {
        StyleResolver.ParseWhiteSpace("break-spaces").Should().Be(WhiteSpaceMode.Normal);
        StyleResolver.ParseWhiteSpace("xyz").Should().Be(WhiteSpaceMode.Normal);
    }
}

#endregion

#region ParseOverflow Tests

public class StyleResolver_ParseOverflowTests
{
    [Fact]
    public void ParseOverflow_Visible_ReturnsVisible()
    {
        StyleResolver.ParseOverflow("visible").Should().Be(OverflowMode.Visible);
    }

    [Fact]
    public void ParseOverflow_Hidden_ReturnsHidden()
    {
        StyleResolver.ParseOverflow("hidden").Should().Be(OverflowMode.Hidden);
    }

    [Fact]
    public void ParseOverflow_Scroll_ReturnsScroll()
    {
        StyleResolver.ParseOverflow("scroll").Should().Be(OverflowMode.Scroll);
    }

    [Fact]
    public void ParseOverflow_Auto_ReturnsAuto()
    {
        StyleResolver.ParseOverflow("auto").Should().Be(OverflowMode.Auto);
    }

    [Fact]
    public void ParseOverflow_Unknown_DefaultsToVisible()
    {
        StyleResolver.ParseOverflow("clip").Should().Be(OverflowMode.Visible);
        StyleResolver.ParseOverflow("nonsense").Should().Be(OverflowMode.Visible);
    }
}

#endregion
