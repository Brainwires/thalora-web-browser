using System.Net;
using System.Text;
using System.Text.Json;

namespace BrowserController;

/// <summary>
/// External-facing HTTP server that handles controller endpoints directly
/// and proxies all other requests to the GUI's internal BrowserControlServer.
/// </summary>
public sealed class HttpProxyServer : IAsyncDisposable
{
    private readonly HttpListener _listener;
    private readonly GuiProcessManager _guiManager;
    private readonly HttpClient _proxyClient;
    private readonly int _port;
    private readonly CancellationTokenSource _cts = new();
    private readonly Func<Task> _shutdownCallback;
    private Task? _listenTask;
    private bool _disposed;

    /// <summary>Endpoints that are forwarded verbatim to the GUI.</summary>
    private static readonly HashSet<string> ProxiedPaths = new(StringComparer.OrdinalIgnoreCase)
    {
        "/screenshot",
        "/state",
        "/navigate",
        "/click",
        "/type",
        "/scroll",
        "/content-height",
        "/wait-for-images",
        "/layout",
        "/html",
        "/hover-element",
        "/unhover-element",
        "/click-element",
        "/elements",
        "/find-element",
    };

    public HttpProxyServer(int port, GuiProcessManager guiManager, Func<Task> shutdownCallback)
    {
        _port = port;
        _guiManager = guiManager;
        _shutdownCallback = shutdownCallback;
        _listener = new HttpListener();
        _listener.Prefixes.Add($"http://localhost:{port}/");

        // Long timeout for operations like /navigate with timeout_ms=120000
        _proxyClient = new HttpClient { Timeout = TimeSpan.FromSeconds(300) };
    }

    /// <summary>
    /// Start listening for HTTP requests.
    /// </summary>
    public void Start()
    {
        _listener.Start();
        Console.Error.WriteLine($"[controller] HTTP server listening on http://localhost:{_port}/");
        _listenTask = ListenLoop(_cts.Token);
    }

    private async Task ListenLoop(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                var context = await _listener.GetContextAsync().WaitAsync(ct);
                _ = HandleRequestSafe(context);
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
                Console.Error.WriteLine($"[controller] Listener error: {ex.Message}");
            }
        }
    }

    private async Task HandleRequestSafe(HttpListenerContext context)
    {
        try
        {
            await HandleRequest(context);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[controller] Request error: {ex.Message}");
            try
            {
                await RespondError(context.Response, 500, ex.Message);
            }
            catch { /* ignore */ }
        }
    }

    private async Task HandleRequest(HttpListenerContext context)
    {
        var request = context.Request;
        var response = context.Response;

        // CORS headers
        response.Headers.Add("Access-Control-Allow-Origin", "*");
        response.Headers.Add("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
        response.Headers.Add("Access-Control-Allow-Headers", "Content-Type");

        if (request.HttpMethod == "OPTIONS")
        {
            response.StatusCode = 204;
            response.Close();
            return;
        }

        var path = request.Url?.AbsolutePath ?? "/";

        // Controller-handled endpoints
        switch (path)
        {
            case "/health":
                await HandleHealth(response);
                return;

            case "/status":
                await HandleStatus(response);
                return;

            case "/register":
                if (request.HttpMethod == "POST")
                    await HandleRegister(request, response);
                else
                    await RespondError(response, 405, "POST required");
                return;

            case "/unregister":
                if (request.HttpMethod == "POST")
                    await HandleUnregister(response);
                else
                    await RespondError(response, 405, "POST required");
                return;

            case "/shutdown":
                await HandleShutdown(response);
                return;

            case "/launch":
                // Accept GET or POST (POST body may contain {"url":"..."})
                await HandleLaunch(request, response);
                return;

            case "/restart":
                // Accept GET or POST (POST body may contain {"url":"..."})
                await HandleRestart(request, response);
                return;
        }

        // Proxied endpoints
        if (ProxiedPaths.Contains(path))
        {
            await ProxyToGui(request, response);
            return;
        }

        await RespondError(response, 404, $"Unknown endpoint: {path}");
    }

    // --- Controller-handled endpoints ---

    private async Task HandleHealth(HttpListenerResponse response)
    {
        var guiStatus = _guiManager.IsHealthy ? "healthy" :
                        _guiManager.State == GuiProcessManager.GuiState.WaitingForGui ? "not_connected" :
                        "unhealthy";
        var controllerStatus = _guiManager.IsHealthy ? "ok" : "degraded";

        await RespondJson(response, new
        {
            status = controllerStatus,
            gui = guiStatus,
        });
    }

    private async Task HandleStatus(HttpListenerResponse response)
    {
        await RespondJson(response, _guiManager.GetStatusInfo(_port));
    }

    private async Task HandleRegister(HttpListenerRequest request, HttpListenerResponse response)
    {
        var body = await ReadBody(request);
        var data = JsonSerializer.Deserialize<JsonElement>(body);

        if (!data.TryGetProperty("port", out var portProp) || !portProp.TryGetInt32(out var guiPort))
        {
            await RespondError(response, 400, "Missing 'port' field");
            return;
        }

        var guiPid = -1;
        if (data.TryGetProperty("pid", out var pidProp) && pidProp.TryGetInt32(out var pid))
        {
            guiPid = pid;
        }

        _guiManager.RegisterGui(guiPort, guiPid);

        await RespondJson(response, new
        {
            status = "registered",
            gui_port = guiPort,
            gui_pid = guiPid,
        });
    }

    private async Task HandleUnregister(HttpListenerResponse response)
    {
        _guiManager.UnregisterGui();
        await RespondJson(response, new { status = "unregistered" });
    }

    private async Task HandleShutdown(HttpListenerResponse response)
    {
        await RespondJson(response, new { status = "shutting_down" });

        // Trigger shutdown asynchronously to let the response flush
        _ = Task.Run(async () =>
        {
            await Task.Delay(200);
            await _shutdownCallback();
        });
    }

    private async Task HandleLaunch(HttpListenerRequest request, HttpListenerResponse response)
    {
        var url = await ParseUrlFromBody(request);

        if (_guiManager.State != GuiProcessManager.GuiState.WaitingForGui)
        {
            await RespondJson(response, new
            {
                status = "already_running",
                gui_pid = _guiManager.GuiPid,
                gui_state = _guiManager.State.ToString().ToLowerInvariant(),
            });
            return;
        }

        _guiManager.LaunchGui(url);
        await RespondJson(response, new { status = "launched", url });
    }

    private async Task HandleRestart(HttpListenerRequest request, HttpListenerResponse response)
    {
        var url = await ParseUrlFromBody(request);

        // Kill the existing GUI if any, then launch a fresh one
        _guiManager.KillGui();
        await Task.Delay(300); // let the OS clean up the port binding
        _guiManager.LaunchGui(url);

        await RespondJson(response, new { status = "restarting", url });
    }

    private static async Task<string?> ParseUrlFromBody(HttpListenerRequest request)
    {
        if (!request.HasEntityBody) return null;
        try
        {
            var body = await ReadBody(request);
            if (string.IsNullOrWhiteSpace(body)) return null;
            var data = JsonSerializer.Deserialize<JsonElement>(body);
            return data.TryGetProperty("url", out var u) ? u.GetString() : null;
        }
        catch { return null; }
    }

    // --- Proxy logic ---

    private async Task ProxyToGui(HttpListenerRequest request, HttpListenerResponse response)
    {
        var guiPort = _guiManager.GuiPort;

        if (guiPort == 0 || _guiManager.State == GuiProcessManager.GuiState.WaitingForGui)
        {
            await RespondError(response, 503, "No GUI connected. Launch ThaloraBrowser with --control-port to connect.");
            return;
        }

        if (_guiManager.State == GuiProcessManager.GuiState.Unresponsive)
        {
            await RespondError(response, 503, "GUI is unresponsive. It may need to be restarted.");
            return;
        }

        try
        {
            // Build the target URL preserving path and query string
            var targetUri = $"http://localhost:{guiPort}{request.Url!.PathAndQuery}";

            using var proxyRequest = new HttpRequestMessage(
                new HttpMethod(request.HttpMethod), targetUri);

            // Copy request body for POST methods
            if (request.HttpMethod == "POST" && request.HasEntityBody)
            {
                var bodyStream = new MemoryStream();
                await request.InputStream.CopyToAsync(bodyStream);
                bodyStream.Position = 0;
                proxyRequest.Content = new StreamContent(bodyStream);

                if (request.ContentType != null)
                    proxyRequest.Content.Headers.ContentType =
                        new System.Net.Http.Headers.MediaTypeHeaderValue(request.ContentType);
            }

            using var proxyResponse = await _proxyClient.SendAsync(proxyRequest);

            // Copy status code
            response.StatusCode = (int)proxyResponse.StatusCode;

            // Copy content-type
            if (proxyResponse.Content.Headers.ContentType != null)
                response.ContentType = proxyResponse.Content.Headers.ContentType.ToString();

            // Copy body via stream (handles binary data like screenshots naturally)
            var responseBytes = await proxyResponse.Content.ReadAsByteArrayAsync();
            response.ContentLength64 = responseBytes.Length;
            await response.OutputStream.WriteAsync(responseBytes);
            response.Close();
        }
        catch (HttpRequestException)
        {
            await RespondError(response, 503, "GUI not reachable");
        }
        catch (TaskCanceledException)
        {
            await RespondError(response, 504, "GUI request timed out");
        }
    }

    // --- Helpers ---

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

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        await _cts.CancelAsync();

        try { _listener.Close(); } catch { }

        if (_listenTask != null)
        {
            try { await _listenTask.WaitAsync(TimeSpan.FromSeconds(2)); } catch { }
        }

        _proxyClient.Dispose();
        _cts.Dispose();
    }
}
