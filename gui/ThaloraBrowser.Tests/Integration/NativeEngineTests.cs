using ThaloraBrowser.Services;

namespace ThaloraBrowser.Tests.Integration;

[Trait("Category", "Integration")]
public class NativeEngineTests
{
    private static bool NativeLibraryAvailable()
    {
        try
        {
            using var engine = new ThaloraBrowserEngine();
            return true;
        }
        catch
        {
            return false;
        }
    }

    [SkippableFact]
    public void Engine_InitSucceeds()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        // If we get here without exception, initialization succeeded
    }

    [SkippableFact]
    public async Task Navigate_HttpbinHtml_ReturnsHtml()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        var html = await engine.NavigateAsync("https://httpbin.org/html");

        html.Should().NotBeNullOrWhiteSpace();
        html.Should().Contain("html");
    }

    [SkippableFact]
    public async Task GetPageTitle_AfterNavigate_ReturnsNonNull()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        await engine.NavigateAsync("https://httpbin.org/html");
        var title = await engine.GetPageTitleAsync();

        title.Should().NotBeNull();
    }

    [SkippableFact]
    public async Task GoBack_NoHistory_ReturnsFalse()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        var result = await engine.GoBackAsync();

        result.Should().BeFalse();
    }

    [SkippableFact]
    public async Task GoForward_NoHistory_ReturnsFalse()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        var result = await engine.GoForwardAsync();

        result.Should().BeFalse();
    }

    [SkippableFact]
    public async Task ExecuteJavaScript_ReturnsResult()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        var result = await engine.ExecuteJavaScriptAsync("1 + 1");

        result.Should().NotBeNull();
    }

    [SkippableFact]
    public void GetLastError_NoError_ReturnsNull()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        using var engine = new ThaloraBrowserEngine();
        var error = engine.GetLastError();

        error.Should().BeNull();
    }

    [SkippableFact]
    public void DoubleDispose_DoesNotThrow()
    {
        Skip.IfNot(NativeLibraryAvailable(), "Native library not available");

        var engine = new ThaloraBrowserEngine();

        var act = () =>
        {
            engine.Dispose();
            engine.Dispose();
        };

        act.Should().NotThrow();
    }
}
