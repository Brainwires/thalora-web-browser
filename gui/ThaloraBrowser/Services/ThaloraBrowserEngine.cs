using System.Runtime.InteropServices;

namespace ThaloraBrowser.Services;

/// <summary>
/// Safe managed wrapper around the Thalora native browser engine.
/// All methods run P/Invoke calls on the thread pool to avoid blocking the UI thread.
/// </summary>
public sealed class ThaloraBrowserEngine : IThaloraBrowserEngine
{
    private IntPtr _instance;
    private bool _disposed;

    public ThaloraBrowserEngine()
    {
        _instance = ThaloraNative.thalora_init();
        if (_instance == IntPtr.Zero)
            throw new InvalidOperationException("Failed to initialize Thalora browser engine");
    }

    /// <summary>
    /// Navigate to a URL and return the page HTML.
    /// </summary>
    public Task<string?> NavigateAsync(string url)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_navigate(_instance, url);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Get the current URL.
    /// </summary>
    public Task<string?> GetCurrentUrlAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_get_current_url(_instance);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Get the current page HTML content.
    /// </summary>
    public Task<string?> GetPageHtmlAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_get_page_html(_instance);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Go back in navigation history.
    /// </summary>
    public Task<bool> GoBackAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            return ThaloraNative.thalora_go_back(_instance) == 0;
        });

    /// <summary>
    /// Go forward in navigation history.
    /// </summary>
    public Task<bool> GoForwardAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            return ThaloraNative.thalora_go_forward(_instance) == 0;
        });

    /// <summary>
    /// Reload the current page and return the new HTML.
    /// </summary>
    public Task<string?> ReloadAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_reload(_instance);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Check if the browser can go back in history.
    /// </summary>
    public bool CanGoBack
    {
        get
        {
            if (_disposed) return false;
            return ThaloraNative.thalora_can_go_back(_instance) == 1;
        }
    }

    /// <summary>
    /// Check if the browser can go forward in history.
    /// </summary>
    public bool CanGoForward
    {
        get
        {
            if (_disposed) return false;
            return ThaloraNative.thalora_can_go_forward(_instance) == 1;
        }
    }

    /// <summary>
    /// Execute JavaScript code and return the result.
    /// </summary>
    public Task<string?> ExecuteJavaScriptAsync(string code)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_execute_js(_instance, code);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Click an element identified by CSS selector.
    /// </summary>
    public Task<bool> ClickElementAsync(string selector)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            return ThaloraNative.thalora_click_element(_instance, selector) == 0;
        });

    /// <summary>
    /// Type text into an element identified by CSS selector.
    /// </summary>
    public Task<bool> TypeTextAsync(string selector, string text, bool clearFirst = true)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            return ThaloraNative.thalora_type_text(_instance, selector, text, clearFirst ? 1 : 0) == 0;
        });

    /// <summary>
    /// Submit a form with optional JSON field data.
    /// </summary>
    public Task<bool> SubmitFormAsync(string formSelector, string? jsonData = null)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            return ThaloraNative.thalora_submit_form(_instance, formSelector, jsonData) == 0;
        });

    /// <summary>
    /// Get the current page title.
    /// </summary>
    public Task<string?> GetPageTitleAsync()
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_get_page_title(_instance);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Compute the page layout on the Rust side and return it as JSON (old pipeline).
    /// </summary>
    public Task<string?> ComputeLayoutAsync(float viewportW, float viewportH)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_compute_layout(_instance, viewportW, viewportH);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Compute the styled element tree on the Rust side and return it as JSON (new pipeline).
    /// Returns CSS-resolved elements without positions — C# handles layout via Avalonia controls.
    /// </summary>
    public Task<string?> ComputeStyledTreeAsync(float viewportW, float viewportH)
        => Task.Run(() =>
        {
            ThrowIfDisposed();
            var ptr = ThaloraNative.thalora_compute_styled_tree(_instance, viewportW, viewportH);
            return ConsumeRustString(ptr);
        });

    /// <summary>
    /// Poll for History API events (pushState, replaceState, popstate).
    /// Returns a JSON array string or null if no events are pending.
    /// </summary>
    public string? PollHistoryEvents()
    {
        if (_disposed) return null;
        var ptr = ThaloraNative.thalora_poll_history_events(_instance);
        return ConsumeRustString(ptr);
    }

    /// <summary>
    /// Set the navigation mode. 0 = Interactive (no delays), 1 = Stealth (human-like delays).
    /// </summary>
    public bool SetNavigationMode(int mode)
    {
        if (_disposed) return false;
        return ThaloraNative.thalora_set_navigation_mode(_instance, mode) == 0;
    }

    /// <summary>
    /// Get the last error from the native engine.
    /// </summary>
    public string? GetLastError()
    {
        if (_disposed) return null;
        var ptr = ThaloraNative.thalora_last_error(_instance);
        if (ptr == IntPtr.Zero) return null;
        // last_error string is owned by the instance — don't free it
        return Marshal.PtrToStringUTF8(ptr);
    }

    /// <summary>
    /// Marshal a Rust-allocated UTF-8 string and free the native memory.
    /// </summary>
    private static string? ConsumeRustString(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero) return null;
        try
        {
            return Marshal.PtrToStringUTF8(ptr);
        }
        finally
        {
            ThaloraNative.thalora_free_string(ptr);
        }
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        if (_instance != IntPtr.Zero)
        {
            ThaloraNative.thalora_destroy(_instance);
            _instance = IntPtr.Zero;
        }
    }
}
