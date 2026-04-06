using System.Text.Json;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.ViewModels;

/// <summary>
/// ViewModel for a single browser tab.
/// Each tab owns its own ThaloraBrowserEngine instance.
/// </summary>
public partial class BrowserTabViewModel : ViewModelBase, IDisposable
{
    private readonly IThaloraBrowserEngine _engine;
    private System.Timers.Timer? _historyPollTimer;
    private volatile bool _disposed;

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

    public IThaloraBrowserEngine Engine => _engine;

    public BrowserTabViewModel()
    {
        _engine = new ThaloraBrowserEngine();
    }

    public BrowserTabViewModel(IThaloraBrowserEngine engine)
    {
        _engine = engine;
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

            // Start polling for History API events (idempotent — won't double-start)
            StartHistoryPolling();
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
        catch (Exception ex)
        {
            StatusText = $"Error: {ex.Message}";
            return false;
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
        catch (Exception ex)
        {
            StatusText = $"Error: {ex.Message}";
            return false;
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

    /// <summary>
    /// Dispatch a DOM event to the JavaScript engine.
    /// Looks up the CSS selector for the element and dispatches a synthetic event.
    /// </summary>
    public async Task DispatchDomEventAsync(string eventType, string elementId, Dictionary<string, string>? elementSelectors)
    {
        if (_disposed || elementSelectors == null) return;

        if (!elementSelectors.TryGetValue(elementId, out var selector))
            return;

        // Escape the selector for use in querySelector
        var escapedSelector = selector.Replace("\\", "\\\\").Replace("\"", "\\\"");

        // Build JavaScript to dispatch the event
        var js = $@"(function() {{
            var el = document.querySelector(""{escapedSelector}"");
            if (el) {{
                el.dispatchEvent(new MouseEvent(""{eventType}"", {{bubbles: true, cancelable: true}}));
            }}
        }})();";

        try
        {
            await _engine.ExecuteJavaScriptAsync(js);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[BrowserTabVM] DOM event dispatch failed: {ex.Message}");
        }
    }

    /// <summary>
    /// Start polling for History API events from the JS engine.
    /// </summary>
    private void StartHistoryPolling()
    {
        if (_historyPollTimer != null) return; // already running

        _historyPollTimer = new System.Timers.Timer(200);
        _historyPollTimer.Elapsed += OnHistoryPollTick;
        _historyPollTimer.AutoReset = true;
        _historyPollTimer.Start();
    }

    /// <summary>
    /// Stop the history polling timer.
    /// </summary>
    private void StopHistoryPolling()
    {
        if (_historyPollTimer == null) return;
        _historyPollTimer.Stop();
        _historyPollTimer.Elapsed -= OnHistoryPollTick;
        _historyPollTimer.Dispose();
        _historyPollTimer = null;
    }

    /// <summary>
    /// Timer callback: drain history events from the Rust engine and update the address bar.
    /// Runs on a ThreadPool thread — must dispatch UI property changes to the UI thread.
    /// </summary>
    private void OnHistoryPollTick(object? sender, System.Timers.ElapsedEventArgs e)
    {
        // Skip if disposed, navigating, or engine is busy
        if (_disposed || IsLoading) return;

        try
        {
            var json = _engine.PollHistoryEvents();
            if (string.IsNullOrEmpty(json)) return;

            var events = JsonSerializer.Deserialize<JsonElement[]>(json);
            if (events == null) return;

            foreach (var evt in events)
            {
                if (_disposed || IsLoading) return;

                var eventType = evt.GetProperty("type").GetString();
                var url = evt.GetProperty("url").GetString() ?? "";

                switch (eventType)
                {
                    case "pushState":
                    case "replaceState":
                    {
                        var resolved = ResolveHistoryUrl(url);
                        // Must update ObservableProperty on the UI thread
                        Dispatcher.UIThread.Post(() =>
                        {
                            if (!_disposed && !IsLoading)
                                Url = resolved;
                        });
                        System.Diagnostics.Debug.WriteLine($"[HistoryAPI] {eventType}: {resolved}");
                        break;
                    }

                    case "popstate":
                    {
                        var resolved = ResolveHistoryUrl(url);
                        var stateJson = evt.TryGetProperty("state_json", out var stateEl)
                            ? stateEl.GetString() : "null";
                        // Must update ObservableProperty on the UI thread
                        Dispatcher.UIThread.Post(() =>
                        {
                            if (_disposed || IsLoading) return;
                            Url = resolved;
                            // Fire popstate in JS on the UI thread so it's properly sequenced
                            _ = FirePopstateEvent(stateJson);
                        });
                        System.Diagnostics.Debug.WriteLine($"[HistoryAPI] popstate: {resolved}");
                        break;
                    }
                }
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[HistoryAPI] Poll error: {ex.Message}");
        }
    }

    /// <summary>
    /// Resolve a History API URL (may be relative path like "/page2") against the current page.
    /// </summary>
    private string ResolveHistoryUrl(string url)
    {
        if (string.IsNullOrEmpty(url)) return Url;

        // If it's already an absolute URL, use it as-is
        if (url.StartsWith("http://") || url.StartsWith("https://"))
            return url;

        // Resolve relative URL against the current page
        if (Uri.TryCreate(Url, UriKind.Absolute, out var baseUri) &&
            Uri.TryCreate(baseUri, url, out var resolved))
        {
            return resolved.ToString();
        }

        return url;
    }

    /// <summary>
    /// Dispatch a popstate event in the JS engine with the given state.
    /// </summary>
    private async Task FirePopstateEvent(string? stateJson)
    {
        if (_disposed) return;

        var state = stateJson ?? "null";
        var js = $@"(function() {{
            var event = new PopStateEvent('popstate', {{ state: {state} }});
            window.dispatchEvent(event);
        }})();";

        try
        {
            await _engine.ExecuteJavaScriptAsync(js);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[HistoryAPI] popstate dispatch failed: {ex.Message}");
        }
    }

    internal static string TruncateUrl(string url)
    {
        if (url.Length <= 40) return url;
        return url[..37] + "...";
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        StopHistoryPolling();
        // Delay engine destruction slightly so any in-flight FFI calls can return
        // before thalora_destroy reclaims the Rust instance (avoids use-after-free).
        var engine = _engine;
        _ = Task.Run(async () =>
        {
            await Task.Delay(150);
            engine.Dispose();
        });
    }
}
