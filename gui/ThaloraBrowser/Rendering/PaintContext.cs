using Avalonia;
using Avalonia.Media;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Paints a LayoutBox tree onto an Avalonia DrawingContext.
/// Handles backgrounds, borders, text, images, list markers, and clipping.
/// </summary>
public class PaintContext
{
    private readonly ImageCache _imageCache;

    public PaintContext(ImageCache imageCache)
    {
        _imageCache = imageCache;
    }

    /// <summary>
    /// Paint the entire layout tree.
    /// </summary>
    public void Paint(DrawingContext ctx, LayoutBox root, double scrollOffsetY = 0)
    {
        using (ctx.PushTransform(Matrix.CreateTranslation(0, -scrollOffsetY)))
        {
            PaintBox(ctx, root);
        }
    }

    private void PaintBox(DrawingContext ctx, LayoutBox box)
    {
        if (!box.Style.IsVisible)
            return;

        // Apply opacity
        if (box.Style.Opacity < 1.0)
        {
            using (ctx.PushOpacity(box.Style.Opacity))
            {
                PaintBoxContent(ctx, box);
            }
        }
        else
        {
            PaintBoxContent(ctx, box);
        }
    }

    private void PaintBoxContent(DrawingContext ctx, LayoutBox box)
    {
        var borderBox = box.BorderBox;

        // 1. Background
        if (box.Style.BackgroundColor != null)
        {
            if (box.Style.BorderRadius != default)
            {
                var rr = new RoundedRect(borderBox, box.Style.BorderRadius.TopLeft, box.Style.BorderRadius.TopRight,
                    box.Style.BorderRadius.BottomRight, box.Style.BorderRadius.BottomLeft);
                ctx.DrawRectangle(box.Style.BackgroundColor, null, rr);
            }
            else
            {
                ctx.DrawRectangle(box.Style.BackgroundColor, null, borderBox);
            }
        }

        // 2. Borders
        PaintBorders(ctx, box, borderBox);

        // 3. List markers
        if (box.Type == BoxType.ListItem)
            PaintListMarker(ctx, box);

        // 4. Clipping for overflow
        bool needsClip = box.Style.Overflow == OverflowMode.Hidden || box.Style.Overflow == OverflowMode.Scroll;
        IDisposable? clipState = null;
        if (needsClip)
        {
            clipState = ctx.PushClip(box.PaddingBox);
        }

        try
        {
            // 5. Text runs
            if (box.TextRuns != null)
            {
                foreach (var run in box.TextRuns)
                {
                    PaintTextRun(ctx, run);
                }
            }

            // 6. Images
            if (box.ImageSource != null)
            {
                PaintImage(ctx, box);
            }

            // 7. Children
            foreach (var child in box.Children)
            {
                PaintBox(ctx, child);
            }
        }
        finally
        {
            clipState?.Dispose();
        }
    }

    private void PaintBorders(DrawingContext ctx, LayoutBox box, Rect borderBox)
    {
        var borderBrush = box.Style.BorderBrush ?? Brushes.Transparent;

        // Top border
        if (box.Border.Top > 0)
        {
            var pen = new Pen(borderBrush, box.Border.Top);
            var y = borderBox.Top + box.Border.Top / 2;
            ctx.DrawLine(pen, new Point(borderBox.Left, y), new Point(borderBox.Right, y));
        }

        // Bottom border
        if (box.Border.Bottom > 0)
        {
            var pen = new Pen(borderBrush, box.Border.Bottom);
            var y = borderBox.Bottom - box.Border.Bottom / 2;
            ctx.DrawLine(pen, new Point(borderBox.Left, y), new Point(borderBox.Right, y));
        }

        // Left border
        if (box.Border.Left > 0)
        {
            var pen = new Pen(borderBrush, box.Border.Left);
            var x = borderBox.Left + box.Border.Left / 2;
            ctx.DrawLine(pen, new Point(x, borderBox.Top), new Point(x, borderBox.Bottom));
        }

        // Right border
        if (box.Border.Right > 0)
        {
            var pen = new Pen(borderBrush, box.Border.Right);
            var x = borderBox.Right - box.Border.Right / 2;
            ctx.DrawLine(pen, new Point(x, borderBox.Top), new Point(x, borderBox.Bottom));
        }
    }

    private void PaintListMarker(DrawingContext ctx, LayoutBox box)
    {
        var markerX = box.ContentRect.X - 20;
        var markerY = box.ContentRect.Y + (box.Style.FontSize * box.Style.LineHeight) / 2;

        switch (box.Style.ListStyleType)
        {
            case ListStyleType.Disc:
                ctx.DrawEllipse(box.Style.Color, null, new Point(markerX, markerY), 3, 3);
                break;
            case ListStyleType.Circle:
                ctx.DrawEllipse(null, new Pen(box.Style.Color, 1), new Point(markerX, markerY), 3, 3);
                break;
            case ListStyleType.Square:
                ctx.DrawRectangle(box.Style.Color, null, new Rect(markerX - 3, markerY - 3, 6, 6));
                break;
            case ListStyleType.Decimal:
            case ListStyleType.LowerAlpha:
            case ListStyleType.UpperAlpha:
                // Determine index: count this box's position among its parent's ListItem children
                int index = 1;
                // We don't have a parent reference, so use a simple heuristic:
                // The list marker index is passed from layout. For now, default to 1.
                // TODO: propagate list-item index from Rust layout if needed

                string marker = box.Style.ListStyleType switch
                {
                    ListStyleType.Decimal => $"{index}.",
                    ListStyleType.LowerAlpha => $"{(char)('a' + index - 1)}.",
                    ListStyleType.UpperAlpha => $"{(char)('A' + index - 1)}.",
                    _ => $"{index}.",
                };

                var typeface = new Typeface(box.Style.FontFamily, box.Style.FontStyle, box.Style.FontWeight);
                var ft = new FormattedText(
                    marker,
                    System.Globalization.CultureInfo.CurrentCulture,
                    FlowDirection.LeftToRight,
                    typeface,
                    box.Style.FontSize,
                    box.Style.Color
                );
                ctx.DrawText(ft, new Point(markerX - ft.Width, box.ContentRect.Y));
                break;
        }
    }

    private static void PaintTextRun(DrawingContext ctx, TextRun run)
    {
        if (string.IsNullOrEmpty(run.Text) || run.Text == "\n")
            return;

        var typeface = new Typeface(run.Style.FontFamily, run.Style.FontStyle, run.Style.FontWeight);
        var formatted = new FormattedText(
            run.Text,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            typeface,
            run.Style.FontSize,
            run.Style.Color
        );

        ctx.DrawText(formatted, run.Bounds.TopLeft);

        // Draw text decorations
        if (run.Style.TextDecorations != null)
        {
            foreach (var decoration in run.Style.TextDecorations)
            {
                var pen = new Pen(run.Style.Color, 1);
                double lineY;
                if (decoration.Location == TextDecorationLocation.Underline)
                    lineY = run.Bounds.Bottom - 2;
                else // Strikethrough
                    lineY = run.Bounds.Top + run.Bounds.Height / 2;

                ctx.DrawLine(pen, new Point(run.Bounds.Left, lineY), new Point(run.Bounds.Left + formatted.Width, lineY));
            }
        }
    }

    private void PaintImage(DrawingContext ctx, LayoutBox box)
    {
        // For now, draw a placeholder rectangle for images
        // Actual image loading happens asynchronously via ImageCache
        var rect = box.ContentRect;
        if (rect.Width <= 0 || rect.Height <= 0)
        {
            // Default image size
            rect = new Rect(rect.X, rect.Y, 200, 150);
        }

        var placeholderBrush = new SolidColorBrush(Color.FromArgb(30, 128, 128, 128));
        var borderPen = new Pen(new SolidColorBrush(Color.FromArgb(60, 128, 128, 128)), 1);
        ctx.DrawRectangle(placeholderBrush, borderPen, rect);

        // Draw alt text or "[img]" placeholder
        var altText = "[img]";
        var typeface = new Typeface(FontFamily.Default, FontStyle.Italic, FontWeight.Normal);
        var ft = new FormattedText(
            altText,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            typeface,
            12,
            new SolidColorBrush(Color.FromRgb(150, 150, 150))
        );
        ctx.DrawText(ft, new Point(rect.X + 4, rect.Y + 4));
    }
}
