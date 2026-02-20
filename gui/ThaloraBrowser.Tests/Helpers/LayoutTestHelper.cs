using AngleSharp;
using AngleSharp.Css;
using AngleSharp.Dom;

namespace ThaloraBrowser.Tests.Helpers;

/// <summary>
/// Helper for parsing HTML into AngleSharp IDocument with CSS support.
/// Reused by all layout/rendering tests.
/// </summary>
public static class LayoutTestHelper
{
    private static readonly IBrowsingContext BrowsingContext;

    static LayoutTestHelper()
    {
        var config = Configuration.Default
            .WithDefaultLoader()
            .WithCss();
        BrowsingContext = AngleSharp.BrowsingContext.New(config);
    }

    public static async Task<IDocument> ParseHtmlAsync(string html)
    {
        return await BrowsingContext.OpenAsync(req => req.Content(html));
    }
}
