using Avalonia;
using Avalonia.Media;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Text measurement and line-breaking engine.
/// Uses Avalonia's FormattedText to measure text and split into lines that fit
/// within a given available width.
/// </summary>
public class TextLayoutEngine
{
    /// <summary>
    /// Measure a single line of text and return its size.
    /// </summary>
    public static Size MeasureText(string text, CssComputedStyle style)
    {
        if (string.IsNullOrEmpty(text))
            return new Size(0, style.FontSize * style.LineHeight);

        var typeface = new Typeface(style.FontFamily, style.FontStyle, style.FontWeight);
        var formatted = new FormattedText(
            text,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            typeface,
            style.FontSize,
            style.Color
        );

        return new Size(formatted.Width, formatted.Height);
    }

    /// <summary>
    /// Break text into lines that fit within the given available width.
    /// Returns a list of TextRun objects with positions relative to (0, 0).
    /// </summary>
    public static List<TextRun> BreakIntoLines(
        string text,
        CssComputedStyle style,
        double availableWidth,
        double startX,
        double startY,
        string? linkHref = null)
    {
        var runs = new List<TextRun>();

        if (string.IsNullOrEmpty(text))
            return runs;

        var lineHeight = style.FontSize * style.LineHeight;

        // Handle pre-formatted text
        if (style.WhiteSpace == WhiteSpaceMode.Pre || style.WhiteSpace == WhiteSpaceMode.PreWrap)
        {
            var lines = text.Split('\n');
            double y = startY;
            foreach (var line in lines)
            {
                var lineText = line;
                if (style.WhiteSpace == WhiteSpaceMode.PreWrap && availableWidth > 0)
                {
                    // Pre-wrap: preserve whitespace but allow wrapping
                    var wrappedLines = WrapLine(lineText, style, availableWidth);
                    foreach (var wl in wrappedLines)
                    {
                        var size = MeasureText(wl, style);
                        runs.Add(new TextRun
                        {
                            Text = wl,
                            Style = style,
                            Bounds = new Rect(startX, y, size.Width, lineHeight),
                            LinkHref = linkHref,
                        });
                        y += lineHeight;
                    }
                }
                else
                {
                    var size = MeasureText(lineText, style);
                    runs.Add(new TextRun
                    {
                        Text = lineText,
                        Style = style,
                        Bounds = new Rect(startX, y, size.Width, lineHeight),
                        LinkHref = linkHref,
                    });
                    y += lineHeight;
                }
            }
            return runs;
        }

        // Normal white-space handling: collapse whitespace and word-wrap
        text = CollapseWhitespace(text);

        if (availableWidth <= 0)
        {
            var size = MeasureText(text, style);
            runs.Add(new TextRun
            {
                Text = text,
                Style = style,
                Bounds = new Rect(startX, startY, size.Width, lineHeight),
                LinkHref = linkHref,
            });
            return runs;
        }

        var wrappedResult = WrapLine(text, style, availableWidth - startX);
        double currentY = startY;
        bool firstLine = true;

        foreach (var line in wrappedResult)
        {
            var size = MeasureText(line, style);
            double x = firstLine ? startX : 0;

            runs.Add(new TextRun
            {
                Text = line,
                Style = style,
                Bounds = new Rect(x, currentY, size.Width, lineHeight),
                LinkHref = linkHref,
            });

            currentY += lineHeight;
            firstLine = false;
        }

        return runs;
    }

    /// <summary>
    /// Wrap a single line of text to fit within available width.
    /// </summary>
    internal static List<string> WrapLine(string text, CssComputedStyle style, double availableWidth)
    {
        var lines = new List<string>();
        if (string.IsNullOrEmpty(text))
        {
            lines.Add("");
            return lines;
        }

        var words = text.Split(' ');
        var currentLine = "";

        foreach (var word in words)
        {
            var testLine = currentLine.Length == 0 ? word : currentLine + " " + word;
            var size = MeasureText(testLine, style);

            if (size.Width > availableWidth && currentLine.Length > 0)
            {
                lines.Add(currentLine);
                currentLine = word;
            }
            else
            {
                currentLine = testLine;
            }
        }

        if (currentLine.Length > 0)
            lines.Add(currentLine);

        return lines;
    }

    /// <summary>
    /// Collapse sequences of whitespace into single spaces (CSS white-space: normal behavior).
    /// </summary>
    internal static string CollapseWhitespace(string text)
    {
        if (string.IsNullOrEmpty(text))
            return text;

        var result = new System.Text.StringBuilder(text.Length);
        bool lastWasSpace = false;

        foreach (var c in text)
        {
            if (char.IsWhiteSpace(c))
            {
                if (!lastWasSpace)
                {
                    result.Append(' ');
                    lastWasSpace = true;
                }
            }
            else
            {
                result.Append(c);
                lastWasSpace = false;
            }
        }

        return result.ToString();
    }
}
