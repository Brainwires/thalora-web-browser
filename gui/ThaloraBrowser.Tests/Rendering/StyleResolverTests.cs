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
        // Avalonia's Color.TryParse handles rgba() and may round alpha differently
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
        // Avalonia's Color.TryParse may handle clamping; verify alpha is near max
        result!.Value.A.Should().BeGreaterOrEqualTo(254);
    }

    [Fact]
    public void ParseColor_Rgba_AlphaBelowZero_ClampsToMin()
    {
        var result = StyleResolver.ParseColor("rgba(100, 100, 100, -1.0)");
        result.Should().NotBeNull();
        // Avalonia's Color.TryParse may handle clamping; verify alpha is near min
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

#region CreateRootStyle Tests

[Collection("Avalonia")]
public class StyleResolver_CreateRootStyleTests
{
    public StyleResolver_CreateRootStyleTests()
    {
        AvaloniaTestApp.EnsureInitialized();
    }

    [Fact]
    public void CreateRootStyle_ReturnsCorrectDefaults()
    {
        var resolver = new StyleResolver();
        var root = resolver.CreateRootStyle();

        root.FontSize.Should().Be(16);
        root.FontWeight.Should().Be(FontWeight.Normal);
        root.FontFamily.Should().NotBeNull();
        root.Display.Should().Be(DisplayMode.Block);
        root.LineHeight.Should().Be(1.4);

        // Color should be rgb(220, 220, 220)
        var colorBrush = root.Color.Should().BeOfType<SolidColorBrush>().Subject;
        colorBrush.Color.R.Should().Be(220);
        colorBrush.Color.G.Should().Be(220);
        colorBrush.Color.B.Should().Be(220);

        // BackgroundColor should be rgb(30, 30, 30)
        var bgBrush = root.BackgroundColor.Should().BeOfType<SolidColorBrush>().Subject;
        bgBrush.Color.R.Should().Be(30);
        bgBrush.Color.G.Should().Be(30);
        bgBrush.Color.B.Should().Be(30);
    }
}

#endregion

#region ComputeStyle Tests (require AngleSharp DOM)

[Trait("Category", "Integration")]
[Collection("Avalonia")]
public class StyleResolver_ComputeStyleTests
{
    private readonly StyleResolver _resolver = new();

    private async Task<AngleSharp.Dom.IElement> GetElementAsync(string html, string selector)
    {
        AvaloniaTestApp.EnsureInitialized();
        var doc = await LayoutTestHelper.ParseHtmlAsync(html);
        return doc.QuerySelector(selector)!;
    }

    [Fact]
    public async Task ComputeStyle_H1_HasCorrectFontSizeAndWeight()
    {
        var element = await GetElementAsync("<html><body><h1>Title</h1></body></html>", "h1");
        var style = _resolver.ComputeStyle(element, null);

        style.FontSize.Should().Be(32);
        style.FontWeight.Should().Be(FontWeight.Bold);
    }

    [Fact]
    public async Task ComputeStyle_H2_HasCorrectFontSize()
    {
        var element = await GetElementAsync("<html><body><h2>Subtitle</h2></body></html>", "h2");
        var style = _resolver.ComputeStyle(element, null);

        style.FontSize.Should().Be(24);
        style.FontWeight.Should().Be(FontWeight.Bold);
    }

    [Fact]
    public async Task ComputeStyle_P_HasCorrectMargin()
    {
        var element = await GetElementAsync("<html><body><p>Text</p></body></html>", "p");
        var style = _resolver.ComputeStyle(element, null);

        // AngleSharp CSS may apply UA stylesheet margins (e.g. 1em = 16px or 1.12em ≈ 17.92px)
        style.Margin.Top.Should().BeGreaterOrEqualTo(16);
        style.Margin.Bottom.Should().BeGreaterOrEqualTo(16);
    }

    [Fact]
    public async Task ComputeStyle_A_HasUnderlineAndCornflowerBlueColor()
    {
        var element = await GetElementAsync("<html><body><a href=\"#\">Link</a></body></html>", "a");
        var style = _resolver.ComputeStyle(element, null);

        style.TextDecorations.Should().NotBeNull();
        style.TextDecorations.Should().BeSameAs(Avalonia.Media.TextDecorations.Underline);

        var colorBrush = style.Color.Should().BeOfType<SolidColorBrush>().Subject;
        colorBrush.Color.R.Should().Be(100);
        colorBrush.Color.G.Should().Be(149);
        colorBrush.Color.B.Should().Be(237);
    }

    [Fact]
    public async Task ComputeStyle_InheritsFromParent_FontSize()
    {
        var element = await GetElementAsync("<html><body><div><span>Text</span></div></body></html>", "span");
        var parentStyle = new CssComputedStyle { FontSize = 24 };
        var style = _resolver.ComputeStyle(element, parentStyle);

        style.FontSize.Should().Be(24);
    }

    [Fact]
    public async Task ComputeStyle_InheritsFromParent_Color()
    {
        var element = await GetElementAsync("<html><body><div><span>Text</span></div></body></html>", "span");
        var parentColor = new SolidColorBrush(Color.FromRgb(255, 0, 0));
        var parentStyle = new CssComputedStyle { Color = parentColor };
        var style = _resolver.ComputeStyle(element, parentStyle);

        var colorBrush = style.Color.Should().BeOfType<SolidColorBrush>().Subject;
        colorBrush.Color.R.Should().Be(255);
        colorBrush.Color.G.Should().Be(0);
        colorBrush.Color.B.Should().Be(0);
    }

    [Fact]
    public async Task ComputeStyle_InheritsFromParent_FontWeight()
    {
        var element = await GetElementAsync("<html><body><div><span>Text</span></div></body></html>", "span");
        var parentStyle = new CssComputedStyle { FontWeight = FontWeight.Bold };
        var style = _resolver.ComputeStyle(element, parentStyle);

        style.FontWeight.Should().Be(FontWeight.Bold);
    }

    [Fact]
    public async Task ComputeStyle_InheritsFromParent_TextAlign()
    {
        var element = await GetElementAsync("<html><body><div><p>Text</p></div></body></html>", "p");
        var parentStyle = new CssComputedStyle { TextAlign = TextAlignment.Center };
        var style = _resolver.ComputeStyle(element, parentStyle);

        style.TextAlign.Should().Be(TextAlignment.Center);
    }

    [Fact]
    public async Task ComputeStyle_DisplayNone_SetsIsVisibleFalse()
    {
        var element = await GetElementAsync(
            "<html><body><div style=\"display:none\">Hidden</div></body></html>", "div");
        var style = _resolver.ComputeStyle(element, null);

        style.Display.Should().Be(DisplayMode.None);
        style.IsVisible.Should().BeFalse();
    }

    [Fact]
    public async Task ComputeStyle_Div_IsBlock()
    {
        var element = await GetElementAsync("<html><body><div>Content</div></body></html>", "div");
        var style = _resolver.ComputeStyle(element, null);

        style.Display.Should().Be(DisplayMode.Block);
    }

    [Fact]
    public async Task ComputeStyle_Span_IsInline()
    {
        var element = await GetElementAsync("<html><body><span>Text</span></body></html>", "span");
        var style = _resolver.ComputeStyle(element, null);

        style.Display.Should().Be(DisplayMode.Inline);
    }

    [Fact]
    public async Task ComputeStyle_Strong_IsBold()
    {
        var element = await GetElementAsync("<html><body><strong>Bold</strong></body></html>", "strong");
        var style = _resolver.ComputeStyle(element, null);

        style.FontWeight.Should().Be(FontWeight.Bold);
    }

    [Fact]
    public async Task ComputeStyle_Em_IsItalic()
    {
        var element = await GetElementAsync("<html><body><em>Italic</em></body></html>", "em");
        var style = _resolver.ComputeStyle(element, null);

        style.FontStyle.Should().Be(FontStyle.Italic);
    }

    [Fact]
    public async Task ComputeStyle_Li_IsListItem()
    {
        var element = await GetElementAsync("<html><body><ul><li>Item</li></ul></body></html>", "li");
        var style = _resolver.ComputeStyle(element, null);

        style.Display.Should().Be(DisplayMode.ListItem);
    }

    [Fact]
    public async Task ComputeStyle_NullParent_DoesNotThrow()
    {
        var element = await GetElementAsync("<html><body><div>Root</div></body></html>", "div");

        var act = () => _resolver.ComputeStyle(element, null);
        act.Should().NotThrow();
    }
}

#endregion
