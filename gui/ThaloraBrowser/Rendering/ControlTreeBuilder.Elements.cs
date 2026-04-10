using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using System.Globalization;
using System.Runtime.InteropServices;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Element-specific builders: TextBlock, Image, HR, list items.
/// These handle individual HTML elements that need special rendering.
/// </summary>
public partial class ControlTreeBuilder
{
    /// <summary>Apply CSS text-transform to a string.</summary>
    internal static string ApplyTextTransform(string text, string? transform) =>
        transform?.ToLowerInvariant() switch
        {
            "uppercase" => text.ToUpperInvariant(),
            "lowercase" => text.ToLowerInvariant(),
            "capitalize" => CultureInfo.CurrentCulture.TextInfo.ToTitleCase(text.ToLowerInvariant()),
            _ => text,
        };

    /// <summary>
    /// Build a simple TextBlock for a text-only element.
    /// </summary>
    private Control BuildTextBlock(StyledElement element, double fontSize)
    {
        var text = element.TextContent ?? "";
        if (string.IsNullOrWhiteSpace(text))
            return new Panel(); // Empty placeholder

        text = ApplyTextTransform(text, element.Styles.TextTransform);

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
            // Propagate MaxWidth/MaxHeight so the Border doesn't stretch full-width
            if (!hasExplicitWidth && !hasExplicitHeight)
            {
                border.MaxWidth = image.MaxWidth;
                border.MaxHeight = image.MaxHeight;
            }
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
            // SVG images — load via Svg.Skia and convert to Avalonia bitmap.
            // Check extension on the final path component (before any query params),
            // not a substring match — e.g. ".svg.png" is a PNG, not an SVG.
            var srcPath = src.Split('?')[0].Split('#')[0];
            if (srcPath.EndsWith(".svg", StringComparison.OrdinalIgnoreCase))
            {
                await LoadSvgImageAsync(imageControl, src);
                return;
            }

            var bitmap = await _imageCache.GetImageAsync(src, _baseUrl);
            if (bitmap != null)
            {
                await Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(() =>
                {
                    imageControl.Source = bitmap;
                });
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[ImageLoad] Failed to load '{src}': {ex.Message}");
        }
    }

    /// <summary>
    /// Load an SVG image via Svg.Skia library, rasterize to bitmap, and display in Image control.
    /// </summary>
    private async Task LoadSvgImageAsync(Avalonia.Controls.Image imageControl, string src)
    {
        try
        {
            // Resolve URL — handle relative paths (/static/...), protocol-relative (//...), etc.
            var resolvedUrl = src;
            if (!src.StartsWith("http", StringComparison.OrdinalIgnoreCase))
            {
                if (_baseUrl != null && Uri.TryCreate(new Uri(_baseUrl), src, out var fullUri))
                    resolvedUrl = fullUri.ToString();
                else if (src.StartsWith("//"))
                    resolvedUrl = "https:" + src;
            }

            // Download SVG bytes — must set User-Agent or Wikimedia returns 403
            using var httpClient = new System.Net.Http.HttpClient();
            httpClient.Timeout = TimeSpan.FromSeconds(10);
            httpClient.DefaultRequestHeaders.Add("User-Agent", BuildSvgUserAgent());
            var svgBytes = await httpClient.GetByteArrayAsync(resolvedUrl);

            // Parse SVG with Svg.Skia
            var svg = new Svg.Skia.SKSvg();
            using var stream = new System.IO.MemoryStream(svgBytes);
            var picture = svg.Load(stream);
            if (picture == null) return;

            // Determine render size
            var bounds = picture.CullRect;
            int width = Math.Max((int)bounds.Width, 1);
            int height = Math.Max((int)bounds.Height, 1);

            // Rasterize to SkiaSharp bitmap
            using var skBitmap = new SkiaSharp.SKBitmap(width, height);
            using var canvas = new SkiaSharp.SKCanvas(skBitmap);
            canvas.Clear(SkiaSharp.SKColors.Transparent);
            canvas.DrawPicture(picture);
            canvas.Flush();

            // Convert to Avalonia bitmap
            using var skImage = SkiaSharp.SKImage.FromBitmap(skBitmap);
            using var data = skImage.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
            using var memStream = new System.IO.MemoryStream(data.ToArray());

            var avBitmap = new Bitmap(memStream);

            await Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(() =>
            {
                imageControl.Source = avBitmap;
            });
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[SVG] Failed to load SVG {src}: {ex.Message}");
        }
    }

    /// <summary>
    /// Render an inline SVG element by loading its serialized markup via Svg.Skia.
    /// Falls back to a sized placeholder if rendering fails.
    /// </summary>
    private Control BuildInlineSvg(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var svgMarkup = element.SvgContent!;

        // Parse explicit dimensions from styles
        var widthVal = Len(styles.Width, fontSize);
        var heightVal = Len(styles.Height, fontSize);

        try
        {
            var svg = new Svg.Skia.SKSvg();
            using var stream = new System.IO.MemoryStream(System.Text.Encoding.UTF8.GetBytes(svgMarkup));
            var picture = svg.Load(stream);
            if (picture == null)
                return BuildInlineSvgPlaceholder(element, fontSize);

            var bounds = picture.CullRect;
            int w = widthVal.HasValue && widthVal.Value > 0 ? (int)widthVal.Value : Math.Max((int)bounds.Width, 1);
            int h = heightVal.HasValue && heightVal.Value > 0 ? (int)heightVal.Value : Math.Max((int)bounds.Height, 1);

            // Scale to explicit dimensions if provided
            using var skBitmap = new SkiaSharp.SKBitmap(w, h);
            using var canvas = new SkiaSharp.SKCanvas(skBitmap);
            canvas.Clear(SkiaSharp.SKColors.Transparent);
            if (bounds.Width > 0 && bounds.Height > 0)
            {
                float scaleX = w / bounds.Width;
                float scaleY = h / bounds.Height;
                canvas.Scale(scaleX, scaleY);
                canvas.Translate(-bounds.Left, -bounds.Top);
            }
            canvas.DrawPicture(picture);
            canvas.Flush();

            using var skImage = SkiaSharp.SKImage.FromBitmap(skBitmap);
            using var data = skImage.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
            using var ms = new System.IO.MemoryStream(data.ToArray());
            var bitmap = new Bitmap(ms);

            var image = new Avalonia.Controls.Image
            {
                Source = bitmap,
                Width = w,
                Height = h,
                Stretch = Stretch.Uniform,
                HorizontalAlignment = HorizontalAlignment.Left,
            };
            return image;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[InlineSVG] Failed to render SVG: {ex.Message}");
            return BuildInlineSvgPlaceholder(element, fontSize);
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

    /// <summary>
    /// Helper to get an HTML attribute from StyledElement.
    /// </summary>
    private static string? GetAttr(StyledElement element, string name)
    {
        if (element.Attributes == null) return null;
        return element.Attributes.TryGetValue(name, out var val) ? val : null;
    }

    /// <summary>
    /// Build an &lt;input&gt; element. Maps to various Avalonia controls based on type attribute.
    /// </summary>
    private Control? BuildInputElement(StyledElement element, double fontSize)
    {
        var inputType = GetAttr(element, "type")?.ToLowerInvariant() ?? "text";
        var placeholder = GetAttr(element, "placeholder");
        var value = GetAttr(element, "value") ?? "";
        var styles = element.Styles;

        switch (inputType)
        {
            case "hidden":
                return null;

            case "checkbox":
            {
                var cb = new CheckBox
                {
                    IsChecked = GetAttr(element, "checked") != null,
                    FontSize = fontSize,
                    // Always set foreground for web content — dark theme default is invisible on light pages
                    Foreground = Brushes.Black,
                };
                if (!string.IsNullOrEmpty(styles.Color))
                {
                    var brush = StyleParser.ParseBrush(styles.Color);
                    if (brush != null) cb.Foreground = brush;
                }
                return cb;
            }

            case "radio":
            {
                var rb = new RadioButton
                {
                    IsChecked = GetAttr(element, "checked") != null,
                    FontSize = fontSize,
                    Foreground = Brushes.Black,
                };
                if (!string.IsNullOrEmpty(styles.Color))
                {
                    var brush = StyleParser.ParseBrush(styles.Color);
                    if (brush != null) rb.Foreground = brush;
                }
                return rb;
            }

            case "submit":
            case "button":
            case "reset":
            {
                var buttonText = value;
                if (string.IsNullOrEmpty(buttonText))
                {
                    buttonText = inputType == "submit" ? "Submit"
                        : inputType == "reset" ? "Reset" : "Button";
                }
                // Use CSS background if available, otherwise web-standard light grey
                var btnBg = StyleParser.ParseBrush(styles.BackgroundColor)
                    ?? new SolidColorBrush(Color.FromRgb(248, 249, 250));
                var btnFg = StyleParser.ParseBrush(styles.Color) ?? Brushes.Black;
                var btn = new Button
                {
                    Content = buttonText,
                    FontSize = fontSize,
                    Padding = new Thickness(8, 4),
                    VerticalAlignment = VerticalAlignment.Center,
                    Background = btnBg,
                    Foreground = btnFg,
                };
                return btn;
            }

            default:
            {
                // text, search, email, password, url, tel, number, etc.
                // Width, background, border, padding, and dimensions are now handled
                // by the outer Border wrapper (WrapInBorder) in BuildControl.
                // Set BorderThickness=0 to prevent Avalonia's default TextBox border
                // from doubling up with the CSS border on the outer Border.
                // Explicitly set white background — Avalonia's dark theme would otherwise
                // bleed through on light web pages that don't specify a background-color.
                var textBg = StyleParser.ParseBrush(styles.BackgroundColor) ?? Brushes.White;
                var textFg = StyleParser.ParseBrush(styles.Color) ?? Brushes.Black;
                var textBox = new TextBox
                {
                    Text = value,
                    Watermark = placeholder,
                    FontSize = fontSize,
                    FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily),
                    VerticalAlignment = VerticalAlignment.Center,
                    BorderThickness = new Thickness(0),
                    MinWidth = 120,
                    Background = textBg,
                    Foreground = textFg,
                };

                return textBox;
            }
        }
    }

    /// <summary>
    /// Build a &lt;button&gt; element → Avalonia Button.
    /// </summary>
    private Control BuildButtonElement(StyledElement element, double fontSize)
    {
        var styles = element.Styles;

        var btnBg = StyleParser.ParseBrush(styles.BackgroundColor)
            ?? new SolidColorBrush(Color.FromRgb(248, 249, 250));
        var btnFg = StyleParser.ParseBrush(styles.Color) ?? Brushes.Black;
        var btn = new Button
        {
            FontSize = fontSize,
            FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily),
            Padding = new Thickness(8, 4),
            VerticalAlignment = VerticalAlignment.Center,
            Background = btnBg,
            Foreground = btnFg,
        };

        // If the button has children with inline content, build a TextBlock with
        // styled Runs as the button content. This preserves per-child font-size,
        // color, etc. — e.g., a 6px caret icon inside a 12px button.
        if (element.Children.Count > 0)
        {
            var textBlock = new SelectableTextBlock();
            ApplyTextProperties(textBlock, styles, fontSize);
            foreach (var child in element.Children)
                AddInlineContent(textBlock.Inlines!, child, fontSize);

            // If the textblock has content, use it; otherwise fall back to flat text
            if (textBlock.Inlines!.Count > 0)
            {
                btn.Content = textBlock;
                return btn;
            }
        }

        // Simple button with flat text content
        var text = CollectInlineText(element).Trim();
        if (string.IsNullOrEmpty(text))
            text = "Button";
        btn.Content = text;

        return btn;
    }

    /// <summary>
    /// Build a &lt;select&gt; element → Avalonia ComboBox.
    /// </summary>
    private Control BuildSelectElement(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var comboBg = StyleParser.ParseBrush(styles.BackgroundColor) ?? Brushes.White;
        var comboFg = StyleParser.ParseBrush(styles.Color) ?? Brushes.Black;
        var comboBox = new ComboBox
        {
            FontSize = fontSize,
            FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily),
            VerticalAlignment = VerticalAlignment.Center,
            MinWidth = 80,
            Background = comboBg,
            Foreground = comboFg,
        };

        // Extract <option> children as items
        int selectedIndex = 0;
        int idx = 0;
        foreach (var child in element.Children)
        {
            if (child.Tag == "option")
            {
                var optionText = CollectInlineText(child).Trim();
                if (string.IsNullOrEmpty(optionText))
                    optionText = GetAttr(child, "value") ?? "";

                comboBox.Items.Add(optionText);

                if (GetAttr(child, "selected") != null)
                    selectedIndex = idx;
                idx++;
            }
            else if (child.Tag == "optgroup")
            {
                // Flatten optgroup children into the combo box
                foreach (var optChild in child.Children)
                {
                    if (optChild.Tag == "option")
                    {
                        var optText = CollectInlineText(optChild).Trim();
                        comboBox.Items.Add(optText);
                        idx++;
                    }
                }
            }
        }

        if (comboBox.Items.Count > 0)
            comboBox.SelectedIndex = selectedIndex;

        return comboBox;
    }

    /// <summary>
    /// Build a &lt;textarea&gt; element → Avalonia TextBox with AcceptsReturn.
    /// </summary>
    private Control BuildTextareaElement(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var text = CollectInlineText(element);

        // Width, height, border, padding are now handled by the outer Border wrapper.
        // Set BorderThickness=0 to prevent double-border.
        var areaBg = StyleParser.ParseBrush(styles.BackgroundColor) ?? Brushes.White;
        var areaFg = StyleParser.ParseBrush(styles.Color) ?? Brushes.Black;
        var textBox = new TextBox
        {
            Text = text,
            AcceptsReturn = true,
            FontSize = fontSize,
            FontFamily = StyleParser.ResolveFontFamily(styles.FontFamily),
            MinHeight = 60,
            MinWidth = 200,
            TextWrapping = TextWrapping.Wrap,
            BorderThickness = new Thickness(0),
            Background = areaBg,
            Foreground = areaFg,
        };

        return textBox;
    }

    /// <summary>
    /// Build a placeholder control for &lt;audio&gt; elements.
    /// Renders a simple play button with "Audio" label. Full playback is future work.
    /// </summary>
    private Control BuildAudioPlaceholder(StyledElement element, double fontSize)
    {
        var playButton = new Button
        {
            Content = "\u25B6 Audio",
            FontSize = fontSize,
            Padding = new Thickness(8, 4),
            VerticalAlignment = VerticalAlignment.Center,
            IsEnabled = false, // Placeholder — no playback yet
        };

        return playButton;
    }

    /// <summary>
    /// Build a placeholder panel for inline &lt;svg&gt; elements.
    /// Uses the SVG's declared width/height for sizing.
    /// </summary>
    private Control? BuildInlineSvgPlaceholder(StyledElement element, double fontSize)
    {
        var styles = element.Styles;
        var w = Len(styles.Width, fontSize) ?? 24;
        var h = Len(styles.Height, fontSize) ?? 24;

        return new Panel
        {
            Width = w,
            Height = h,
            MinWidth = 1,
            MinHeight = 1,
        };
    }

    /// <summary>
    /// Build a Chrome-compatible User-Agent string reflecting the actual OS and CPU architecture.
    /// Used for SVG HTTP requests to bypass servers (e.g. Wikimedia) that return 403 for unknown agents.
    /// </summary>
    private static string BuildSvgUserAgent()
    {
        var arch = RuntimeInformation.ProcessArchitecture;
        string archStr = arch switch
        {
            Architecture.Arm64 => "aarch64",
            Architecture.Arm   => "armv7l",
            Architecture.X86   => "i686",
            _                  => "x86_64",
        };

        string osPart;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            // macOS — architecture exposed as Intel or Apple Silicon
            osPart = arch == Architecture.Arm64
                ? "Macintosh; Apple Silicon Mac OS X 14_0"
                : "Macintosh; Intel Mac OS X 10_15_7";
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            osPart = arch == Architecture.Arm64
                ? "Windows NT 10.0; ARM64"
                : "Windows NT 10.0; Win64; x64";
        }
        else
        {
            // Linux and others
            osPart = $"X11; Linux {archStr}";
        }

        return $"Mozilla/5.0 ({osPart}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    }
}
