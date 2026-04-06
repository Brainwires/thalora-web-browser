using System.Net;
using System.Net.Http;

namespace BrowserController;

/// <summary>
/// Tracks a registered GUI instance and monitors its health.
/// The GUI registers itself (provides its ephemeral port and PID).
/// The controller never launches the GUI — the GUI launches itself and connects.
/// </summary>
public sealed class GuiProcessManager : IAsyncDisposable
{
    public enum GuiState
    {
        /// <summary>No GUI registered yet (or previous GUI disconnected).</summary>
        WaitingForGui,
        /// <summary>GUI registered and responding to health checks.</summary>
        Healthy,
        /// <summary>GUI registered but not responding (may be zombie).</summary>
        Unresponsive,
        /// <summary>Controller is shutting down.</summary>
        ShuttingDown,
    }

    private readonly HttpClient _healthClient;
    private readonly CancellationTokenSource _monitorCts = new();
    private readonly object _lock = new();

    private int _guiPort;
    private int _guiPid;
    private Task? _monitorTask;
    private int _consecutiveFailures;
    private int _registrationCount;
    private DateTime _startTime;
    private DateTime _lastRegistrationTime;
    private volatile GuiState _state = GuiState.WaitingForGui;
    private bool _disposed;

    // Allow up to 30 seconds of unresponsiveness before marking the GUI as unhealthy.
    // Large pages (GitHub, Wikipedia) can take 10–20s to build their control trees even
    // on background threads; we want health checks to survive that window.
    private const int MaxConsecutiveFailures = 15;
    private const int HealthCheckIntervalMs = 2000;

    public GuiProcessManager()
    {
        _healthClient = new HttpClient { Timeout = TimeSpan.FromSeconds(5) };
        _startTime = DateTime.UtcNow;
    }

    /// <summary>Current port the GUI is listening on (0 if no GUI registered).</summary>
    public int GuiPort
    {
        get { lock (_lock) return _guiPort; }
    }

    /// <summary>Current GUI process ID (-1 if no GUI registered).</summary>
    public int GuiPid
    {
        get { lock (_lock) return _guiPid; }
    }

    /// <summary>Current state of the GUI connection.</summary>
    public GuiState State => _state;

    /// <summary>Number of times a GUI has registered.</summary>
    public int RegistrationCount
    {
        get { lock (_lock) return _registrationCount; }
    }

    /// <summary>Whether the GUI is considered healthy and reachable.</summary>
    public bool IsHealthy => _state == GuiState.Healthy;

    /// <summary>Time since the controller started.</summary>
    public TimeSpan Uptime => DateTime.UtcNow - _startTime;

    /// <summary>
    /// Register a GUI instance. Called when the GUI starts up and connects to the controller.
    /// Replaces any previously registered GUI (handles GUI restarts naturally).
    /// </summary>
    public void RegisterGui(int port, int pid)
    {
        lock (_lock)
        {
            _guiPort = port;
            _guiPid = pid;
            _registrationCount++;
            _consecutiveFailures = 0;
            _lastRegistrationTime = DateTime.UtcNow;
        }

        _state = GuiState.Healthy;
        Console.Error.WriteLine($"[controller] GUI registered (PID: {pid}, port: {port}, registration #{_registrationCount})");

        // Start health monitor if not already running
        if (_monitorTask == null || _monitorTask.IsCompleted)
        {
            _monitorTask = MonitorHealthAsync(_monitorCts.Token);
        }
    }

    /// <summary>
    /// Unregister the current GUI (e.g., on clean shutdown).
    /// </summary>
    public void UnregisterGui()
    {
        lock (_lock)
        {
            _guiPort = 0;
            _guiPid = -1;
        }
        _state = GuiState.WaitingForGui;
        Console.Error.WriteLine("[controller] GUI unregistered, waiting for new connection");
    }

    /// <summary>
    /// Shut down health monitoring.
    /// </summary>
    public async Task ShutdownAsync()
    {
        _state = GuiState.ShuttingDown;
        await _monitorCts.CancelAsync();

        if (_monitorTask != null)
        {
            try { await _monitorTask; } catch (OperationCanceledException) { }
        }
    }

    private async Task MonitorHealthAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                await Task.Delay(HealthCheckIntervalMs, ct);
            }
            catch (OperationCanceledException)
            {
                break;
            }

            if (_state == GuiState.ShuttingDown || _state == GuiState.WaitingForGui)
                continue;

            int port;
            lock (_lock) { port = _guiPort; }
            if (port == 0) continue;

            var healthy = await CheckHealthOnceAsync(port);

            if (healthy)
            {
                if (_state != GuiState.Healthy)
                {
                    Console.Error.WriteLine("[controller] GUI is now healthy");
                    _state = GuiState.Healthy;
                }
                _consecutiveFailures = 0;
            }
            else
            {
                _consecutiveFailures++;

                if (_consecutiveFailures >= MaxConsecutiveFailures && _state == GuiState.Healthy)
                {
                    _state = GuiState.Unresponsive;
                    Console.Error.WriteLine($"[controller] GUI unresponsive after {MaxConsecutiveFailures} failed health checks");
                }
            }
        }
    }

    private async Task<bool> CheckHealthOnceAsync(int port)
    {
        try
        {
            var response = await _healthClient.GetAsync($"http://localhost:{port}/health");
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Get status info for the /status endpoint.
    /// </summary>
    public object GetStatusInfo(int controllerPort)
    {
        lock (_lock)
        {
            return new
            {
                controller = "running",
                controller_port = controllerPort,
                gui_state = _state.ToString().ToLowerInvariant(),
                gui_pid = _guiPid,
                gui_port = _guiPort,
                gui_healthy = IsHealthy,
                registration_count = _registrationCount,
                uptime_seconds = (int)Uptime.TotalSeconds,
            };
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        await _monitorCts.CancelAsync();
        if (_monitorTask != null)
        {
            try { await _monitorTask; } catch { }
        }

        _healthClient.Dispose();
        _monitorCts.Dispose();
    }
}
