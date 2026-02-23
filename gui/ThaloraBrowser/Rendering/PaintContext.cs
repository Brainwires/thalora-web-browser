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
    private string? _baseUrl;
    private Action? _requestRepaint;

    public PaintContext(ImageCache imageCache)
    {
        _imageCache = imageCache;
    }

    /// <summary>
    /// Configure the paint context with the current page's base URL and a repaint callback
    /// for triggering redraws when async resources (images) finish loading.
    /// </summary>
    public void SetPaintContext(string? baseUrl, Action? requestRepaint)
    {
        _baseUrl = baseUrl;
        _requestRepaint = requestRepaint;
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
        bool hasBorder = box.Border.Top > 0 || box.Border.Bottom > 0 || box.Border.Left > 0 || box.Border.Right > 0;
        if (!hasBorder)
            return;

        if (box.Style.BorderRadius != default)
        {
            // Rounded borders: stroke a rounded rect with average border width, inset by half pen width
            double avgWidth = (box.Border.Top + box.Border.Bottom + box.Border.Left + box.Border.Right) / 4.0;
            if (avgWidth <= 0) return;
            var pen = new Pen(borderBrush, avgWidth);
            var half = avgWidth / 2;
            var insetRect = new Rect(
                borderBox.X + half,
                borderBox.Y + half,
                Math.Max(0, borderBox.Width - avgWidth),
                Math.Max(0, borderBox.Height - avgWidth)
            );
            // Adjust border radius to account for inset
            var br = box.Style.BorderRadius;
            var rr = new RoundedRect(
                insetRect,
                Math.Max(0, br.TopLeft - half),
                Math.Max(0, br.TopRight - half),
                Math.Max(0, br.BottomRight - half),
                Math.Max(0, br.BottomLeft - half)
            );
            ctx.DrawRectangle(null, pen, rr);
        }
        else
        {
            // Square borders: draw 4 filled rectangles to avoid corner overlap
            // Top border (full width)
            if (box.Border.Top > 0)
            {
                ctx.DrawRectangle(borderBrush, null,
                    new Rect(borderBox.Left, borderBox.Top, borderBox.Width, box.Border.Top));
            }

            // Bottom border (full width)
            if (box.Border.Bottom > 0)
            {
                ctx.DrawRectangle(borderBrush, null,
                    new Rect(borderBox.Left, borderBox.Bottom - box.Border.Bottom, borderBox.Width, box.Border.Bottom));
            }

            // Left border (inset between top and bottom borders)
            if (box.Border.Left > 0)
            {
                ctx.DrawRectangle(borderBrush, null,
                    new Rect(borderBox.Left, borderBox.Top + box.Border.Top,
                        box.Border.Left, Math.Max(0, borderBox.Height - box.Border.Top - box.Border.Bottom)));
            }

            // Right border (inset between top and bottom borders)
            if (box.Border.Right > 0)
            {
                ctx.DrawRectangle(borderBrush, null,
                    new Rect(borderBox.Right - box.Border.Right, borderBox.Top + box.Border.Top,
                        box.Border.Right, Math.Max(0, borderBox.Height - box.Border.Top - box.Border.Bottom)));
            }
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
                int index = box.ListItemIndex;

                string marker = box.Style.ListStyleType switch
                {
                    ListStyleType.Decimal => $"{index}.",
                    ListStyleType.LowerAlpha => $"{(char)('a' + index - 1)}.",
                    ListStyleType.UpperAlpha => $"{(char)('A' + index - 1)}.",
                    _ => $"{index}.",
                };

                Typeface markerTypeface;
                try
                {
                    markerTypeface = new Typeface(box.Style.FontFamily, box.Style.FontStyle, box.Style.FontWeight);
                    _ = markerTypeface.GlyphTypeface;
                }
                catch
                {
                    markerTypeface = new Typeface(FontFamily.Default, box.Style.FontStyle, box.Style.FontWeight);
                }

                var ft = new FormattedText(
                    marker,
                    System.Globalization.CultureInfo.CurrentCulture,
                    FlowDirection.LeftToRight,
                    markerTypeface,
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

        Typeface typeface;
        try
        {
            typeface = new Typeface(run.Style.FontFamily, run.Style.FontStyle, run.Style.FontWeight);
            // Validate by accessing the GlyphTypeface — some font families pass
            // construction but throw on first use. Catch here to fall back cleanly.
            _ = typeface.GlyphTypeface;
        }
        catch
        {
            typeface = new Typeface(FontFamily.Default, run.Style.FontStyle, run.Style.FontWeight);
        }

        var formatted = new FormattedText(
            run.Text,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            typeface,
            run.Style.FontSize,
            run.Style.Color
        );

        // Constrain text to the layout box width so text wraps naturally
        // within the container boundaries.
        if (run.Bounds.Width > 0)
            formatted.MaxTextWidth = run.Bounds.Width;

        // Apply text alignment
        formatted.TextAlignment = run.Style.TextAlign;

        ctx.DrawText(formatted, run.Bounds.TopLeft);

        // Draw text decorations using actual formatted text height for multi-line
        if (run.Style.TextDecorations != null)
        {
            foreach (var decoration in run.Style.TextDecorations)
            {
                var pen = new Pen(run.Style.Color, 1);
                double lineY;
                if (decoration.Location == TextDecorationLocation.Underline)
                    lineY = run.Bounds.Top + formatted.Height - 2;
                else // Strikethrough
                    lineY = run.Bounds.Top + formatted.Height / 2;

                ctx.DrawLine(pen, new Point(run.Bounds.Left, lineY), new Point(run.Bounds.Left + formatted.Width, lineY));
            }
        }
    }

    private void PaintImage(DrawingContext ctx, LayoutBox box)
    {
        var rect = box.ContentRect;
        if (rect.Width <= 0 || rect.Height <= 0)
        {
            // Default image size
            rect = new Rect(rect.X, rect.Y, 200, 150);
        }

        var imageUrl = box.ImageSource!;

        // Try to get cached bitmap
        if (_imageCache.IsCached(imageUrl, _baseUrl))
        {
            var bitmap = _imageCache.GetCachedBitmap(imageUrl, _baseUrl);
            if (bitmap != null)
            {
                ctx.DrawImage(bitmap, new Rect(0, 0, bitmap.PixelSize.Width, bitmap.PixelSize.Height), rect);
                return;
            }
        }

        // Draw placeholder while loading
        var placeholderBrush = new SolidColorBrush(Color.FromArgb(30, 128, 128, 128));
        var borderPen = new Pen(new SolidColorBrush(Color.FromArgb(60, 128, 128, 128)), 1);
        ctx.DrawRectangle(placeholderBrush, borderPen, rect);

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

        // Fire off async image load and request repaint when done
        var repaint = _requestRepaint;
        _ = Task.Run(async () =>
        {
            try
            {
                var bitmap = await _imageCache.GetImageAsync(imageUrl, _baseUrl);
                if (bitmap != null)
                {
                    repaint?.Invoke();
                }
            }
            catch
            {
                // Image load failed — placeholder remains
            }
        });
    }
}
