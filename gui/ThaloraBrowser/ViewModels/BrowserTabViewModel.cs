using CommunityToolkit.Mvvm.ComponentModel;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.ViewModels;

/// <summary>
/// ViewModel for a single browser tab.
/// Each tab owns its own ThaloraBrowserEngine instance.
/// </summary>
public partial class BrowserTabViewModel : ViewModelBase, IDisposable
{
    private readonly ThaloraBrowserEngine _engine;
    private bool _disposed;

    [ObservableProperty]
    private string _title = "New Tab";

    [ObservableProperty]
    private string _url = "";

    [ObservableProperty]
    private string? _htmlContent;

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private string _statusText = "Ready";

    public Guid Id { get; } = Guid.NewGuid();

    public ThaloraBrowserEngine Engine => _engine;

    public BrowserTabViewModel()
    {
        _engine = new ThaloraBrowserEngine();
    }

    /// <summary>
    /// Navigate to a URL, updating all tab state.
    /// </summary>
    public async Task NavigateAsync(string url)
    {
        if (_disposed) return;

        // Normalize URL — add https:// if no scheme is present
        if (!url.Contains("://"))
            url = "https://" + url;

        IsLoading = true;
        StatusText = $"Loading {url}...";
        Url = url;

        try
        {
            var html = await _engine.NavigateAsync(url);
            HtmlContent = html;

            // Update URL to final (possibly redirected) URL
            var currentUrl = await _engine.GetCurrentUrlAsync();
            if (currentUrl != null)
                Url = currentUrl;

            // Update title
            var title = await _engine.GetPageTitleAsync();
            Title = title ?? TruncateUrl(Url);

            StatusText = Url;
        }
        catch (Exception ex)
        {
            StatusText = $"Error: {ex.Message}";
            HtmlContent = $"<html><body><h1>Error</h1><p>{System.Net.WebUtility.HtmlEncode(ex.Message)}</p></body></html>";
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Go back in navigation history.
    /// </summary>
    public async Task<bool> GoBackAsync()
    {
        if (_disposed) return false;

        IsLoading = true;
        try
        {
            var success = await _engine.GoBackAsync();
            if (success)
                await RefreshStateAsync();
            return success;
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Go forward in navigation history.
    /// </summary>
    public async Task<bool> GoForwardAsync()
    {
        if (_disposed) return false;

        IsLoading = true;
        try
        {
            var success = await _engine.GoForwardAsync();
            if (success)
                await RefreshStateAsync();
            return success;
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Reload the current page.
    /// </summary>
    public async Task ReloadAsync()
    {
        if (_disposed) return;

        IsLoading = true;
        StatusText = "Reloading...";
        try
        {
            var html = await _engine.ReloadAsync();
            HtmlContent = html;

            var title = await _engine.GetPageTitleAsync();
            Title = title ?? TruncateUrl(Url);

            StatusText = Url;
        }
        catch (Exception ex)
        {
            StatusText = $"Error: {ex.Message}";
        }
        finally
        {
            IsLoading = false;
        }
    }

    /// <summary>
    /// Refresh all state from the engine after back/forward navigation.
    /// </summary>
    private async Task RefreshStateAsync()
    {
        var html = await _engine.GetPageHtmlAsync();
        HtmlContent = html;

        var currentUrl = await _engine.GetCurrentUrlAsync();
        if (currentUrl != null)
            Url = currentUrl;

        var title = await _engine.GetPageTitleAsync();
        Title = title ?? TruncateUrl(Url);

        StatusText = Url;
    }

    private static string TruncateUrl(string url)
    {
        if (url.Length <= 40) return url;
        return url[..37] + "...";
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _engine.Dispose();
    }
}
