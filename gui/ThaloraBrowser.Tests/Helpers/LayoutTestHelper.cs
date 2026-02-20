using Avalonia;
using ThaloraBrowser.Rendering;

namespace ThaloraBrowser.Tests.Helpers;

/// <summary>
/// Helper for building LayoutBox trees for testing.
/// Since layout is now computed on the Rust side, tests build LayoutBox trees directly.
/// </summary>
public static class LayoutTestHelper
{
    /// <summary>
    /// Create a simple LayoutBox tree for testing.
    /// </summary>
    public static LayoutBox CreateTestLayoutBox(
        BoxType type = BoxType.Block,
        Rect? contentRect = null,
        CssComputedStyle? style = null)
    {
        return new LayoutBox
        {
            Type = type,
            ContentRect = contentRect ?? new Rect(0, 0, 800, 600),
            Style = style ?? new CssComputedStyle(),
        };
    }

    /// <summary>
    /// Flatten a LayoutBox tree into a list for easy querying.
    /// </summary>
    public static List<LayoutBox> FlattenBoxTree(LayoutBox root)
    {
        var result = new List<LayoutBox> { root };
        foreach (var child in root.Children)
        {
            result.AddRange(FlattenBoxTree(child));
        }
        return result;
    }
}
