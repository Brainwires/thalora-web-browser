using System.Text.Json;
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

    // Sample layout JSON matching Rust LayoutResult format
    private static string CreateTestLayoutJson(double width = 800, double height = 600, string? text = null)
    {
        var textContent = text != null ? $"\"text_content\":\"{text}\"," : "";
        return $@"{{
            ""width"": {width},
            ""height"": {height},
            ""elements"": [{{
                ""id"": ""e0"",
                ""tag"": ""html"",
                ""x"": 0, ""y"": 0,
                ""width"": {width}, ""height"": {height},
                ""display"": ""block"",
                ""is_visible"": true,
                ""children"": [{{
                    ""id"": ""e1"",
                    ""tag"": ""body"",
                    ""x"": 8, ""y"": 8,
                    ""width"": {width - 16}, ""height"": {height - 16},
                    ""display"": ""block"",
                    ""is_visible"": true,
                    ""margin"": {{""top"": 8, ""right"": 8, ""bottom"": 8, ""left"": 8}},
                    ""children"": [{{
                        ""id"": ""t0"",
                        ""tag"": ""#text"",
                        ""x"": 8, ""y"": 8,
                        ""width"": 100, ""height"": 22,
                        ""display"": ""block"",
                        ""is_visible"": true,
                        {textContent}
                        ""font_size"": 16,
                        ""children"": []
                    }}]
                }}]
            }}]
        }}";
    }

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
    public void ResolveUrl_RelativeUrl_ResolvedAgainstBaseUrl()
    {
        var json = CreateTestLayoutJson();
        _renderer.RenderFromLayoutJson(json, "https://example.com/dir/page.html");

        var result = _renderer.ResolveUrl("/other.html");

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
    public void ContentHeight_AfterRender_ReturnsPositive()
    {
        var json = CreateTestLayoutJson(text: "Hello world");
        _renderer.RenderFromLayoutJson(json, "https://example.com");

        _renderer.ContentHeight.Should().BeGreaterThan(0);
    }

    // ---------------------------------------------------------------
    // RenderFromLayoutJson tests
    // ---------------------------------------------------------------

    [Fact]
    public void RenderFromLayoutJson_ValidJson_ReturnsNonNullLayoutBox()
    {
        var json = CreateTestLayoutJson(text: "Content");

        var result = _renderer.RenderFromLayoutJson(json, "https://example.com");

        result.Should().NotBeNull();
        _renderer.CurrentLayout.Should().NotBeNull();
        _renderer.CurrentLayout.Should().BeSameAs(result);
    }

    [Fact]
    public void RenderFromLayoutJson_InvalidJson_ReturnsErrorBox()
    {
        var result = _renderer.RenderFromLayoutJson("not valid json", "https://example.com");

        result.Should().NotBeNull();
        // Should contain an error indication
        _renderer.CurrentLayout.Should().NotBeNull();
    }

    [Fact]
    public void RenderFromLayoutJson_EmptyElements_ReturnsDefaultBox()
    {
        var json = @"{""width"": 800, ""height"": 600, ""elements"": []}";

        var result = _renderer.RenderFromLayoutJson(json, "https://example.com");

        result.Should().NotBeNull();
    }

    public void Dispose()
    {
        _renderer.Dispose();
    }
}
