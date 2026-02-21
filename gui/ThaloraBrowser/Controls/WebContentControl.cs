using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Controls;

/// <summary>
/// Custom Avalonia control that renders HTML content using layout computed by the Rust engine.
/// Handles scrolling, link clicks, hover cursor changes, and viewport resizing.
/// </summary>
public class WebContentControl : Control
{
    private HtmlRenderer? _renderer;
    private double _scrollOffsetY;
    private double _maxScrollY;
    private string? _hoveredLink;
    private bool _isRendering;
    private bool _renderPending;

    /// <summary>
    /// Fired when a link is clicked in the rendered content.
    /// </summary>
    public event EventHandler<LinkClickedEventArgs>? LinkClicked;

    /// <summary>
    /// Fired when the hovered link changes (for status bar updates).
    /// </summary>
    public event EventHandler<string?>? HoveredLinkChanged;

    public static readonly StyledProperty<string?> HtmlContentProperty =
        AvaloniaProperty.Register<WebContentControl, string?>(nameof(HtmlContent));

    public static readonly StyledProperty<string?> BaseUrlProperty =
        AvaloniaProperty.Register<WebContentControl, string?>(nameof(BaseUrl));

    public static readonly StyledProperty<IThaloraBrowserEngine?> EngineProperty =
        AvaloniaProperty.Register<WebContentControl, IThaloraBrowserEngine?>(nameof(Engine));

    public string? HtmlContent
    {
        get => GetValue(HtmlContentProperty);
        set => SetValue(HtmlContentProperty, value);
    }

    public string? BaseUrl
    {
        get => GetValue(BaseUrlProperty);
        set => SetValue(BaseUrlProperty, value);
    }

    public IThaloraBrowserEngine? Engine
    {
        get => GetValue(EngineProperty);
        set => SetValue(EngineProperty, value);
    }

    static WebContentControl()
    {
        HtmlContentProperty.Changed.AddClassHandler<WebContentControl>((c, _) => c.OnHtmlContentChanged());
        BaseUrlProperty.Changed.AddClassHandler<WebContentControl>((c, _) => c.OnHtmlContentChanged());

        // Enable focus for keyboard input
        FocusableProperty.OverrideDefaultValue<WebContentControl>(true);
    }

    public WebContentControl()
    {
        _renderer = new HtmlRenderer();
        ClipToBounds = true;
    }

    protected override void OnSizeChanged(SizeChangedEventArgs e)
    {
        base.OnSizeChanged(e);

        // On viewport resize, recompute layout from Rust
        if (_renderer?.CurrentLayout != null && !_isRendering)
        {
            OnHtmlContentChanged(); // Re-trigger layout with new size
        }
    }

    public override void Render(DrawingContext context)
    {
        base.Render(context);

        // Draw default background
        IBrush canvasBg = new SolidColorBrush(Color.FromRgb(30, 30, 30));

        if (_renderer?.CurrentLayout != null)
        {
            // CSS canvas background propagation: use the root element's or body's
            // background color to fill the entire viewport.
            var rootBg = GetCanvasBackground(_renderer.CurrentLayout);
            if (rootBg != null)
                canvasBg = rootBg;
        }

        context.DrawRectangle(canvasBg, null, new Rect(Bounds.Size));

        if (_renderer?.CurrentLayout != null)
        {
            _renderer.PaintContext.SetPaintContext(BaseUrl, () => Avalonia.Threading.Dispatcher.UIThread.Post(InvalidateVisual));
            _renderer.PaintContext.Paint(context, _renderer.CurrentLayout, _scrollOffsetY);
        }
        else
        {
            // Draw "No content" message
            var typeface = new Typeface(FontFamily.Default);
            var text = new FormattedText(
                "Navigate to a URL to get started",
                System.Globalization.CultureInfo.CurrentCulture,
                FlowDirection.LeftToRight,
                typeface,
                16,
                new SolidColorBrush(Color.FromRgb(120, 120, 120))
            );

            var x = (Bounds.Width - text.Width) / 2;
            var y = (Bounds.Height - text.Height) / 2;
            context.DrawText(text, new Point(x, y));
        }
    }

    protected override void OnPointerWheelChanged(PointerWheelEventArgs e)
    {
        base.OnPointerWheelChanged(e);

        _scrollOffsetY -= e.Delta.Y * 40; // 40px per scroll notch
        _scrollOffsetY = Math.Clamp(_scrollOffsetY, 0, _maxScrollY);
        InvalidateVisual();
        e.Handled = true;
    }

    protected override void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e);
        Focus();

        if (_renderer?.CurrentLayout == null)
            return;

        var point = e.GetPosition(this);
        var layoutPoint = new Point(point.X, point.Y + _scrollOffsetY);

        var hit = _renderer.HitTester.HitTest(layoutPoint, _renderer.CurrentLayout);
        if (hit?.LinkHref != null)
        {
            var resolvedUrl = _renderer.ResolveUrl(hit.LinkHref);
            if (resolvedUrl != null)
            {
                LinkClicked?.Invoke(this, new LinkClickedEventArgs(resolvedUrl));
                e.Handled = true;
            }
        }
    }

    protected override void OnPointerMoved(PointerEventArgs e)
    {
        base.OnPointerMoved(e);

        if (_renderer?.CurrentLayout == null)
        {
            Cursor = Cursor.Default;
            return;
        }

        var point = e.GetPosition(this);
        var layoutPoint = new Point(point.X, point.Y + _scrollOffsetY);

        var link = _renderer.HitTester.FindLinkAt(layoutPoint, _renderer.CurrentLayout);

        if (link != _hoveredLink)
        {
            _hoveredLink = link;
            Cursor = link != null ? new Cursor(StandardCursorType.Hand) : Cursor.Default;
            HoveredLinkChanged?.Invoke(this, _renderer.ResolveUrl(link));
        }
    }

    protected override void OnPointerExited(PointerEventArgs e)
    {
        base.OnPointerExited(e);
        if (_hoveredLink != null)
        {
            _hoveredLink = null;
            Cursor = Cursor.Default;
            HoveredLinkChanged?.Invoke(this, null);
        }
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);

        switch (e.Key)
        {
            case Key.Up:
                _scrollOffsetY = Math.Max(0, _scrollOffsetY - 40);
                InvalidateVisual();
                e.Handled = true;
                break;
            case Key.Down:
                _scrollOffsetY = Math.Min(_maxScrollY, _scrollOffsetY + 40);
                InvalidateVisual();
                e.Handled = true;
                break;
            case Key.PageUp:
                _scrollOffsetY = Math.Max(0, _scrollOffsetY - Bounds.Height);
                InvalidateVisual();
                e.Handled = true;
                break;
            case Key.PageDown:
                _scrollOffsetY = Math.Min(_maxScrollY, _scrollOffsetY + Bounds.Height);
                InvalidateVisual();
                e.Handled = true;
                break;
            case Key.Home:
                _scrollOffsetY = 0;
                InvalidateVisual();
                e.Handled = true;
                break;
            case Key.End:
                _scrollOffsetY = _maxScrollY;
                InvalidateVisual();
                e.Handled = true;
                break;
        }
    }

    private async void OnHtmlContentChanged()
    {
        if (_isRendering)
        {
            _renderPending = true;
            return;
        }
        _isRendering = true;

        try
        {
            if (string.IsNullOrEmpty(HtmlContent))
            {
                _renderer?.Dispose();
                _renderer = new HtmlRenderer();
                _scrollOffsetY = 0;
                _maxScrollY = 0;
                InvalidateVisual();
                return;
            }

            var engine = Engine ?? (DataContext as ThaloraBrowser.ViewModels.BrowserTabViewModel)?.Engine;
            if (engine == null)
            {
                Console.Error.WriteLine("[WebContentControl] No engine available for layout computation");
                _renderer?.Dispose();
                _renderer = new HtmlRenderer();
                InvalidateVisual();
                return;
            }

            // Compute layout on the Rust side — keep the old renderer visible
            // until the new layout is ready to avoid flicker during resize.
            var viewportW = (float)Math.Max(100, Bounds.Width);
            var viewportH = (float)Math.Max(100, Bounds.Height);

            Console.Error.WriteLine($"[WebContentControl] Computing layout: {viewportW}x{viewportH}, HTML length: {HtmlContent?.Length ?? 0}");

            var layoutJson = await engine.ComputeLayoutAsync(viewportW, viewportH);

            // Now that we have the new layout, swap out the renderer
            var oldRenderer = _renderer;
            _renderer = new HtmlRenderer();

            if (!string.IsNullOrEmpty(layoutJson))
            {
                Console.Error.WriteLine($"[WebContentControl] Layout JSON received: {layoutJson.Length} chars");
                _renderer.RenderFromLayoutJson(layoutJson, BaseUrl);

                if (_renderer.CurrentLayout == null)
                    Console.Error.WriteLine("[WebContentControl] WARNING: RenderFromLayoutJson produced null layout");
            }
            else
            {
                var lastError = engine.GetLastError();
                Console.Error.WriteLine($"[WebContentControl] Layout returned null. Rust error: {lastError ?? "(none)"}");
            }

            // Only reset scroll on actual navigation, not resize
            if (oldRenderer?.CurrentLayout == null)
                _scrollOffsetY = 0;

            oldRenderer?.Dispose();
            UpdateScrollBounds();
            InvalidateVisual();
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[WebContentControl] Error in OnHtmlContentChanged: {ex}");
            InvalidateVisual();
        }
        finally
        {
            _isRendering = false;

            // If a render was requested while we were busy, process it now
            if (_renderPending)
            {
                _renderPending = false;
                OnHtmlContentChanged();
            }
        }
    }

    private void UpdateScrollBounds()
    {
        if (_renderer?.CurrentLayout != null)
        {
            _maxScrollY = Math.Max(0, _renderer.ContentHeight - Bounds.Height);
        }
        else
        {
            _maxScrollY = 0;
        }
        _scrollOffsetY = Math.Clamp(_scrollOffsetY, 0, _maxScrollY);
    }

    /// <summary>
    /// CSS canvas background propagation: if the root element (html) has a background,
    /// use it. Otherwise, check the body element. This matches browser behavior where
    /// the body/html background fills the entire viewport.
    /// </summary>
    private static IBrush? GetCanvasBackground(Rendering.LayoutBox root)
    {
        // Check root element (html)
        if (root.Style.BackgroundColor != null)
            return root.Style.BackgroundColor;

        // Check body (first block child of root)
        foreach (var child in root.Children)
        {
            if (child.Style.BackgroundColor != null)
                return child.Style.BackgroundColor;
        }

        return null;
    }
}

/// <summary>
/// Event args for link click events.
/// </summary>
public class LinkClickedEventArgs : EventArgs
{
    public string Url { get; }
    public LinkClickedEventArgs(string url) { Url = url; }
}
