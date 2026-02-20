namespace ThaloraBrowser.Services;

/// <summary>
/// Interface for the Thalora browser engine, enabling dependency injection and testability.
/// </summary>
public interface IThaloraBrowserEngine : IDisposable
{
    Task<string?> NavigateAsync(string url);
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
    string? GetLastError();
}
