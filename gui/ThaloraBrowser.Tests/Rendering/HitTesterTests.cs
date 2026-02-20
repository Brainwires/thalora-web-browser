using Avalonia;
using ThaloraBrowser.Rendering;

namespace ThaloraBrowser.Tests.Rendering;

public class HitTesterTests
{
    private readonly HitTester _sut = new();

    private static LayoutBox MakeBox(double x, double y, double w, double h,
        string? linkHref = null, bool visible = true)
    {
        return new LayoutBox
        {
            ContentRect = new Rect(x, y, w, h),
            Style = new CssComputedStyle { IsVisible = visible },
            LinkHref = linkHref,
        };
    }

    // 1. Point inside single box returns that box
    [Fact]
    public void HitTest_PointInsideSingleBox_ReturnsBox()
    {
        var box = MakeBox(10, 10, 100, 50);

        var result = _sut.HitTest(new Point(50, 30), box);

        result.Should().NotBeNull();
        result!.Box.Should().BeSameAs(box);
    }

    // 2. Point outside all boxes returns null
    [Fact]
    public void HitTest_PointOutsideAllBoxes_ReturnsNull()
    {
        var box = MakeBox(10, 10, 100, 50);

        var result = _sut.HitTest(new Point(500, 500), box);

        result.Should().BeNull();
    }

    // 3. Nested boxes: child returned over parent
    [Fact]
    public void HitTest_NestedBoxes_ChildReturnedOverParent()
    {
        var child = MakeBox(20, 20, 40, 30);
        var parent = MakeBox(10, 10, 100, 80);
        parent.Children.Add(child);

        var result = _sut.HitTest(new Point(30, 30), parent);

        result.Should().NotBeNull();
        result!.Box.Should().BeSameAs(child);
    }

    // 4. Z-order: last child wins over earlier sibling at same point
    [Fact]
    public void HitTest_OverlappingSiblings_LastChildWins()
    {
        var firstChild = MakeBox(10, 10, 80, 60);
        var lastChild = MakeBox(10, 10, 80, 60);
        var parent = MakeBox(0, 0, 200, 200);
        parent.Children.Add(firstChild);
        parent.Children.Add(lastChild);

        var result = _sut.HitTest(new Point(50, 40), parent);

        result.Should().NotBeNull();
        result!.Box.Should().BeSameAs(lastChild, "last child has highest z-order");
    }

    // 5. Invisible box is skipped
    [Fact]
    public void HitTest_InvisibleBox_ReturnsNull()
    {
        var box = MakeBox(10, 10, 100, 50, visible: false);

        var result = _sut.HitTest(new Point(50, 30), box);

        result.Should().BeNull();
    }

    // 6. Invisible parent with visible child: child still skipped
    [Fact]
    public void HitTest_InvisibleParentWithVisibleChild_ReturnsNull()
    {
        var child = MakeBox(20, 20, 40, 30, visible: true);
        var parent = MakeBox(10, 10, 100, 80, visible: false);
        parent.Children.Add(child);

        var result = _sut.HitTest(new Point(30, 30), parent);

        result.Should().BeNull("parent is invisible so recursion returns null at parent level");
    }

    // 7. Text run hit: point inside TextRun.Bounds returns result with TextRun
    [Fact]
    public void HitTest_PointInsideTextRun_ReturnsResultWithTextRun()
    {
        var textRun = new TextRun
        {
            Text = "Hello",
            Bounds = new Rect(15, 15, 60, 20),
        };
        var box = MakeBox(10, 10, 100, 50);
        box.TextRuns = new List<TextRun> { textRun };

        var result = _sut.HitTest(new Point(40, 25), box);

        result.Should().NotBeNull();
        result!.TextRun.Should().BeSameAs(textRun);
        result.Box.Should().BeSameAs(box);
    }

    // 8. Text run with LinkHref returns LinkHref in result
    [Fact]
    public void HitTest_TextRunWithLink_ReturnsLinkHref()
    {
        var textRun = new TextRun
        {
            Text = "Click me",
            Bounds = new Rect(15, 15, 60, 20),
            LinkHref = "https://example.com",
        };
        var box = MakeBox(10, 10, 100, 50);
        box.TextRuns = new List<TextRun> { textRun };

        var result = _sut.HitTest(new Point(40, 25), box);

        result.Should().NotBeNull();
        result!.LinkHref.Should().Be("https://example.com");
    }

    // 9. Box with LinkHref but no text runs returns LinkHref in result
    [Fact]
    public void HitTest_BoxWithLinkHrefNoTextRuns_ReturnsLinkHref()
    {
        var box = MakeBox(10, 10, 100, 50, linkHref: "https://box-link.com");

        var result = _sut.HitTest(new Point(50, 30), box);

        result.Should().NotBeNull();
        result!.LinkHref.Should().Be("https://box-link.com");
        result.TextRun.Should().BeNull();
    }

    // 10. FindLinkAt: linked text returns href string
    [Fact]
    public void FindLinkAt_LinkedText_ReturnsHref()
    {
        var textRun = new TextRun
        {
            Text = "Link text",
            Bounds = new Rect(15, 15, 60, 20),
            LinkHref = "https://found.com",
        };
        var box = MakeBox(10, 10, 100, 50);
        box.TextRuns = new List<TextRun> { textRun };

        var href = _sut.FindLinkAt(new Point(40, 25), box);

        href.Should().Be("https://found.com");
    }

    // 11. FindLinkAt: non-linked area returns null
    [Fact]
    public void FindLinkAt_NonLinkedArea_ReturnsNull()
    {
        var box = MakeBox(10, 10, 100, 50);

        var href = _sut.FindLinkAt(new Point(50, 30), box);

        href.Should().BeNull();
    }

    // 12. Empty children, zero-size box returns null
    [Fact]
    public void HitTest_ZeroSizeBox_ReturnsNull()
    {
        var box = MakeBox(10, 10, 0, 0);

        var result = _sut.HitTest(new Point(10, 10), box);

        result.Should().BeNull("box with zero Width/Height fails the size check");
    }

    // 13. Deeply nested tree (3 levels): innermost box returned
    [Fact]
    public void HitTest_DeeplyNestedTree_InnermostBoxReturned()
    {
        var innermost = MakeBox(30, 30, 20, 20);
        var middle = MakeBox(20, 20, 60, 60);
        middle.Children.Add(innermost);
        var outer = MakeBox(10, 10, 100, 100);
        outer.Children.Add(middle);

        var result = _sut.HitTest(new Point(35, 35), outer);

        result.Should().NotBeNull();
        result!.Box.Should().BeSameAs(innermost, "innermost box should win in a 3-level tree");
    }

    // 14. Multiple siblings, point only in second returns second
    [Fact]
    public void HitTest_MultipleSiblings_PointInSecond_ReturnsSecond()
    {
        var first = MakeBox(10, 10, 40, 40);
        var second = MakeBox(100, 10, 40, 40);
        var parent = MakeBox(0, 0, 200, 200);
        parent.Children.Add(first);
        parent.Children.Add(second);

        var result = _sut.HitTest(new Point(120, 30), parent);

        result.Should().NotBeNull();
        result!.Box.Should().BeSameAs(second);
    }

    // 15. Box with zero-size ContentRect but non-zero padding has hittable BorderBox
    [Fact]
    public void HitTest_ZeroContentRectWithPadding_BorderBoxIsHittable()
    {
        var box = MakeBox(50, 50, 0, 0);
        box.Padding = new Thickness(10); // BorderBox becomes (40,40,20,20)

        // Point inside the padding area (BorderBox) but outside the zero-size ContentRect
        var result = _sut.HitTest(new Point(45, 45), box);

        result.Should().NotBeNull("BorderBox includes padding and has non-zero size");
        result!.Box.Should().BeSameAs(box);
    }
}
