using Avalonia;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Tests.Helpers;

namespace ThaloraBrowser.Tests.Rendering;

#region MapDisplayToBoxType — Pure tests (no Avalonia needed)

public class LayoutEngine_MapDisplayToBoxTypeTests
{
    [Fact]
    public void MapDisplayToBoxType_Block_ReturnsBlock()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.Block).Should().Be(BoxType.Block);
    }

    [Fact]
    public void MapDisplayToBoxType_Inline_ReturnsInline()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.Inline).Should().Be(BoxType.Inline);
    }

    [Fact]
    public void MapDisplayToBoxType_InlineBlock_ReturnsInlineBlock()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.InlineBlock).Should().Be(BoxType.InlineBlock);
    }

    [Fact]
    public void MapDisplayToBoxType_Flex_ReturnsBlock()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.Flex).Should().Be(BoxType.Block);
    }

    [Fact]
    public void MapDisplayToBoxType_ListItem_ReturnsListItem()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.ListItem).Should().Be(BoxType.ListItem);
    }

    [Fact]
    public void MapDisplayToBoxType_Table_ReturnsTableBox()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.Table).Should().Be(BoxType.TableBox);
    }

    [Fact]
    public void MapDisplayToBoxType_TableRow_ReturnsTableRowBox()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.TableRow).Should().Be(BoxType.TableRowBox);
    }

    [Fact]
    public void MapDisplayToBoxType_TableCell_ReturnsTableCellBox()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.TableCell).Should().Be(BoxType.TableCellBox);
    }

    [Fact]
    public void MapDisplayToBoxType_None_DefaultsToBlock()
    {
        LayoutEngineHelpers.MapDisplayToBoxType(DisplayMode.None).Should().Be(BoxType.Block);
    }
}

#endregion

#region LayoutBox Construction Tests

[Trait("Category", "AvaloniaHeadless")]
[Collection("Avalonia")]
public class LayoutBoxConstructionTests
{
    public LayoutBoxConstructionTests()
    {
        AvaloniaTestApp.EnsureInitialized();
    }

    [Fact]
    public void LayoutBox_MarginBox_IncludesAllEdges()
    {
        var box = new LayoutBox
        {
            Type = BoxType.Block,
            ContentRect = new Rect(50, 50, 200, 100),
            Margin = new Thickness(10),
            Border = new Thickness(2),
            Padding = new Thickness(5),
        };

        box.MarginBox.X.Should().Be(50 - 5 - 2 - 10);
        box.MarginBox.Y.Should().Be(50 - 5 - 2 - 10);
        box.MarginBox.Width.Should().Be(200 + 2 * (5 + 2 + 10));
        box.MarginBox.Height.Should().Be(100 + 2 * (5 + 2 + 10));
    }

    [Fact]
    public void LayoutBox_BorderBox_ExcludesMargin()
    {
        var box = new LayoutBox
        {
            Type = BoxType.Block,
            ContentRect = new Rect(50, 50, 200, 100),
            Margin = new Thickness(10),
            Border = new Thickness(2),
            Padding = new Thickness(5),
        };

        box.BorderBox.X.Should().Be(50 - 5 - 2);
        box.BorderBox.Y.Should().Be(50 - 5 - 2);
    }

    [Fact]
    public void FlattenBoxTree_ReturnsAllDescendants()
    {
        var root = LayoutTestHelper.CreateTestLayoutBox();
        var child1 = LayoutTestHelper.CreateTestLayoutBox();
        var child2 = LayoutTestHelper.CreateTestLayoutBox();
        var grandchild = LayoutTestHelper.CreateTestLayoutBox();

        root.Children.Add(child1);
        root.Children.Add(child2);
        child1.Children.Add(grandchild);

        var all = LayoutTestHelper.FlattenBoxTree(root);
        all.Should().HaveCount(4);
    }
}

#endregion
