using System.Diagnostics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Controls;

/// <summary>
/// Avalonia control that renders HTML content using the new pipeline:
/// Rust (CSS resolution) → styled element tree JSON → Avalonia native controls.
///
/// Uses a ScrollViewer for scrolling instead of manual offset tracking.
/// Avalonia handles layout and rendering natively — no PaintContext or DrawingContext painting.
/// </summary>
public class WebContentControl : UserControl
{
    private readonly ScrollViewer _scrollViewer;
    private readonly StackPanel _contentPanel;
    private readonly ImageCache _imageCache;
    private bool _isRendering;
    private bool _renderPending;
    private bool _hasContent;
    private bool _disposed;
    private string? _lastRenderedHtml;
    private Dictionary<string, string>? _elementSelectors;
    private ElementActionRegistry? _elementActions;

    /// <summary>
    /// Fired when a link is clicked in the rendered content.
    /// </summary>
    public event EventHandler<LinkClickedEventArgs>? LinkClicked;

    /// <summary>
    /// Fired when the hovered link changes (for status bar updates).
    /// </summary>
    public event EventHandler<string?>? HoveredLinkChanged;

    /// <summary>
    /// Fired when a DOM event (click, etc.) should be dispatched to the JS engine.
    /// </summary>
    public event EventHandler<DomEventArgs>? DomEventDispatched;

    /// <summary>
    /// Element ID → CSS selector mapping from the last rendered styled tree.
    /// Used to dispatch DOM events to the JS engine.
    /// </summary>
    public Dictionary<string, string>? ElementSelectors => _elementSelectors;

    /// <summary>
    /// Registry of interactive elements from the last rendered tree.
    /// Used by BrowserControlServer for /click-element, /hover-element, etc.
    /// </summary>
    public ElementActionRegistry? ElementActions => _elementActions;

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

    /// <summary>
    /// Current vertical scroll offset in pixels.
    /// </summary>
    public double ScrollOffsetY => _scrollViewer.Offset.Y;

    /// <summary>
    /// Maximum vertical scroll offset in pixels.
    /// </summary>
    public double MaxScrollY => Math.Max(0, _scrollViewer.Extent.Height - _scrollViewer.Viewport.Height);

    /// <summary>
    /// Total content height of the rendered page in pixels.
    /// </summary>
    public double ContentHeight => _scrollViewer.Extent.Height;

    /// <summary>
    /// Current viewport height in pixels.
    /// </summary>
    public double ViewportHeight => _scrollViewer.Viewport.Height;

    /// <summary>
    /// Current viewport width in pixels.
    /// </summary>
    public double ViewportWidth => _scrollViewer.Viewport.Width;

    /// <summary>
    /// Programmatically set the scroll offset.
    /// The value is clamped to valid range by the ScrollViewer.
    /// </summary>
    public void SetScrollOffset(double y)
    {
        _scrollViewer.Offset = new Vector(_scrollViewer.Offset.X, Math.Max(0, y));
    }

    public WebContentControl()
    {
        _imageCache = new ImageCache();
        ClipToBounds = true;
        Background = Brushes.White;

        // Build the visual tree: ScrollViewer → StackPanel (content host)
        _contentPanel = new StackPanel
        {
            Orientation = Orientation.Vertical,
            HorizontalAlignment = HorizontalAlignment.Stretch,
        };

        _scrollViewer = new ScrollViewer
        {
            Content = _contentPanel,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            HorizontalAlignment = HorizontalAlignment.Stretch,
            VerticalAlignment = VerticalAlignment.Stretch,
        };

        Content = _scrollViewer;

        // Show placeholder
        ShowPlaceholder();
    }

    protected override void OnSizeChanged(SizeChangedEventArgs e)
    {
        base.OnSizeChanged(e);

        // On viewport resize, recompute styled tree from Rust
        // (CSS media queries and viewport-relative units may change)
        if (_hasContent && !_isRendering)
        {
            OnHtmlContentChanged();
        }
    }

    protected override void OnUnloaded(Avalonia.Interactivity.RoutedEventArgs e)
    {
        base.OnUnloaded(e);
        _disposed = true;
    }

    private async void OnHtmlContentChanged()
    {
        if (_disposed) return;
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
                _hasContent = false;
                ShowPlaceholder();
                return;
            }

            var engine = Engine ?? (DataContext as ThaloraBrowser.ViewModels.BrowserTabViewModel)?.Engine;
            if (engine == null)
            {
                Console.Error.WriteLine("[WebContentControl] No engine available for styled tree computation");
                ShowPlaceholder();
                return;
            }

            var viewportW = (float)Math.Max(100, Bounds.Width);
            var viewportH = (float)Math.Max(100, Bounds.Height);

#if DEBUG
            Console.Error.WriteLine($"[WebContentControl] Computing styled tree: {viewportW}x{viewportH}, HTML length: {HtmlContent?.Length ?? 0}");
#endif

            // Show loading indicator for fresh navigations (new HTML content),
            // but not for resize re-renders of the same page.
            // This fires right as the HTML arrives and before the styled tree is built.
            bool isFreshNavigation = HtmlContent != _lastRenderedHtml;
            if (isFreshNavigation)
                ShowLoading();
            _lastRenderedHtml = HtmlContent;

            // Get the styled tree from Rust (HTML parsed, CSS resolved, no positions)
#if DEBUG
            var swTotal = Stopwatch.StartNew();
            var swFfi = Stopwatch.StartNew();
#endif
            var styledTreeJson = await engine.ComputeStyledTreeAsync(viewportW, viewportH);
            if (_disposed) return; // window was closed during the FFI call
#if DEBUG
            swFfi.Stop();
            Console.Error.WriteLine($"[TIMING] C# ComputeStyledTreeAsync (FFI call): {swFfi.ElapsedMilliseconds}ms");
#endif

            if (!string.IsNullOrEmpty(styledTreeJson))
            {
#if DEBUG
                Console.Error.WriteLine($"[WebContentControl] Styled tree JSON received: {styledTreeJson.Length} chars");
#endif

                // Clear stale state before rebuilding — if BuildFromJson throws,
                // we don't want leftover selectors/actions from the previous page.
                _elementSelectors = null;
                _elementActions = null;

                // Build Avalonia control tree from the styled element tree
                var builder = new ControlTreeBuilder(
                    BaseUrl,
                    _imageCache,
                    onLinkClicked: href => OnLinkClicked(href),
                    onHoveredLinkChanged: href => HoveredLinkChanged?.Invoke(this, href),
                    onDomEvent: (eventType, elementId) => OnDomEvent(eventType, elementId)
                );

#if DEBUG
                var swBuild = Stopwatch.StartNew();
#endif
                var controlTree = builder.BuildFromJson(styledTreeJson);
                if (_disposed) return; // window closed during tree build
#if DEBUG
                swBuild.Stop();
                Console.Error.WriteLine($"[TIMING] C# ControlTreeBuilder.BuildFromJson: {swBuild.ElapsedMilliseconds}ms");
                swTotal.Stop();
                Console.Error.WriteLine($"[TIMING] C# Total OnHtmlContentChanged: {swTotal.ElapsedMilliseconds}ms");
#endif

                // Store element selectors for JS event dispatch
                _elementSelectors = builder.ElementSelectors;

                // Store element action registry for programmatic interaction
                _elementActions = builder.ElementActions;

                // Apply canvas background (CSS propagation from html/body)
                if (builder.CanvasBackground != null)
                    Background = builder.CanvasBackground;
                else
                    Background = Brushes.White;

                // Replace content — only reset scroll on actual navigation
                bool wasEmpty = !_hasContent;
                _hasContent = true;

                _contentPanel.Children.Clear();
                if (controlTree != null)
                    _contentPanel.Children.Add(controlTree);

                if (wasEmpty)
                    _scrollViewer.Offset = default;
            }
            else
            {
                var lastError = engine.GetLastError();
                Console.Error.WriteLine($"[WebContentControl] Styled tree returned null. Rust error: {lastError ?? "(none)"}");
                // Surface error to the GUI status bar so the user can see it
                if (DataContext is ThaloraBrowser.ViewModels.BrowserTabViewModel vm)
                    vm.StatusText = $"Render error: {lastError ?? "styled tree returned null"}";
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[WebContentControl] Error in OnHtmlContentChanged: {ex}");
            if (DataContext is ThaloraBrowser.ViewModels.BrowserTabViewModel vm2)
                vm2.StatusText = $"Render error: {ex.Message}";
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

    private void OnDomEvent(string eventType, string elementId)
    {
        DomEventDispatched?.Invoke(this, new DomEventArgs(eventType, elementId));
    }

    private void OnLinkClicked(string href)
    {
        // Resolve relative URLs against base URL.
        // Note: Uri.TryCreate("/path", UriKind.Absolute) returns true on .NET (treated as file path),
        // so we must check the scheme to distinguish real absolute URLs from root-relative paths.
        string? resolvedUrl = null;
        if (Uri.TryCreate(href, UriKind.Absolute, out var absUri)
            && (absUri.Scheme == "http" || absUri.Scheme == "https"))
        {
            resolvedUrl = href;
        }
        else
        {
            // Resolve against BaseUrl (bound to the current page URL)
            var baseUrl = BaseUrl;

            // Fallback: if BaseUrl is empty or has no host, ask the engine directly
            if (string.IsNullOrEmpty(baseUrl)
                || !Uri.TryCreate(baseUrl, UriKind.Absolute, out var checkUri)
                || string.IsNullOrEmpty(checkUri.Host))
            {
                var engine = Engine ?? (DataContext as ThaloraBrowser.ViewModels.BrowserTabViewModel)?.Engine;
                var engineUrl = engine?.GetCurrentUrlAsync().Result;
                if (!string.IsNullOrEmpty(engineUrl))
                    baseUrl = engineUrl;
            }

            if (baseUrl != null && Uri.TryCreate(baseUrl, UriKind.Absolute, out var baseUri))
            {
                if (Uri.TryCreate(baseUri, href, out var resolved))
                    resolvedUrl = resolved.ToString();
            }
        }

#if DEBUG
        Console.Error.WriteLine($"[WebContentControl] OnLinkClicked: href='{href}', BaseUrl='{BaseUrl}', resolved='{resolvedUrl}'");
#endif

        if (resolvedUrl != null)
            LinkClicked?.Invoke(this, new LinkClickedEventArgs(resolvedUrl));
    }

    private void ShowPlaceholder()
    {
        _contentPanel.Children.Clear();
        _contentPanel.Children.Add(new TextBlock
        {
            Text = "Navigate to a URL to get started",
            Foreground = new SolidColorBrush(Color.FromRgb(120, 120, 120)),
            FontSize = 16,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Margin = new Thickness(0, 100, 0, 0),
        });
    }

    private void ShowLoading()
    {
        _contentPanel.Children.Clear();

        // DISABLED: Loading indicator is more disruptive than helpful.
        // _contentPanel.Children.Add(new TextBlock
        // {
        //     Text = "Loading page. Standby...",
        //     Foreground = new SolidColorBrush(Color.FromRgb(100, 100, 100)),
        //     FontSize = 16,
        //     HorizontalAlignment = HorizontalAlignment.Center,
        //     VerticalAlignment = VerticalAlignment.Center,
        //     Margin = new Thickness(0, 100, 0, 0),
        // });
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

/// <summary>
/// Event args for DOM events dispatched from the GUI to the JS engine.
/// </summary>
public class DomEventArgs : EventArgs
{
    public string EventType { get; }
    public string ElementId { get; }
    public DomEventArgs(string eventType, string elementId)
    {
        EventType = eventType;
        ElementId = elementId;
    }
}
