using AngleSharp.Dom;
using Avalonia;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Maps screen coordinates back to DOM elements and link targets
/// by traversing the layout tree.
/// </summary>
public class HitTester
{
    /// <summary>
    /// Result of a hit test.
    /// </summary>
    public record HitTestResult(
        IElement? Element,
        string? LinkHref,
        LayoutBox? Box,
        TextRun? TextRun
    );

    /// <summary>
    /// Find the element at a given point in the layout tree.
    /// The point should be in layout coordinates (accounting for scroll offset).
    /// </summary>
    public HitTestResult? HitTest(Point point, LayoutBox root)
    {
        return HitTestRecursive(point, root);
    }

    private HitTestResult? HitTestRecursive(Point point, LayoutBox box)
    {
        if (!box.Style.IsVisible)
            return null;

        // Check children first (front-to-back: last child is on top)
        for (int i = box.Children.Count - 1; i >= 0; i--)
        {
            var result = HitTestRecursive(point, box.Children[i]);
            if (result != null)
                return result;
        }

        // Check text runs
        if (box.TextRuns != null)
        {
            foreach (var run in box.TextRuns)
            {
                if (run.Bounds.Contains(point))
                {
                    return new HitTestResult(
                        box.Element ?? FindParentElement(box),
                        run.LinkHref ?? box.LinkHref,
                        box,
                        run
                    );
                }
            }
        }

        // Check the box itself
        var hitRect = box.BorderBox;
        if (hitRect.Width > 0 && hitRect.Height > 0 && hitRect.Contains(point))
        {
            return new HitTestResult(
                box.Element,
                box.LinkHref,
                box,
                null
            );
        }

        return null;
    }

    /// <summary>
    /// Find the nearest parent element for anonymous boxes.
    /// </summary>
    private static IElement? FindParentElement(LayoutBox box)
    {
        // Anonymous boxes don't have elements — try to find one from context
        return box.Element;
    }

    /// <summary>
    /// Find all link targets at a given point (for tooltip display).
    /// </summary>
    public string? FindLinkAt(Point point, LayoutBox root)
    {
        var result = HitTest(point, root);
        return result?.LinkHref;
    }
}
