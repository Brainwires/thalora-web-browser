using Avalonia;
using ThaloraBrowser.Rendering;

namespace ThaloraBrowser.Tests.Rendering;

public class LayoutBoxTests
{
    // ---------------------------------------------------------------
    // MarginBox tests
    // ---------------------------------------------------------------

    [Fact]
    public void MarginBox_AllZeroEdges_EqualsContentRect()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(10, 20, 100, 50),
            Margin = new Thickness(0),
            Border = new Thickness(0),
            Padding = new Thickness(0),
        };

        box.MarginBox.Should().Be(box.ContentRect);
    }

    [Fact]
    public void MarginBox_UniformMargin_ExpandsEvenlyOnAllSides()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(50, 50, 200, 100),
            Margin = new Thickness(10),
            Border = new Thickness(0),
            Padding = new Thickness(0),
        };

        box.MarginBox.X.Should().Be(40);
        box.MarginBox.Y.Should().Be(40);
        box.MarginBox.Width.Should().Be(220);
        box.MarginBox.Height.Should().Be(120);
    }

    [Fact]
    public void MarginBox_AsymmetricEdges_ReflectsDifferentMarginPerSide()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(100, 100, 200, 100),
            Margin = new Thickness(5, 10, 15, 20), // left, top, right, bottom
            Border = new Thickness(0),
            Padding = new Thickness(0),
        };

        box.MarginBox.X.Should().Be(95);       // 100 - 5 (left)
        box.MarginBox.Y.Should().Be(90);       // 100 - 10 (top)
        box.MarginBox.Width.Should().Be(220);  // 200 + 5 + 15
        box.MarginBox.Height.Should().Be(130); // 100 + 10 + 20
    }

    [Fact]
    public void MarginBox_FullBoxModel_IncludesMarginBorderAndPadding()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(50, 50, 100, 80),
            Margin = new Thickness(4, 3, 2, 1),
            Border = new Thickness(1, 2, 3, 4),
            Padding = new Thickness(5, 6, 7, 8),
        };

        // X = 50 - padding.left(5) - border.left(1) - margin.left(4) = 40
        box.MarginBox.X.Should().Be(40);
        // Y = 50 - padding.top(6) - border.top(2) - margin.top(3) = 39
        box.MarginBox.Y.Should().Be(39);
        // Width = 100 + padding(5+7) + border(1+3) + margin(4+2) = 122
        box.MarginBox.Width.Should().Be(122);
        // Height = 80 + padding(6+8) + border(2+4) + margin(3+1) = 104
        box.MarginBox.Height.Should().Be(104);
    }

    [Fact]
    public void MarginBox_ZeroSizeContentRect_StillExpandsByEdges()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(30, 30, 0, 0),
            Margin = new Thickness(5),
            Border = new Thickness(2),
            Padding = new Thickness(3),
        };

        // X = 30 - 3 - 2 - 5 = 20
        box.MarginBox.X.Should().Be(20);
        // Y = 30 - 3 - 2 - 5 = 20
        box.MarginBox.Y.Should().Be(20);
        // Width = 0 + (3+3) + (2+2) + (5+5) = 20
        box.MarginBox.Width.Should().Be(20);
        // Height = 0 + (3+3) + (2+2) + (5+5) = 20
        box.MarginBox.Height.Should().Be(20);
    }

    // ---------------------------------------------------------------
    // BorderBox tests
    // ---------------------------------------------------------------

    [Fact]
    public void BorderBox_ZeroPaddingAndBorder_EqualsContentRect()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(10, 20, 300, 150),
            Padding = new Thickness(0),
            Border = new Thickness(0),
        };

        box.BorderBox.Should().Be(box.ContentRect);
    }

    [Fact]
    public void BorderBox_SymmetricBorderAndPadding_ExpandsEvenly()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(50, 50, 100, 60),
            Border = new Thickness(2),
            Padding = new Thickness(8),
        };

        box.BorderBox.X.Should().Be(40);       // 50 - 8 - 2
        box.BorderBox.Y.Should().Be(40);       // 50 - 8 - 2
        box.BorderBox.Width.Should().Be(120);  // 100 + (8+8) + (2+2)
        box.BorderBox.Height.Should().Be(80);  // 60 + (8+8) + (2+2)
    }

    [Fact]
    public void BorderBox_AsymmetricBorderAndPadding_ReflectsPerSideValues()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(40, 40, 120, 80),
            Border = new Thickness(1, 2, 3, 4),
            Padding = new Thickness(5, 6, 7, 8),
        };

        // X = 40 - padding.left(5) - border.left(1) = 34
        box.BorderBox.X.Should().Be(34);
        // Y = 40 - padding.top(6) - border.top(2) = 32
        box.BorderBox.Y.Should().Be(32);
        // Width = 120 + padding(5+7) + border(1+3) = 136
        box.BorderBox.Width.Should().Be(136);
        // Height = 80 + padding(6+8) + border(2+4) = 100
        box.BorderBox.Height.Should().Be(100);
    }

    [Fact]
    public void BorderBox_OnlyBorderNoPadding_ExpandsByBorderOnly()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(20, 20, 60, 40),
            Border = new Thickness(3, 5, 7, 9),
            Padding = new Thickness(0),
        };

        box.BorderBox.X.Should().Be(17);      // 20 - 0 - 3
        box.BorderBox.Y.Should().Be(15);      // 20 - 0 - 5
        box.BorderBox.Width.Should().Be(70);  // 60 + 0 + 0 + 3 + 7
        box.BorderBox.Height.Should().Be(54); // 40 + 0 + 0 + 5 + 9
    }

    // ---------------------------------------------------------------
    // PaddingBox tests
    // ---------------------------------------------------------------

    [Fact]
    public void PaddingBox_ZeroPadding_EqualsContentRect()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(15, 25, 200, 100),
            Padding = new Thickness(0),
        };

        box.PaddingBox.Should().Be(box.ContentRect);
    }

    [Fact]
    public void PaddingBox_SymmetricPadding_ExpandsEvenly()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(50, 50, 100, 60),
            Padding = new Thickness(12),
        };

        box.PaddingBox.X.Should().Be(38);       // 50 - 12
        box.PaddingBox.Y.Should().Be(38);       // 50 - 12
        box.PaddingBox.Width.Should().Be(124);  // 100 + 12 + 12
        box.PaddingBox.Height.Should().Be(84);  // 60 + 12 + 12
    }

    [Fact]
    public void PaddingBox_AsymmetricPadding_ReflectsPerSideValues()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(30, 40, 80, 60),
            Padding = new Thickness(2, 4, 6, 8),
        };

        box.PaddingBox.X.Should().Be(28);      // 30 - 2
        box.PaddingBox.Y.Should().Be(36);      // 40 - 4
        box.PaddingBox.Width.Should().Be(88);  // 80 + 2 + 6
        box.PaddingBox.Height.Should().Be(72); // 60 + 4 + 8
    }

    [Fact]
    public void PaddingBox_ContentAtNonZeroPosition_ShiftsCorrectly()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(200, 300, 50, 50),
            Padding = new Thickness(10, 20, 30, 40),
        };

        box.PaddingBox.X.Should().Be(190);      // 200 - 10
        box.PaddingBox.Y.Should().Be(280);      // 300 - 20
        box.PaddingBox.Width.Should().Be(90);   // 50 + 10 + 30
        box.PaddingBox.Height.Should().Be(110); // 50 + 20 + 40
    }

    // ---------------------------------------------------------------
    // Edge case tests
    // ---------------------------------------------------------------

    [Fact]
    public void EdgeCase_ContentRectAtNegativePosition_ComputesCorrectly()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(-50, -30, 100, 80),
            Margin = new Thickness(5),
            Border = new Thickness(2),
            Padding = new Thickness(3),
        };

        // MarginBox: X = -50 - 3 - 2 - 5 = -60
        box.MarginBox.X.Should().Be(-60);
        box.MarginBox.Y.Should().Be(-40);       // -30 - 3 - 2 - 5
        box.MarginBox.Width.Should().Be(120);   // 100 + 2*(3+2+5)
        box.MarginBox.Height.Should().Be(100);  // 80 + 2*(3+2+5)

        // BorderBox: X = -50 - 3 - 2 = -55
        box.BorderBox.X.Should().Be(-55);
        box.BorderBox.Y.Should().Be(-35);

        // PaddingBox: X = -50 - 3 = -53
        box.PaddingBox.X.Should().Be(-53);
        box.PaddingBox.Y.Should().Be(-33);
    }

    [Fact]
    public void EdgeCase_LargeValues_ComputesWithoutOverflow()
    {
        var box = new LayoutBox
        {
            ContentRect = new Rect(1e6, 1e6, 1e6, 1e6),
            Margin = new Thickness(1e4),
            Border = new Thickness(1e3),
            Padding = new Thickness(1e2),
        };

        box.MarginBox.X.Should().Be(1e6 - 1e2 - 1e3 - 1e4);
        box.MarginBox.Y.Should().Be(1e6 - 1e2 - 1e3 - 1e4);
        box.MarginBox.Width.Should().Be(1e6 + 2 * (1e2 + 1e3 + 1e4));
        box.MarginBox.Height.Should().Be(1e6 + 2 * (1e2 + 1e3 + 1e4));
    }

    [Fact]
    public void EdgeCase_DefaultConstruction_AllRectsAreZero()
    {
        var box = new LayoutBox();

        box.ContentRect.Should().Be(default(Rect));
        box.MarginBox.Should().Be(default(Rect));
        box.BorderBox.Should().Be(default(Rect));
        box.PaddingBox.Should().Be(default(Rect));
        box.Margin.Should().Be(default(Thickness));
        box.Border.Should().Be(default(Thickness));
        box.Padding.Should().Be(default(Thickness));
    }

    [Fact]
    public void EdgeCase_ChildrenListStartsEmpty()
    {
        var box = new LayoutBox();

        box.Children.Should().NotBeNull();
        box.Children.Should().BeEmpty();
    }

    [Fact]
    public void EdgeCase_TextRunsStartsNull()
    {
        var box = new LayoutBox();

        box.TextRuns.Should().BeNull();
    }
}
