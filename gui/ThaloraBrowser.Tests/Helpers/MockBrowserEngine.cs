using ThaloraBrowser.Services;

namespace ThaloraBrowser.Tests.Helpers;

/// <summary>
/// Manual IThaloraBrowserEngine implementation with configurable return values.
/// Useful for MainWindowViewModel tests where we need a factory.
/// </summary>
public class MockBrowserEngine : IThaloraBrowserEngine
{
    public string? NavigateResult { get; set; } = "<html><body>Mock Page</body></html>";
    public string? CurrentUrl { get; set; } = "https://example.com";
    public string? PageHtml { get; set; } = "<html><body>Mock Page</body></html>";
    public string? PageTitle { get; set; } = "Mock Title";
    public string? JsResult { get; set; }
    public string? LastError { get; set; }
    public bool GoBackResult { get; set; }
    public bool GoForwardResult { get; set; }
    public bool CanGoBackValue { get; set; }
    public bool CanGoForwardValue { get; set; }
    public bool IsDisposed { get; private set; }

    public int NavigateCallCount { get; private set; }
    public int GoBackCallCount { get; private set; }
    public int GoForwardCallCount { get; private set; }
    public int ReloadCallCount { get; private set; }
    public int DisposeCallCount { get; private set; }
    public string? LastNavigatedUrl { get; private set; }

    public bool CanGoBack => CanGoBackValue;
    public bool CanGoForward => CanGoForwardValue;

    public Task<string?> NavigateAsync(string url)
    {
        NavigateCallCount++;
        LastNavigatedUrl = url;
        return Task.FromResult(NavigateResult);
    }

    public Task<string?> NavigateStaticAsync(string url)
    {
        NavigateCallCount++;
        LastNavigatedUrl = url;
        return Task.FromResult(NavigateResult);
    }

    public Task<bool> ExecutePageScriptsAsync() => Task.FromResult(true);

    public Task<string?> GetCurrentUrlAsync() => Task.FromResult(CurrentUrl);
    public Task<string?> GetPageHtmlAsync() => Task.FromResult(PageHtml);

    public Task<bool> GoBackAsync()
    {
        GoBackCallCount++;
        return Task.FromResult(GoBackResult);
    }

    public Task<bool> GoForwardAsync()
    {
        GoForwardCallCount++;
        return Task.FromResult(GoForwardResult);
    }

    public Task<string?> ReloadAsync()
    {
        ReloadCallCount++;
        return Task.FromResult(NavigateResult);
    }

    public Task<string?> ExecuteJavaScriptAsync(string code) => Task.FromResult(JsResult);
    public Task<bool> ClickElementAsync(string selector) => Task.FromResult(true);
    public Task<bool> TypeTextAsync(string selector, string text, bool clearFirst = true) => Task.FromResult(true);
    public Task<bool> SubmitFormAsync(string formSelector, string? jsonData = null) => Task.FromResult(true);
    public Task<string?> GetPageTitleAsync() => Task.FromResult(PageTitle);
    public Task<string?> ComputeLayoutAsync(float viewportW, float viewportH) => Task.FromResult<string?>(null);
    public Task<string?> ComputeStyledTreeAsync(float viewportW, float viewportH) => Task.FromResult<string?>(null);
    public string? PollHistoryEvents() => null;
    public bool SetNavigationMode(int mode) => true;
    public string? GetLastError() => LastError;

    public void Dispose()
    {
        DisposeCallCount++;
        IsDisposed = true;
    }
}
