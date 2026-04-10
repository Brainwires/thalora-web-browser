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
        StyleParser.ParseLength("100px", 16).Should().Be(100);
    }

    [Fact]
    public void ParseLength_Em_MultipliesByParentFontSize()
    {
        StyleParser.ParseLength("2em", 16).Should().Be(32);
    }

    [Fact]
    public void ParseLength_Em_UsesActualParentFontSize()
    {
        StyleParser.ParseLength("1.5em", 20).Should().Be(30);
    }

    [Fact]
    public void ParseLength_Rem_AlwaysMultipliesBy16()
    {
        StyleParser.ParseLength("2rem", 20).Should().Be(32);
    }

    [Fact]
    public void ParseLength_Percent_UsesParentSize()
    {
        // 150% of parentSize=16 → 24
        StyleParser.ParseLength("150%", parentFontSize: 16, parentSize: 16).Should().Be(24);
    }

    [Fact]
    public void ParseLength_Pt_MultipliesByFourThirds()
    {
        // 12pt = 12 * 4/3 = 16
        StyleParser.ParseLength("12pt", 16).Should().Be(16);
    }

    [Fact]
    public void ParseLength_Vh_MultipliesBy10()
    {
        StyleParser.ParseLength("50vh", viewportHeight: 1000).Should().Be(500);
    }

    [Fact]
    public void ParseLength_Vw_MultipliesBy10()
    {
        StyleParser.ParseLength("10vw", viewportWidth: 1000).Should().Be(100);
    }

    [Fact]
    public void ParseLength_Auto_ReturnsNull()
    {
        StyleParser.ParseLength("auto", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_None_ReturnsNull()
    {
        StyleParser.ParseLength("none", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_EmptyString_ReturnsNull()
    {
        StyleParser.ParseLength("", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_Whitespace_ReturnsNull()
    {
        StyleParser.ParseLength("   ", 16).Should().BeNull();
    }

    [Fact]
    public void ParseLength_BareNumber_ReturnsRawValue()
    {
        StyleParser.ParseLength("42", 16).Should().Be(42);
    }

    [Fact]
    public void ParseLength_NegativeValue_ReturnsNegative()
    {
        StyleParser.ParseLength("-10px", 16).Should().Be(-10);
    }

    [Fact]
    public void ParseLength_DecimalValue_ReturnsDecimal()
    {
        StyleParser.ParseLength("1.5px", 16).Should().Be(1.5);
    }
}

#endregion

#region ParseColor Tests
// NOTE: Rust normalizes rgb()/rgba()/hsl() to #rrggbb or #rrggbbaa hex before serialization.
// C# ParseColor only receives hex (#rrggbb, #rgb, #rrggbbaa) and CSS named colors.

public class StyleResolver_ParseColorTests
{
    [Fact]
    public void ParseColor_Hex6Digit_ReturnsCorrectColor()
    {
        var result = StyleParser.ParseColor("#ff0000");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_Hex3Digit_ReturnsCorrectColor()
    {
        var result = StyleParser.ParseColor("#f00");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_Hex8Digit_WithAlpha_ReturnsCorrectColor()
    {
        // Rust pre-normalizes rgba(100,200,50,0.5) → "#80 64 c8 32" in ARGB order
        // (Avalonia uses #AARRGGBB, not CSS's #RRGGBBAA)
        var result = StyleParser.ParseColor("#8064c832");
        result.Should().NotBeNull();
        result!.Value.A.Should().Be(0x80); // ~128
        result.Value.R.Should().Be(100);   // 0x64
        result.Value.G.Should().Be(200);   // 0xc8
        result.Value.B.Should().Be(50);    // 0x32
    }

    [Fact]
    public void ParseColor_Transparent_ReturnsNull()
    {
        StyleParser.ParseColor("transparent").Should().BeNull();
    }

    [Fact]
    public void ParseColor_NamedColor_Red_ReturnsCorrectColor()
    {
        var result = StyleParser.ParseColor("red");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(255);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(0);
    }

    [Fact]
    public void ParseColor_NamedColor_Blue_ReturnsCorrectColor()
    {
        var result = StyleParser.ParseColor("blue");
        result.Should().NotBeNull();
        result!.Value.R.Should().Be(0);
        result.Value.G.Should().Be(0);
        result.Value.B.Should().Be(255);
    }

    [Fact]
    public void ParseColor_Null_ReturnsNull()
    {
        StyleParser.ParseColor(null!).Should().BeNull();
    }

    [Fact]
    public void ParseColor_Empty_ReturnsNull()
    {
        StyleParser.ParseColor("").Should().BeNull();
    }
}

#endregion

#region ParseFontWeight Tests
// NOTE: Rust normalizes font-weight keywords to numeric strings before serialization:
//   "bold" → "700", "normal" → "400", "bolder" → "600", "lighter" → "300"
// C# ParseFontWeight only receives numeric strings; keyword inputs default to Normal.

public class StyleResolver_ParseFontWeightTests
{
    [Fact]
    public void ParseFontWeight_700_ReturnsBoldFromNumeric()
    {
        StyleParser.ParseFontWeight("700").Should().Be(FontWeight.Bold);
    }

    [Fact]
    public void ParseFontWeight_400_ReturnsNormalFromNumeric()
    {
        StyleParser.ParseFontWeight("400").Should().Be(FontWeight.Normal);
    }

    [Fact]
    public void ParseFontWeight_Keywords_DefaultToNormal()
    {
        // Keywords are normalized by Rust; if somehow they reach C#, return Normal safely.
        StyleParser.ParseFontWeight("bold").Should().Be(FontWeight.Normal);
        StyleParser.ParseFontWeight("normal").Should().Be(FontWeight.Normal);
    }

    [Fact]
    public void ParseFontWeight_100_ReturnsThin()
    {
        StyleParser.ParseFontWeight("100").Should().Be(FontWeight.Thin);
    }

    [Fact]
    public void ParseFontWeight_200_ReturnsExtraLight()
    {
        StyleParser.ParseFontWeight("200").Should().Be(FontWeight.ExtraLight);
    }

    [Fact]
    public void ParseFontWeight_300_ReturnsLight()
    {
        StyleParser.ParseFontWeight("300").Should().Be(FontWeight.Light);
    }

    [Fact]
    public void ParseFontWeight_500_ReturnsMedium()
    {
        StyleParser.ParseFontWeight("500").Should().Be(FontWeight.Medium);
    }

    [Fact]
    public void ParseFontWeight_600_ReturnsSemiBold()
    {
        StyleParser.ParseFontWeight("600").Should().Be(FontWeight.SemiBold);
    }

    [Fact]
    public void ParseFontWeight_800_ReturnsExtraBold()
    {
        StyleParser.ParseFontWeight("800").Should().Be(FontWeight.ExtraBold);
    }

    [Fact]
    public void ParseFontWeight_900_ReturnsBlack()
    {
        StyleParser.ParseFontWeight("900").Should().Be(FontWeight.Black);
    }

    [Fact]
    public void ParseFontWeight_Unknown_DefaultsToNormal()
    {
        StyleParser.ParseFontWeight("ultralight").Should().Be(FontWeight.Normal);
        StyleParser.ParseFontWeight("xyz").Should().Be(FontWeight.Normal);
    }
}

#endregion
