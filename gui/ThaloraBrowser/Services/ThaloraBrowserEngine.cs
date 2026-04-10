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
    // Serializes all native engine calls — the Rust engine is not thread-safe.
    // ComputeStyledTreeAsync (thread pool) and PollHistoryEvents (200ms timer) can
    // overlap; this lock ensures only one native call runs at a time.
    private readonly SemaphoreSlim _engineLock = new(1, 1);

    // Cached navigation capability — updated inside the lock after each mutating FFI call.
    // CanGoBack / CanGoForward must NOT call FFI directly because they're read from the UI
    // thread and can race with navigate operations on background threads.
    private volatile bool _canGoBackCached;
    private volatile bool _canGoForwardCached;

    public ThaloraBrowserEngine()
    {
        _instance = ThaloraNative.thalora_init();
        if (_instance == IntPtr.Zero)
            throw new InvalidOperationException("Failed to initialize Thalora browser engine");
    }

    /// <summary>
    /// Navigate to a URL and return the page HTML.
    /// Uses a dedicated thread with 8MB stack to handle deeply nested HTML parsing
    /// and Boa JS execution without stack overflow.
    /// </summary>
    public Task<string?> NavigateAsync(string url)
    {
        var tcs = new TaskCompletionSource<string?>();
        var thread = new Thread(() =>
        {
            _engineLock.Wait();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_navigate(_instance, url);
                RefreshNavCache();
                tcs.SetResult(ConsumeRustString(ptr));
            }
            catch (Exception ex)
            {
                tcs.SetException(ex);
            }
            finally
            {
                _engineLock.Release();
            }
        }, 8 * 1024 * 1024); // 8MB stack for Boa JS parser + scraper HTML parsing
        thread.IsBackground = true;
        thread.Start();
        return tcs.Task;
    }

    /// <summary>
    /// Navigate to a URL without executing JavaScript (Phase 1 of two-phase navigation).
    /// Returns the static HTML immediately after fetch + CSS load, before any JS runs.
    /// Follow up with ExecutePageScriptsAsync() for JS-driven content.
    /// Uses a dedicated thread with 8MB stack (same reason as NavigateAsync).
    /// </summary>
    public Task<string?> NavigateStaticAsync(string url)
    {
        var tcs = new TaskCompletionSource<string?>();
        var thread = new Thread(() =>
        {
            _engineLock.Wait();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_navigate_static(_instance, url);
                RefreshNavCache();
                tcs.SetResult(ConsumeRustString(ptr));
            }
            catch (Exception ex)
            {
                tcs.SetException(ex);
            }
            finally
            {
                _engineLock.Release();
            }
        }, 8 * 1024 * 1024);
        thread.IsBackground = true;
        thread.Start();
        return tcs.Task;
    }

    /// <summary>
    /// Execute page scripts on the already-loaded page (Phase 2 of two-phase navigation).
    /// Runs Boa JS engine on the static HTML, updates the DOM, and returns whether the DOM changed.
    /// Uses a dedicated thread with 8MB stack — Boa JS execution needs deep stack space,
    /// same as NavigateAsync.
    /// </summary>
    public Task<bool> ExecutePageScriptsAsync()
    {
        var tcs = new TaskCompletionSource<bool>();
        var thread = new Thread(() =>
        {
            _engineLock.Wait();
            try
            {
                ThrowIfDisposed();
                var result = ThaloraNative.thalora_execute_page_scripts(_instance);
                tcs.SetResult(result == 1);
            }
            catch (Exception ex)
            {
                tcs.SetException(ex);
            }
            finally
            {
                _engineLock.Release();
            }
        }, 8 * 1024 * 1024); // 8MB stack for Boa JS execution
        thread.IsBackground = true;
        thread.Start();
        return tcs.Task;
    }

    /// <summary>
    /// Get the current URL.
    /// </summary>
    public Task<string?> GetCurrentUrlAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_get_current_url(_instance);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Get the current page HTML content.
    /// </summary>
    public Task<string?> GetPageHtmlAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_get_page_html(_instance);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Go back in navigation history.
    /// </summary>
    public Task<bool> GoBackAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ok = ThaloraNative.thalora_go_back(_instance) == 0;
                RefreshNavCache();
                return ok;
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Go forward in navigation history.
    /// </summary>
    public Task<bool> GoForwardAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ok = ThaloraNative.thalora_go_forward(_instance) == 0;
                RefreshNavCache();
                return ok;
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Reload the current page and return the new HTML.
    /// </summary>
    public Task<string?> ReloadAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_reload(_instance);
                RefreshNavCache();
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Whether the browser can navigate back. Reads a cached value — safe to call from any thread.
    /// Cache is refreshed inside the engine lock after each navigate/back/forward/reload.
    /// </summary>
    public bool CanGoBack => _canGoBackCached;

    /// <summary>
    /// Whether the browser can navigate forward. Reads a cached value — safe to call from any thread.
    /// </summary>
    public bool CanGoForward => _canGoForwardCached;

    /// <summary>
    /// Refresh CanGoBack/CanGoForward cache. Must be called while holding _engineLock.
    /// </summary>
    private void RefreshNavCache()
    {
        _canGoBackCached = ThaloraNative.thalora_can_go_back(_instance) == 1;
        _canGoForwardCached = ThaloraNative.thalora_can_go_forward(_instance) == 1;
    }

    /// <summary>
    /// Execute JavaScript code and return the result.
    /// </summary>
    public Task<string?> ExecuteJavaScriptAsync(string code)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_execute_js(_instance, code);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Click an element identified by CSS selector.
    /// </summary>
    public Task<bool> ClickElementAsync(string selector)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                return ThaloraNative.thalora_click_element(_instance, selector) == 0;
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Type text into an element identified by CSS selector.
    /// </summary>
    public Task<bool> TypeTextAsync(string selector, string text, bool clearFirst = true)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                return ThaloraNative.thalora_type_text(_instance, selector, text, clearFirst ? 1 : 0) == 0;
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Submit a form with optional JSON field data.
    /// </summary>
    public Task<bool> SubmitFormAsync(string formSelector, string? jsonData = null)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                return ThaloraNative.thalora_submit_form(_instance, formSelector, jsonData) == 0;
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Get the current page title.
    /// </summary>
    public Task<string?> GetPageTitleAsync()
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_get_page_title(_instance);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Compute the page layout on the Rust side and return it as JSON (old pipeline).
    /// </summary>
    public Task<string?> ComputeLayoutAsync(float viewportW, float viewportH)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_compute_layout(_instance, viewportW, viewportH);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Compute the styled element tree on the Rust side and return it as JSON (new pipeline).
    /// Returns CSS-resolved elements without positions — C# handles layout via Avalonia controls.
    /// </summary>
    public Task<string?> ComputeStyledTreeAsync(float viewportW, float viewportH)
        => Task.Run(async () =>
        {
            await _engineLock.WaitAsync();
            try
            {
                ThrowIfDisposed();
                var ptr = ThaloraNative.thalora_compute_styled_tree(_instance, viewportW, viewportH);
                return ConsumeRustString(ptr);
            }
            finally { _engineLock.Release(); }
        });

    /// <summary>
    /// Poll for History API events (pushState, replaceState, popstate).
    /// Returns a JSON array string or null if no events are pending.
    /// </summary>
    public string? PollHistoryEvents()
    {
        if (_disposed) return null;
        // Non-blocking: skip this poll cycle if the engine is busy with navigation or rendering.
        if (!_engineLock.Wait(0)) return null;
        try
        {
            if (_disposed) return null;
            var ptr = ThaloraNative.thalora_poll_history_events(_instance);
            return ConsumeRustString(ptr);
        }
        finally { _engineLock.Release(); }
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

        _engineLock.Dispose();
    }
}
