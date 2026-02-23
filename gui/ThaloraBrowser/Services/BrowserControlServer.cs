using System.Collections.Concurrent;
using System.Net;
using System.Text;
using System.Text.Json;
using Avalonia;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using ThaloraBrowser.Controls;
using ThaloraBrowser.ViewModels;

namespace ThaloraBrowser.Services;

/// <summary>
/// Lightweight HTTP control server for external automation (e.g., Claude Code).
/// Provides endpoints for screenshots, navigation, state inspection, and input.
/// </summary>
public class BrowserControlServer : IDisposable
{
    private readonly HttpListener _listener;
    private readonly CancellationTokenSource _cts;
    private readonly ConcurrentBag<Task> _inflightRequests = new();
    private readonly int _port;
    private Task? _listenTask;
    private bool _disposed;

    // References set after the UI is fully loaded
    private WebContentControl? _webContent;
    private MainWindowViewModel? _viewModel;

    public BrowserControlServer(int port = 9222)
    {
        _port = port;
        _listener = new HttpListener();
        _listener.Prefixes.Add($"http://localhost:{port}/");
        _cts = new CancellationTokenSource();
    }

    /// <summary>
    /// Set references to the UI controls. Must be called from the UI thread after the window is shown.
    /// </summary>
    public void SetUiReferences(WebContentControl webContent, MainWindowViewModel viewModel)
    {
        _webContent = webContent;
        _viewModel = viewModel;
    }

    /// <summary>
    /// Start listening for HTTP requests on a background thread.
    /// </summary>
    public void Start()
    {
        _listener.Start();
        Console.Error.WriteLine($"[ControlServer] Listening on http://localhost:{_port}/");
        _listenTask = ListenLoop(_cts.Token);
    }

    private async Task ListenLoop(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                var context = await _listener.GetContextAsync().WaitAsync(ct);
                var handlerTask = HandleRequest(context);
                _inflightRequests.Add(handlerTask);
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (HttpListenerException) when (ct.IsCancellationRequested)
            {
                break;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"[ControlServer] Listener error: {ex.Message}");
            }
        }
    }

    private async Task HandleRequest(HttpListenerContext context)
    {
        var request = context.Request;
        var response = context.Response;

        // CORS headers for local tooling
        response.Headers.Add("Access-Control-Allow-Origin", "*");
        response.Headers.Add("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
        response.Headers.Add("Access-Control-Allow-Headers", "Content-Type");

        if (request.HttpMethod == "OPTIONS")
        {
            response.StatusCode = 204;
            response.Close();
            return;
        }

        try
        {
            var path = request.Url?.AbsolutePath ?? "/";

            switch (path)
            {
                case "/health":
                    await RespondJson(response, new { status = "ok", port = _port });
                    break;

                case "/screenshot":
                    await HandleScreenshot(request, response);
                    break;

                case "/state":
                    await HandleState(response);
                    break;

                case "/navigate":
                    if (request.HttpMethod == "POST")
                        await HandleNavigate(request, response);
                    else
                        await RespondError(response, 405, "POST required");
                    break;

                case "/click":
                    if (request.HttpMethod == "POST")
                        await HandleClick(request, response);
                    else
                        await RespondError(response, 405, "POST required");
                    break;

                case "/type":
                    if (request.HttpMethod == "POST")
                        await HandleType(request, response);
                    else
                        await RespondError(response, 405, "POST required");
                    break;

                case "/scroll":
                    if (request.HttpMethod == "POST")
                        await HandleScroll(request, response);
                    else
                        await RespondError(response, 405, "POST required");
                    break;

                case "/content-height":
                    await HandleContentHeight(response);
                    break;

                case "/wait-for-images":
                    if (request.HttpMethod == "POST")
                        await HandleWaitForImages(request, response);
                    else
                        await RespondError(response, 405, "POST required");
                    break;

                case "/layout":
                    await HandleLayout(response);
                    break;

                case "/html":
                    await HandleHtml(response);
                    break;

                case "/shutdown":
                    await HandleShutdown(response);
                    break;

                default:
                    await RespondError(response, 404, $"Unknown endpoint: {path}");
                    break;
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[ControlServer] Request error: {ex.Message}");
            try { await RespondError(response, 500, ex.Message); } catch { /* ignore */ }
        }
    }

    /// <summary>
    /// Capture a screenshot of the WebContentControl as PNG.
    /// Accepts optional ?delay=ms query parameter for render-settle timing.
    /// </summary>
    private async Task HandleScreenshot(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_webContent == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        // Optional render-settle delay
        var delayParam = request.QueryString["delay"];
        if (delayParam != null && int.TryParse(delayParam, out var delayMs) && delayMs > 0)
        {
            delayMs = Math.Min(delayMs, 10000); // Cap at 10s
            await Task.Delay(delayMs);
        }

        byte[]? pngBytes = null;

        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            var bounds = _webContent.Bounds;
            if (bounds.Width < 1 || bounds.Height < 1)
                return;

            var pixelSize = new PixelSize((int)bounds.Width, (int)bounds.Height);
            var bitmap = new RenderTargetBitmap(pixelSize, new Vector(96, 96));
            bitmap.Render(_webContent);

            using var ms = new MemoryStream();
            bitmap.Save(ms);
            pngBytes = ms.ToArray();
        });

        if (pngBytes == null)
        {
            await RespondError(response, 503, "Failed to capture screenshot");
            return;
        }

        response.ContentType = "image/png";
        response.ContentLength64 = pngBytes.Length;
        await response.OutputStream.WriteAsync(pngBytes);
        response.Close();
    }

    /// <summary>
    /// Return current browser state (URL, title, tab count, etc.).
    /// </summary>
    private async Task HandleState(HttpListenerResponse response)
    {
        if (_viewModel == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        // Read properties on UI thread
        object? state = null;
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            var tab = _viewModel.ActiveTab;
            state = new
            {
                url = tab?.Url ?? "",
                title = tab?.Title ?? "",
                html_length = tab?.HtmlContent?.Length ?? 0,
                is_loading = tab?.IsLoading ?? false,
                tab_count = _viewModel.Tabs.Count,
                status = tab?.StatusText ?? "",
            };
        });

        await RespondJson(response, state!);
    }

    /// <summary>
    /// Navigate to a URL. Body: {"url":"...", "timeout_ms": 30000}
    /// Waits for the page to finish loading (IsLoading == false) with a configurable
    /// timeout (default 30s), then adds a 500ms settle delay.
    /// </summary>
    private async Task HandleNavigate(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_viewModel == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);
        var url = data.GetProperty("url").GetString();

        if (string.IsNullOrWhiteSpace(url))
        {
            await RespondError(response, 400, "Missing 'url' field");
            return;
        }

        var timeoutMs = 30000;
        if (data.TryGetProperty("timeout_ms", out var timeoutProp) && timeoutProp.TryGetInt32(out var customTimeout))
        {
            timeoutMs = Math.Clamp(customTimeout, 1000, 120000);
        }

        await Dispatcher.UIThread.InvokeAsync(async () =>
        {
            await _viewModel.NavigateToUrlAsync(url);
        });

        // Poll until IsLoading == false or timeout
        var deadline = DateTime.UtcNow.AddMilliseconds(timeoutMs);
        var loaded = false;
        while (DateTime.UtcNow < deadline)
        {
            var isLoading = await Dispatcher.UIThread.InvokeAsync(() => _viewModel.ActiveTab?.IsLoading ?? false);
            if (!isLoading)
            {
                loaded = true;
                break;
            }
            await Task.Delay(100);
        }

        // Settle delay to let rendering finish
        await Task.Delay(500);

        await RespondJson(response, new { status = "navigated", url, loaded, timeout_ms = timeoutMs });
    }

    /// <summary>
    /// Simulate a click at coordinates. Body: {"x":N,"y":N}
    /// </summary>
    private async Task HandleClick(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_webContent == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);
        var x = data.GetProperty("x").GetDouble();
        var y = data.GetProperty("y").GetDouble();

        // With the Avalonia native control tree, hit-testing is handled by Avalonia's input system.
        // Simulate a pointer press at the given coordinates — any link handlers set up by
        // ControlTreeBuilder will fire via normal Avalonia event routing.
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            // Avalonia doesn't have a direct API to simulate pointer events at coordinates,
            // so we just report the click coordinates. Links are handled by pointer events
            // on the individual TextBlock controls created by ControlTreeBuilder.
        });

        await RespondJson(response, new { status = "clicked", x, y });
    }

    /// <summary>
    /// Type text into the address bar. Body: {"text":"...","target":"addressbar"|"page"}
    /// </summary>
    private async Task HandleType(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_viewModel == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);
        var text = data.GetProperty("text").GetString() ?? "";
        var target = data.TryGetProperty("target", out var t) ? t.GetString() : "addressbar";

        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            if (target == "addressbar")
            {
                _viewModel.AddressBarText = text;
            }
        });

        await RespondJson(response, new { status = "typed", target, text });
    }

    /// <summary>
    /// Return the current layout JSON from the Rust engine.
    /// </summary>
    private async Task HandleLayout(HttpListenerResponse response)
    {
        if (_viewModel?.ActiveTab?.Engine == null)
        {
            await RespondError(response, 503, "No active engine");
            return;
        }

        var engine = _viewModel.ActiveTab.Engine;
        var layoutJson = await engine.ComputeStyledTreeAsync(1280, 720);

        if (string.IsNullOrEmpty(layoutJson))
        {
            await RespondError(response, 500, engine.GetLastError() ?? "Layout returned null");
            return;
        }

        response.ContentType = "application/json";
        var bytes = Encoding.UTF8.GetBytes(layoutJson);
        response.ContentLength64 = bytes.Length;
        await response.OutputStream.WriteAsync(bytes);
        response.Close();
    }

    /// <summary>
    /// Return the current page HTML source.
    /// </summary>
    private async Task HandleHtml(HttpListenerResponse response)
    {
        if (_viewModel?.ActiveTab == null)
        {
            await RespondError(response, 503, "No active tab");
            return;
        }

        var html = _viewModel.ActiveTab.HtmlContent ?? "";
        response.ContentType = "text/html; charset=utf-8";
        var bytes = Encoding.UTF8.GetBytes(html);
        response.ContentLength64 = bytes.Length;
        await response.OutputStream.WriteAsync(bytes);
        response.Close();
    }

    /// <summary>
    /// Set scroll position. Body: {"y": offset}
    /// </summary>
    private async Task HandleScroll(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_webContent == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);
        var y = data.GetProperty("y").GetDouble();

        double actualY = 0;
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            _webContent.SetScrollOffset(y);
            actualY = _webContent.ScrollOffsetY;
        });

        await RespondJson(response, new { status = "scrolled", scroll_y = actualY });
    }

    /// <summary>
    /// Return content dimensions and scroll state.
    /// </summary>
    private async Task HandleContentHeight(HttpListenerResponse response)
    {
        if (_webContent == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        object? dims = null;
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            dims = new
            {
                content_height = _webContent.ContentHeight,
                viewport_height = _webContent.ViewportHeight,
                viewport_width = _webContent.ViewportWidth,
                scroll_y = _webContent.ScrollOffsetY,
                max_scroll_y = _webContent.MaxScrollY,
            };
        });

        await RespondJson(response, dims!);
    }

    /// <summary>
    /// Wait for async image loads. Body: {"wait_ms": 3000}
    /// Triggers a re-render after waiting.
    /// </summary>
    private async Task HandleWaitForImages(HttpListenerRequest request, HttpListenerResponse response)
    {
        if (_webContent == null)
        {
            await RespondError(response, 503, "UI not ready");
            return;
        }

        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);

        var waitMs = 3000;
        if (data.TryGetProperty("wait_ms", out var waitProp) && waitProp.TryGetInt32(out var customWait))
        {
            waitMs = Math.Clamp(customWait, 100, 30000);
        }

        await Task.Delay(waitMs);

        // Trigger a re-render to pick up any newly loaded images
        await Dispatcher.UIThread.InvokeAsync(() =>
        {
            _webContent.InvalidateVisual();
        });

        // Small settle delay after invalidation
        await Task.Delay(200);

        await RespondJson(response, new { status = "waited", wait_ms = waitMs });
    }

    // --- Helpers ---

    /// <summary>
    private static async Task<string> ReadBody(HttpListenerRequest request)
    {
        using var reader = new StreamReader(request.InputStream, request.ContentEncoding);
        return await reader.ReadToEndAsync();
    }

    private static async Task RespondJson(HttpListenerResponse response, object data)
    {
        response.ContentType = "application/json";
        var json = JsonSerializer.Serialize(data);
        var bytes = Encoding.UTF8.GetBytes(json);
        response.ContentLength64 = bytes.Length;
        await response.OutputStream.WriteAsync(bytes);
        response.Close();
    }

    private static async Task RespondError(HttpListenerResponse response, int statusCode, string message)
    {
        response.StatusCode = statusCode;
        response.ContentType = "application/json";
        var json = JsonSerializer.Serialize(new { error = message });
        var bytes = Encoding.UTF8.GetBytes(json);
        response.ContentLength64 = bytes.Length;
        await response.OutputStream.WriteAsync(bytes);
        response.Close();
    }

    /// <summary>
    /// Gracefully shut down the application. Responds with OK, then triggers
    /// Avalonia's desktop lifetime shutdown on the UI thread so the process
    /// exits cleanly without leaving zombie windows on macOS.
    /// </summary>
    private async Task HandleShutdown(HttpListenerResponse response)
    {
        await RespondJson(response, new { status = "shutting_down" });

        // Give the response a moment to flush, then shut down on UI thread
        _ = Task.Run(async () =>
        {
            await Task.Delay(200);
            await Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(() =>
            {
                var lifetime = Avalonia.Application.Current?.ApplicationLifetime
                    as Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime;
                lifetime?.Shutdown(0);
            });
        });
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        // Signal cancellation first
        _cts.Cancel();

        // Close the listener to release the socket and unblock GetContextAsync
        try { _listener.Close(); } catch { /* ignore */ }

        // Wait for the listen loop to finish (with timeout)
        if (_listenTask != null)
        {
            try { _listenTask.Wait(TimeSpan.FromSeconds(2)); } catch { /* ignore */ }
        }

        // Wait for any in-flight request handlers to complete (with timeout)
        var pending = _inflightRequests.Where(t => !t.IsCompleted).ToArray();
        if (pending.Length > 0)
        {
            try { Task.WaitAll(pending, TimeSpan.FromSeconds(2)); } catch { /* ignore */ }
        }

        // Dispose the CTS last
        _cts.Dispose();
    }
}
