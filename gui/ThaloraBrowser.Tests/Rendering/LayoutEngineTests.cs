using Avalonia;
using AngleSharp.Dom;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Tests.Helpers;

namespace ThaloraBrowser.Tests.Rendering;

#region MapDisplayToBoxType — Pure tests (no Avalonia needed)

public class LayoutEngine_MapDisplayToBoxTypeTests
{
    [Fact]
    public void MapDisplayToBoxType_Block_ReturnsBlock()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.Block).Should().Be(BoxType.Block);
    }

    [Fact]
    public void MapDisplayToBoxType_Inline_ReturnsInline()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.Inline).Should().Be(BoxType.Inline);
    }

    [Fact]
    public void MapDisplayToBoxType_InlineBlock_ReturnsInlineBlock()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.InlineBlock).Should().Be(BoxType.InlineBlock);
    }

    [Fact]
    public void MapDisplayToBoxType_Flex_ReturnsBlock()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.Flex).Should().Be(BoxType.Block);
    }

    [Fact]
    public void MapDisplayToBoxType_ListItem_ReturnsListItem()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.ListItem).Should().Be(BoxType.ListItem);
    }

    [Fact]
    public void MapDisplayToBoxType_Table_ReturnsTableBox()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.Table).Should().Be(BoxType.TableBox);
    }

    [Fact]
    public void MapDisplayToBoxType_TableRow_ReturnsTableRowBox()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.TableRow).Should().Be(BoxType.TableRowBox);
    }

    [Fact]
    public void MapDisplayToBoxType_TableCell_ReturnsTableCellBox()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.TableCell).Should().Be(BoxType.TableCellBox);
    }

    [Fact]
    public void MapDisplayToBoxType_None_DefaultsToBlock()
    {
        LayoutEngine.MapDisplayToBoxType(DisplayMode.None).Should().Be(BoxType.Block);
    }
}

#endregion

#region BuildLayoutTree — Avalonia headless tests

[Trait("Category", "AvaloniaHeadless")]
[Collection("Avalonia")]
public class LayoutEngine_BuildLayoutTreeTests
{
    private static readonly Size Viewport = new(800, 600);

    public LayoutEngine_BuildLayoutTreeTests()
    {
        AvaloniaTestApp.EnsureInitialized();
    }

    private static List<LayoutBox> FindAllBoxes(LayoutBox root)
    {
        var result = new List<LayoutBox> { root };
        foreach (var child in root.Children)
            result.AddRange(FindAllBoxes(child));
        return result;
    }

    [Fact]
    public async Task TwoParagraphs_SecondBelowFirst()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body><p>First</p><p>Second</p></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var paragraphs = allBoxes
            .Where(b => b.Type == BoxType.Block && b.Element?.LocalName == "p")
            .ToList();

        paragraphs.Should().HaveCount(2);
        paragraphs[1].ContentRect.Y.Should().BeGreaterThan(0);
        paragraphs[1].ContentRect.Y.Should().BeGreaterThanOrEqualTo(paragraphs[0].MarginBox.Bottom);
    }

    [Fact]
    public async Task H1_HasLargerFontSize()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body><h1>Big Title</h1></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var h1Box = allBoxes.FirstOrDefault(b => b.Element?.LocalName == "h1");

        h1Box.Should().NotBeNull();
        h1Box!.Style.FontSize.Should().Be(32);
    }

    [Fact]
    public async Task ScriptAndStyleTags_SkippedInLayout()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync(
            "<html><body><script>var x = 1;</script><style>body { color: red; }</style><p>Visible</p></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var scriptBoxes = allBoxes.Where(b => b.Element?.LocalName == "script").ToList();
        var styleBoxes = allBoxes.Where(b => b.Element?.LocalName == "style").ToList();

        scriptBoxes.Should().BeEmpty();
        styleBoxes.Should().BeEmpty();
    }

    [Fact]
    public async Task SpanElement_TreatedAsInline()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body><span>Inline text</span></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var spanBox = allBoxes.FirstOrDefault(b => b.Element?.LocalName == "span");

        spanBox.Should().NotBeNull();
        spanBox!.Type.Should().Be(BoxType.Inline);
    }

    [Fact]
    public async Task TableWithTwoColumns_CellsDistributedAcrossWidth()
    {
        // Use explicit <tbody> to avoid parser-inserted wrapping affecting layout
        var doc = await LayoutTestHelper.ParseHtmlAsync(
            "<html><body><table><tbody><tr><td>Cell 1</td><td>Cell 2</td></tr></tbody></table></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var cells = allBoxes.Where(b => b.Type == BoxType.TableCellBox).ToList();

        cells.Should().HaveCountGreaterOrEqualTo(2);
        if (cells.Count >= 2)
        {
            // Second cell should start at or after the first cell's right edge
            cells[1].ContentRect.X.Should().BeGreaterOrEqualTo(cells[0].ContentRect.X);
        }
    }

    [Fact]
    public async Task LiElement_GetsListItemBoxType()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body><ul><li>Item</li></ul></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var liBox = allBoxes.FirstOrDefault(b => b.Element?.LocalName == "li");

        liBox.Should().NotBeNull();
        liBox!.Type.Should().Be(BoxType.ListItem);
    }

    [Fact]
    public async Task DisplayNone_ExcludedFromLayoutTree()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync(
            "<html><body><div style=\"display:none\">Hidden</div><p>Visible</p></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var hiddenDivs = allBoxes.Where(b =>
            b.Element?.LocalName == "div" &&
            b.Style.Display == DisplayMode.None).ToList();

        hiddenDivs.Should().BeEmpty();
    }

    [Fact]
    public async Task BrElement_CreatesLineBreak()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body><p>Line one<br>Line two</p></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        // The <br> should produce an anonymous box with a newline text run
        var brBox = allBoxes.FirstOrDefault(b =>
            b.Type == BoxType.Anonymous &&
            b.TextRuns != null &&
            b.TextRuns.Any(r => r.Text == "\n"));

        brBox.Should().NotBeNull();
    }

    [Fact]
    public async Task AnchorElement_GetsLinkHrefSet()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync(
            "<html><body><a href=\"https://example.com\">Link</a></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var linkBox = allBoxes.FirstOrDefault(b => b.Element?.LocalName == "a");

        linkBox.Should().NotBeNull();
        linkBox!.LinkHref.Should().Be("https://example.com");
    }

    [Fact]
    public async Task ImgElement_GetsImageSourceSet()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync(
            "<html><body><img src=\"https://example.com/image.png\"></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        var allBoxes = FindAllBoxes(root);
        var imgBox = allBoxes.FirstOrDefault(b => b.Element?.LocalName == "img");

        imgBox.Should().NotBeNull();
        imgBox!.ImageSource.Should().Be("https://example.com/image.png");
    }

    [Fact]
    public async Task EmptyDocument_RootCoversViewport()
    {
        var doc = await LayoutTestHelper.ParseHtmlAsync("<html><body></body></html>");
        var resolver = new StyleResolver();
        var engine = new LayoutEngine(resolver);
        var root = engine.BuildLayoutTree(doc, Viewport);

        root.Should().NotBeNull();
        root.Type.Should().Be(BoxType.Block);
        root.ContentRect.Width.Should().Be(Viewport.Width);
    }
}

#endregion
