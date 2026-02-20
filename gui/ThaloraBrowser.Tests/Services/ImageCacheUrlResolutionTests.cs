using ThaloraBrowser.Services;

namespace ThaloraBrowser.Tests.Services;

public class ImageCacheUrlResolutionTests
{
    // ---------------------------------------------------------------
    // Absolute URL pass-through
    // ---------------------------------------------------------------

    [Fact]
    public void ResolveUrl_AbsoluteHttpsUrl_ReturnedAsIs()
    {
        var result = ImageCache.ResolveUrl("https://example.com/image.png", null);

        result.Should().StartWith("https://example.com/image.png");
    }

    [Fact]
    public void ResolveUrl_AbsoluteHttpUrl_ReturnedAsIs()
    {
        var result = ImageCache.ResolveUrl("http://example.com/photo.jpg", null);

        result.Should().StartWith("http://example.com/photo.jpg");
    }

    // ---------------------------------------------------------------
    // Relative URL resolution with base URL
    // ---------------------------------------------------------------

    [Fact]
    public void ResolveUrl_RelativePathWithBaseUrl_ResolvesCorrectly()
    {
        var result = ImageCache.ResolveUrl(
            "images/logo.png",
            "https://example.com/pages/index.html");

        result.Should().Be("https://example.com/pages/images/logo.png");
    }

    [Fact]
    public void ResolveUrl_ParentTraversalWithBaseUrl_ResolvesCorrectly()
    {
        var result = ImageCache.ResolveUrl(
            "../page.html",
            "https://example.com/dir/index.html");

        result.Should().Be("https://example.com/page.html");
    }

    [Fact]
    public void ResolveUrl_BaseUrlWithTrailingPath_ResolvesRelativeToDirectory()
    {
        var result = ImageCache.ResolveUrl(
            "style.css",
            "https://example.com/assets/css/main.css");

        result.Should().Be("https://example.com/assets/css/style.css");
    }

    // ---------------------------------------------------------------
    // Null / empty base URL edge cases
    // ---------------------------------------------------------------

    [Fact]
    public void ResolveUrl_NullBaseUrlWithRelative_ReturnsNull()
    {
        var result = ImageCache.ResolveUrl("images/logo.png", null);

        result.Should().BeNull();
    }

    [Fact]
    public void ResolveUrl_NullBaseUrlWithAbsolute_ReturnsUrl()
    {
        var result = ImageCache.ResolveUrl("https://cdn.example.com/file.js", null);

        result.Should().StartWith("https://cdn.example.com/file.js");
    }

    [Fact]
    public void ResolveUrl_EmptyStringUrl_ReturnsNull()
    {
        var result = ImageCache.ResolveUrl("", null);

        result.Should().BeNull();
    }

    // ---------------------------------------------------------------
    // Fragment and query string relative URLs
    // ---------------------------------------------------------------

    [Fact]
    public void ResolveUrl_FragmentOnlyWithBaseUrl_ResolvesWithFragment()
    {
        var result = ImageCache.ResolveUrl(
            "#section",
            "https://example.com/page.html");

        result.Should().Be("https://example.com/page.html#section");
    }

    [Fact]
    public void ResolveUrl_QueryStringRelativeWithBaseUrl_ResolvesWithQuery()
    {
        var result = ImageCache.ResolveUrl(
            "?page=2",
            "https://example.com/search");

        result.Should().Be("https://example.com/search?page=2");
    }
}
