namespace ThaloraBrowser.Services;

/// <summary>
/// Interface for the Thalora browser engine, enabling dependency injection and testability.
/// </summary>
public interface IThaloraBrowserEngine : IDisposable
{
    Task<string?> NavigateAsync(string url);
    Task<string?> NavigateStaticAsync(string url);
    Task<bool> ExecutePageScriptsAsync();
    Task<string?> GetCurrentUrlAsync();
    Task<string?> GetPageHtmlAsync();
    Task<bool> GoBackAsync();
    Task<bool> GoForwardAsync();
    Task<string?> ReloadAsync();
    bool CanGoBack { get; }
    bool CanGoForward { get; }
    Task<string?> ExecuteJavaScriptAsync(string code);
    Task<bool> ClickElementAsync(string selector);
    Task<bool> TypeTextAsync(string selector, string text, bool clearFirst = true);
    Task<bool> SubmitFormAsync(string formSelector, string? jsonData = null);
    Task<string?> GetPageTitleAsync();
    Task<string?> ComputeLayoutAsync(float viewportW, float viewportH);
    Task<string?> ComputeStyledTreeAsync(float viewportW, float viewportH);
    string? PollHistoryEvents();
    bool SetNavigationMode(int mode);
    string? GetLastError();
}
