using System.Diagnostics;
using System.Net;
using System.Net.Http;

namespace BrowserController;

/// <summary>
/// Tracks a registered GUI instance and monitors its health.
/// The GUI registers itself (provides its ephemeral port and PID).
/// The controller can also spawn and manage the GUI lifecycle directly.
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

    // GUI registration state
    private int _guiPort;
    private int _guiPid;
    private Task? _monitorTask;
    private Task? _watchTask;
    private CancellationTokenSource? _watchCts;
    private int _consecutiveFailures;
    private int _registrationCount;
    private DateTime _startTime;
    private DateTime _lastRegistrationTime;
    private volatile GuiState _state = GuiState.WaitingForGui;
    private bool _disposed;

    // Lifecycle management (for spawning / auto-relaunch)
    private string? _guiPath;
    private int _controllerPort;
    private bool _autoLaunch;
    private string? _lastLogPath;

    // Allow up to 30 seconds of unresponsiveness before marking the GUI as unhealthy.
    // Large pages (GitHub, Wikipedia) can take 10–20s to build their control trees;
    // we want health checks to survive that window.
    private const int MaxConsecutiveFailures = 15;
    private const int HealthCheckIntervalMs = 2000;

    public GuiProcessManager()
    {
        _healthClient = new HttpClient { Timeout = TimeSpan.FromSeconds(5) };
        _startTime = DateTime.UtcNow;
    }

    /// <summary>
    /// Configure the controller's own port and GUI path so this manager can spawn GUIs.
    /// Call before <see cref="RegisterGui"/> or <see cref="LaunchGui"/>.
    /// </summary>
    public void Configure(int controllerPort, string? guiPath = null, bool autoLaunch = false)
    {
        _controllerPort = controllerPort;
        _guiPath = guiPath;
        _autoLaunch = autoLaunch;
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
        // Cancel any previous PID watcher
        CancelPidWatcher();

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

        // Start PID watcher for instant crash detection
        if (pid > 0)
            StartPidWatcher(pid);

        // Start health monitor if not already running (secondary safety net)
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
        CancelPidWatcher();
        lock (_lock)
        {
            _guiPort = 0;
            _guiPid = -1;
        }
        _state = GuiState.WaitingForGui;
        Console.Error.WriteLine("[controller] GUI unregistered, waiting for new connection");
    }

    // --- Process lifecycle (spawning, killing, watching) ---

    /// <summary>
    /// Spawn a new ThaloraBrowser GUI process. Requires <see cref="Configure"/> to have been
    /// called with a gui path. Does nothing if no gui path is set.
    /// </summary>
    public void LaunchGui(string? initialUrl = null)
    {
        if (string.IsNullOrEmpty(_guiPath))
        {
            Console.Error.WriteLine("[controller] Cannot launch GUI: no --gui-path configured");
            return;
        }
        if (_controllerPort == 0)
        {
            Console.Error.WriteLine("[controller] Cannot launch GUI: controller port not configured");
            return;
        }

        var args = $"run --project \"{_guiPath}\" -- --control-port {_controllerPort}";
        if (!string.IsNullOrEmpty(initialUrl))
            args += $" --url \"{initialUrl}\"";

        // Route GUI stderr to a rolling log file so crashes can be diagnosed.
        // Each launch creates a new file: gui-{timestamp}.log in /tmp (or cwd).
        var logPath = Path.Combine(Path.GetTempPath(),
            $"thalora-gui-{DateTime.UtcNow:yyyyMMdd-HHmmss}.log");
        _lastLogPath = logPath;

        var psi = new ProcessStartInfo("dotnet")
        {
            Arguments = args,
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
        };

        Console.Error.WriteLine($"[controller] Launching GUI: dotnet {args}");
        Console.Error.WriteLine($"[controller] GUI log: {logPath}");
        try
        {
            var proc = Process.Start(psi)!;

            // Pipe GUI stdout + stderr to the log file in background
            _ = Task.Run(async () =>
            {
                try
                {
                    await using var logFile = File.CreateText(logPath);
                    logFile.AutoFlush = true;

                    await Task.WhenAll(
                        CopyStreamAsync(proc.StandardOutput, logFile, "[out]"),
                        CopyStreamAsync(proc.StandardError, logFile, "[err]")
                    );
                }
                catch { /* log piping is best-effort */ }
            });
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[controller] Failed to launch GUI: {ex.Message}");
        }
    }

    /// <summary>
    /// Kill the currently registered GUI process (if any).
    /// </summary>
    public void KillGui()
    {
        CancelPidWatcher();

        int pid;
        lock (_lock) { pid = _guiPid; }

        if (pid > 0)
        {
            try
            {
                var proc = Process.GetProcessById(pid);
                proc.Kill(entireProcessTree: true);
                Console.Error.WriteLine($"[controller] Killed GUI process {pid}");
            }
            catch (ArgumentException)
            {
                // Already gone
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"[controller] Failed to kill GUI {pid}: {ex.Message}");
            }
        }

        lock (_lock)
        {
            _guiPort = 0;
            _guiPid = -1;
            _consecutiveFailures = 0;
        }
        _state = GuiState.WaitingForGui;
    }

    // --- Internal: PID watcher ---

    private void StartPidWatcher(int pid)
    {
        var cts = new CancellationTokenSource();
        _watchCts = cts;

        _watchTask = Task.Run(async () =>
        {
            try
            {
                var proc = Process.GetProcessById(pid);
                proc.EnableRaisingEvents = true;
                await proc.WaitForExitAsync(cts.Token);
            }
            catch (OperationCanceledException)
            {
                return; // watcher cancelled (new registration or shutdown)
            }
            catch (ArgumentException)
            {
                // Process already gone by the time we opened it — treat as immediate exit
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"[controller] PID watcher error for {pid}: {ex.Message}");
                return;
            }

            if (_state == GuiState.ShuttingDown) return;

            // Process has exited — update state immediately (no 30s health-check delay)
            lock (_lock)
            {
                _guiPort = 0;
                _guiPid = -1;
                _consecutiveFailures = 0;
            }
            _state = GuiState.WaitingForGui;
            Console.Error.WriteLine($"[controller] GUI process {pid} exited — waiting for new registration");

            // Auto-relaunch if configured
            if (_autoLaunch && !string.IsNullOrEmpty(_guiPath) && _state != GuiState.ShuttingDown)
            {
                Console.Error.WriteLine("[controller] Auto-relaunching GUI...");
                await Task.Delay(500); // brief pause before relaunch
                LaunchGui();
            }
        });
    }

    private static async Task CopyStreamAsync(StreamReader reader, StreamWriter writer, string prefix)
    {
        string? line;
        while ((line = await reader.ReadLineAsync()) != null)
        {
            await writer.WriteLineAsync($"{DateTime.UtcNow:HH:mm:ss.fff} {prefix} {line}");
        }
    }

    private void CancelPidWatcher()
    {
        var cts = _watchCts;
        _watchCts = null;
        try { cts?.Cancel(); cts?.Dispose(); }
        catch { }
    }

    // --- Health monitor (secondary safety net) ---

    /// <summary>
    /// Shut down health monitoring.
    /// </summary>
    public async Task ShutdownAsync()
    {
        _state = GuiState.ShuttingDown;
        CancelPidWatcher();
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
                gui_path = _guiPath,
                auto_launch = _autoLaunch,
                last_log = _lastLogPath,
            };
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        CancelPidWatcher();
        await _monitorCts.CancelAsync();
        if (_monitorTask != null)
        {
            try { await _monitorTask; } catch { }
        }

        _healthClient.Dispose();
        _monitorCts.Dispose();
    }
}
