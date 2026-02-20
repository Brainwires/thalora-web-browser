using AngleSharp;
using AngleSharp.Css;
using AngleSharp.Dom;
using Avalonia;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Orchestrates the HTML/CSS rendering pipeline:
/// Parse (AngleSharp) -> Style (CSS computation) -> Layout (box positioning) -> Paint (Avalonia drawing)
/// </summary>
public class HtmlRenderer : IDisposable
{
    private readonly IBrowsingContext _browsingContext;
    private readonly StyleResolver _styleResolver;
    private readonly LayoutEngine _layoutEngine;
    private readonly PaintContext _paintContext;
    private readonly HitTester _hitTester;
    private readonly ImageCache _imageCache;

    private IDocument? _currentDocument;
    private LayoutBox? _currentLayout;

    public HtmlRenderer()
    {
        // Configure AngleSharp with CSS support
        var config = Configuration.Default
            .WithDefaultLoader()
            .WithCss();

        _browsingContext = BrowsingContext.New(config);
        _styleResolver = new StyleResolver();
        _layoutEngine = new LayoutEngine(_styleResolver);
        _hitTester = new HitTester();
        _imageCache = new ImageCache();
        _paintContext = new PaintContext(_imageCache);
    }

    /// <summary>
    /// The current layout tree (null if no page has been rendered).
    /// </summary>
    public LayoutBox? CurrentLayout => _currentLayout;

    /// <summary>
    /// The paint context for drawing.
    /// </summary>
    public PaintContext PaintContext => _paintContext;

    /// <summary>
    /// The hit tester for coordinate-to-element mapping.
    /// </summary>
    public HitTester HitTester => _hitTester;

    /// <summary>
    /// The image cache for downloaded images.
    /// </summary>
    public ImageCache ImageCache => _imageCache;

    /// <summary>
    /// Total content height of the current layout (for scrolling).
    /// </summary>
    public double ContentHeight
    {
        get
        {
            if (_currentLayout == null) return 0;
            return CalculateTotalHeight(_currentLayout);
        }
    }

    /// <summary>
    /// Parse HTML and perform layout for the given viewport.
    /// This is the main entry point — call this when page content changes.
    /// </summary>
    public async Task<LayoutBox?> RenderPageAsync(string html, string baseUrl, Size viewport)
    {
        try
        {
            // Parse HTML with AngleSharp
            _currentDocument = await _browsingContext.OpenAsync(req =>
            {
                req.Content(html);
                if (!string.IsNullOrEmpty(baseUrl))
                    req.Address(baseUrl);
            });

            if (_currentDocument == null)
                return null;

            // Build the layout tree
            _currentLayout = _layoutEngine.BuildLayoutTree(_currentDocument, viewport);
            return _currentLayout;
        }
        catch (Exception ex)
        {
            // On parse/layout failure, render an error page
            var errorHtml = $"<html><body><h1>Rendering Error</h1><pre>{System.Net.WebUtility.HtmlEncode(ex.Message)}</pre></body></html>";
            _currentDocument = await _browsingContext.OpenAsync(req => req.Content(errorHtml));
            _currentLayout = _layoutEngine.BuildLayoutTree(_currentDocument!, viewport);
            return _currentLayout;
        }
    }

    /// <summary>
    /// Re-layout the current document for a new viewport size.
    /// Faster than a full re-render since we skip HTML parsing.
    /// </summary>
    public LayoutBox? RelayoutForViewport(Size viewport)
    {
        if (_currentDocument == null)
            return null;

        _currentLayout = _layoutEngine.BuildLayoutTree(_currentDocument, viewport);
        return _currentLayout;
    }

    /// <summary>
    /// Resolve a potentially relative URL against the current document's base URL.
    /// </summary>
    public string? ResolveUrl(string? href)
    {
        if (string.IsNullOrEmpty(href))
            return null;

        // Already absolute
        if (Uri.TryCreate(href, UriKind.Absolute, out var absolute))
            return absolute.ToString();

        // Resolve relative to document base
        if (_currentDocument?.BaseUri != null && Uri.TryCreate(_currentDocument.BaseUri, UriKind.Absolute, out var baseUri))
        {
            if (Uri.TryCreate(baseUri, href, out var resolved))
                return resolved.ToString();
        }

        return href;
    }

    /// <summary>
    /// Calculate total height of a layout tree for scrolling.
    /// </summary>
    private static double CalculateTotalHeight(LayoutBox box)
    {
        double maxBottom = box.MarginBox.Bottom;

        foreach (var child in box.Children)
        {
            maxBottom = Math.Max(maxBottom, CalculateTotalHeight(child));
        }

        if (box.TextRuns != null)
        {
            foreach (var run in box.TextRuns)
            {
                maxBottom = Math.Max(maxBottom, run.Bounds.Bottom);
            }
        }

        return maxBottom;
    }

    public void Dispose()
    {
        _currentDocument?.Dispose();
        _imageCache.Dispose();
    }
}
