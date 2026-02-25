using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Element-specific builders: TextBlock, Image, HR, list items.
/// These handle individual HTML elements that need special rendering.
/// </summary>
public partial class ControlTreeBuilder
{
    /// <summary>
    /// Build a simple TextBlock for a text-only element.
    /// </summary>
    private Control BuildTextBlock(StyledElement element, double fontSize)
    {
        var text = element.TextContent ?? "";
        if (string.IsNullOrWhiteSpace(text))
            return new Panel(); // Empty placeholder

        var textBlock = new SelectableTextBlock
        {
            Text = text,
        };
        ApplyTextProperties(textBlock, element.Styles, fontSize);
        return textBlock;
    }

    /// <summary>
    /// Build an Image control for an &lt;img&gt; element.
    /// Loads the image asynchronously via ImageCache.
    /// </summary>
    private Control BuildImage(StyledElement element, double fontSize)
    {
        var imgSrc = element.ImgSrc;
        var styles = element.Styles;

        var image = new Avalonia.Controls.Image
        {
            Stretch = Stretch.Uniform,
            HorizontalAlignment = HorizontalAlignment.Left,
        };

        // Handle width
        bool hasExplicitWidth = false;
        if (!string.IsNullOrEmpty(styles.Width) && styles.Width != "auto")
        {
            if (styles.Width.TrimEnd().EndsWith("%"))
            {
                // Percentage width — stretch to fill container
                image.HorizontalAlignment = HorizontalAlignment.Stretch;
                image.Stretch = Stretch.Uniform;
                hasExplicitWidth = true;
            }
            else
            {
                var width = Len(styles.Width, fontSize);
                if (width.HasValue && width.Value > 0)
                {
                    image.Width = width.Value;
                    hasExplicitWidth = true;
                }
            }
        }

        // Handle height
        bool hasExplicitHeight = false;
        if (!string.IsNullOrEmpty(styles.Height) && styles.Height != "auto")
        {
            if (!styles.Height.TrimEnd().EndsWith("%"))
            {
                var height = Len(styles.Height, fontSize);
                if (height.HasValue && height.Value > 0)
                {
                    image.Height = height.Value;
                    hasExplicitHeight = true;
                }
            }
        }

        // Default max size if no dimensions specified
        if (!hasExplicitWidth && !hasExplicitHeight)
        {
            image.MaxWidth = 800;
            image.MaxHeight = 600;
        }

        // Load image asynchronously
        if (!string.IsNullOrEmpty(imgSrc))
        {
            _ = LoadImageAsync(image, imgSrc);
        }

        // Wrap in Border for border-radius clipping
        if (!string.IsNullOrEmpty(styles.BorderRadius))
        {
            var border = new Border
            {
                Child = image,
                ClipToBounds = true,
                CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize),
            };
            if (hasExplicitWidth) border.Width = image.Width;
            if (hasExplicitHeight) border.Height = image.Height;
            return border;
        }

        return image;
    }

    /// <summary>
    /// Create an Image control for use in InlineUIContainer (inside text flow).
    /// Returns the display control (may be Image or Border wrapping Image) and the Image for async loading.
    /// </summary>
    private (Control displayControl, Avalonia.Controls.Image imageControl) CreateInlineImageWithControl(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var imageControl = new Avalonia.Controls.Image
        {
            Stretch = Stretch.Uniform,
            VerticalAlignment = VerticalAlignment.Center,
        };

        // Handle width
        double? imgWidth = null;
        if (!string.IsNullOrEmpty(styles.Width) && styles.Width != "auto"
            && !styles.Width.TrimEnd().EndsWith("%"))
        {
            var w = Len(styles.Width, fontSize);
            if (w.HasValue && w.Value > 0)
            {
                imageControl.Width = w.Value;
                imgWidth = w.Value;
            }
        }

        // Handle height
        double? imgHeight = null;
        if (!string.IsNullOrEmpty(styles.Height) && styles.Height != "auto"
            && !styles.Height.TrimEnd().EndsWith("%"))
        {
            var h = Len(styles.Height, fontSize);
            if (h.HasValue && h.Value > 0)
            {
                imageControl.Height = h.Value;
                imgHeight = h.Value;
            }
        }

        // Wrap in Border for border-radius clipping
        if (!string.IsNullOrEmpty(styles.BorderRadius))
        {
            var border = new Border
            {
                Child = imageControl,
                ClipToBounds = true,
                CornerRadius = StyleParser.ParseBorderRadius(styles.BorderRadius, fontSize),
            };
            if (imgWidth.HasValue) border.Width = imgWidth.Value;
            if (imgHeight.HasValue) border.Height = imgHeight.Value;
            return (border, imageControl);
        }

        return (imageControl, imageControl);
    }

    private async Task LoadImageAsync(Avalonia.Controls.Image imageControl, string src)
    {
        try
        {
            // Skip SVG images — Avalonia Bitmap doesn't support SVG format
            if (src.Contains(".svg", StringComparison.OrdinalIgnoreCase))
                return;

            var bitmap = await _imageCache.GetImageAsync(src, _baseUrl);
            if (bitmap != null)
            {
                await Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(() =>
                {
                    imageControl.Source = bitmap;
                });
            }
        }
        catch
        {
            // Image load failed — leave blank
        }
    }

    /// <summary>
    /// Wrap list item content with a marker (bullet, number, etc.)
    /// Creates a horizontal DockPanel with the marker on the left and content filling the rest.
    /// </summary>
    private static Control BuildListItemWithMarker(Control content, StyledElement element, double fontSize, int ordinalIndex)
    {
        var listStyleType = element.Styles.ListStyleType ?? "disc";

        // Determine marker text
        string markerText;
        switch (listStyleType)
        {
            case "disc":
                markerText = "\u2022"; // bullet •
                break;
            case "circle":
                markerText = "\u25E6"; // white bullet ◦
                break;
            case "square":
                markerText = "\u25AA"; // black small square ▪
                break;
            case "decimal":
                markerText = $"{ordinalIndex}.";
                break;
            case "decimal-leading-zero":
                markerText = $"{ordinalIndex:D2}.";
                break;
            case "lower-alpha":
            case "lower-latin":
                markerText = $"{(char)('a' + (ordinalIndex - 1) % 26)}.";
                break;
            case "upper-alpha":
            case "upper-latin":
                markerText = $"{(char)('A' + (ordinalIndex - 1) % 26)}.";
                break;
            case "lower-roman":
                markerText = $"{ToRoman(ordinalIndex).ToLowerInvariant()}.";
                break;
            case "upper-roman":
                markerText = $"{ToRoman(ordinalIndex)}.";
                break;
            default:
                markerText = "\u2022"; // fallback to bullet
                break;
        }

        var marker = new TextBlock
        {
            Text = markerText,
            FontSize = fontSize,
            VerticalAlignment = VerticalAlignment.Top,
            HorizontalAlignment = HorizontalAlignment.Right,
            MinWidth = fontSize * 1.5,
            TextAlignment = TextAlignment.Right,
            Margin = new Thickness(0, 0, 6, 0),
        };

        if (element.Styles.Color != null)
        {
            var brush = StyleParser.ParseBrush(element.Styles.Color);
            if (brush != null)
                marker.Foreground = brush;
        }

        var dock = new DockPanel();
        DockPanel.SetDock(marker, Dock.Left);
        dock.Children.Add(marker);
        dock.Children.Add(content); // fills remaining space (last child in DockPanel)

        return dock;
    }

    /// <summary>
    /// Convert an integer to a Roman numeral string.
    /// </summary>
    private static string ToRoman(int number)
    {
        if (number <= 0 || number > 3999)
            return number.ToString();

        string[] thousands = { "", "M", "MM", "MMM" };
        string[] hundreds = { "", "C", "CC", "CCC", "CD", "D", "DC", "DCC", "DCCC", "CM" };
        string[] tens = { "", "X", "XX", "XXX", "XL", "L", "LX", "LXX", "LXXX", "XC" };
        string[] ones = { "", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX" };

        return thousands[number / 1000]
            + hundreds[(number % 1000) / 100]
            + tens[(number % 100) / 10]
            + ones[number % 10];
    }

    /// <summary>
    /// Build an HR element — a thin horizontal line.
    /// </summary>
    private Control BuildHorizontalRule(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var bgColor = StyleParser.ParseBrush(styles.BackgroundColor)
            ?? StyleParser.ParseBrush(styles.BorderColor)
            ?? new SolidColorBrush(Avalonia.Media.Color.FromRgb(200, 200, 200));

        return new Border
        {
            Height = 1,
            Background = bgColor,
            Margin = Box(styles.Margin, fontSize),
            HorizontalAlignment = HorizontalAlignment.Stretch,
        };
    }
}
