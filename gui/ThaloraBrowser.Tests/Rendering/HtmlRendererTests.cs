using Avalonia;
using ThaloraBrowser.Rendering;
using ThaloraBrowser.Tests.Helpers;

namespace ThaloraBrowser.Tests.Rendering;

[Trait("Category", "AvaloniaHeadless")]
[Collection("Avalonia")]
public class HtmlRendererTests : IDisposable
{
    private readonly HtmlRenderer _renderer;

    public HtmlRendererTests()
    {
        AvaloniaTestApp.EnsureInitialized();
        _renderer = new HtmlRenderer();
    }

    private static readonly Size Viewport = new(800, 600);

    // ---------------------------------------------------------------
    // ResolveUrl tests
    // ---------------------------------------------------------------

    [Fact]
    public void ResolveUrl_AbsoluteUrl_PassedThrough()
    {
        var result = _renderer.ResolveUrl("https://example.com/page");

        result.Should().Be("https://example.com/page");
    }

    [Fact]
    public void ResolveUrl_Null_ReturnsNull()
    {
        var result = _renderer.ResolveUrl(null);

        result.Should().BeNull();
    }

    [Fact]
    public void ResolveUrl_Empty_ReturnsNull()
    {
        var result = _renderer.ResolveUrl("");

        result.Should().BeNull();
    }

    [Fact]
    public async Task ResolveUrl_RelativeUrl_ResolvedAgainstBaseUrl()
    {
        await _renderer.RenderPageAsync(
            "<html><body><p>Test</p></body></html>",
            "https://example.com/dir/page.html",
            Viewport);

        var result = _renderer.ResolveUrl("/other.html");

        // The resolved URL should contain the origin and the relative path
        result.Should().NotBeNull();
        result.Should().Contain("other.html");
    }

    // ---------------------------------------------------------------
    // ContentHeight tests
    // ---------------------------------------------------------------

    [Fact]
    public void ContentHeight_NoLayout_ReturnsZero()
    {
        _renderer.ContentHeight.Should().Be(0);
    }

    [Fact]
    public async Task ContentHeight_AfterRender_ReturnsPositive()
    {
        await _renderer.RenderPageAsync(
            "<html><body><p>Hello world</p></body></html>",
            "https://example.com",
            Viewport);

        _renderer.ContentHeight.Should().BeGreaterThan(0);
    }

    [Fact]
    public async Task ContentHeight_TallContent_LargerThanViewportHeight()
    {
        // Build HTML with many paragraphs to ensure content exceeds viewport
        var paragraphs = string.Concat(Enumerable.Range(1, 100).Select(i => $"<p>Paragraph {i} with some text content to ensure height.</p>"));
        var html = $"<html><body>{paragraphs}</body></html>";

        await _renderer.RenderPageAsync(html, "https://example.com", Viewport);

        _renderer.ContentHeight.Should().BeGreaterThan(Viewport.Height);
    }

    // ---------------------------------------------------------------
    // RenderPageAsync tests
    // ---------------------------------------------------------------

    [Fact]
    public async Task RenderPageAsync_ValidHtml_ReturnsNonNullLayoutBox()
    {
        var result = await _renderer.RenderPageAsync(
            "<html><body><h1>Title</h1><p>Content</p></body></html>",
            "https://example.com",
            Viewport);

        result.Should().NotBeNull();
        _renderer.CurrentLayout.Should().NotBeNull();
        _renderer.CurrentLayout.Should().BeSameAs(result);
    }

    [Fact]
    public async Task RenderPageAsync_MalformedInput_StillReturnsLayoutBox()
    {
        // Even with unusual input, the renderer should produce a layout (possibly an error page)
        var result = await _renderer.RenderPageAsync(
            "<html><body><div>Unclosed tags<p>Mixed content",
            "https://example.com",
            Viewport);

        result.Should().NotBeNull();
    }

    // ---------------------------------------------------------------
    // RelayoutForViewport tests
    // ---------------------------------------------------------------

    [Fact]
    public void RelayoutForViewport_NoDocument_ReturnsNull()
    {
        var result = _renderer.RelayoutForViewport(new Size(1024, 768));

        result.Should().BeNull();
    }

    public void Dispose()
    {
        _renderer.Dispose();
    }
}
