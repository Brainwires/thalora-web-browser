using System.Runtime.InteropServices;

namespace BrowserController;

class Program
{
    private static GuiProcessManager? _guiManager;
    private static HttpProxyServer? _httpServer;
    private static readonly TaskCompletionSource<bool> _shutdownTcs = new();

    static async Task<int> Main(string[] args)
    {
        // Parse CLI arguments
        int port = 9290;
        string? guiPath = null;
        bool autoLaunch = false;
        string? initialUrl = null;

        for (int i = 0; i < args.Length; i++)
        {
            if (args[i] == "--port" && i + 1 < args.Length)
            {
                if (int.TryParse(args[++i], out var p)) port = p;
            }
            else if (args[i].StartsWith("--port="))
            {
                if (int.TryParse(args[i]["--port=".Length..], out var p)) port = p;
            }
            else if (args[i] == "--gui-path" && i + 1 < args.Length)
            {
                guiPath = args[++i];
            }
            else if (args[i].StartsWith("--gui-path="))
            {
                guiPath = args[i]["--gui-path=".Length..];
            }
            else if (args[i] == "--auto-launch")
            {
                autoLaunch = true;
            }
            else if (args[i] == "--url" && i + 1 < args.Length)
            {
                initialUrl = args[++i];
            }
            else if (args[i].StartsWith("--url="))
            {
                initialUrl = args[i]["--url=".Length..];
            }
        }

        Console.Error.WriteLine($"[controller] BrowserController starting (port: {port})");
        if (guiPath != null)
            Console.Error.WriteLine($"[controller] GUI path: {guiPath}");
        if (autoLaunch)
            Console.Error.WriteLine("[controller] Auto-launch enabled");

        // Register signal handlers for graceful shutdown
        Console.CancelKeyPress += (_, e) =>
        {
            e.Cancel = true;
            Console.Error.WriteLine("[controller] Ctrl+C received, shutting down...");
            _shutdownTcs.TrySetResult(true);
        };

        using var sigterm = PosixSignalRegistration.Create(PosixSignal.SIGTERM, _ =>
        {
            Console.Error.WriteLine("[controller] SIGTERM received, shutting down...");
            _shutdownTcs.TrySetResult(true);
        });

        AppDomain.CurrentDomain.ProcessExit += (_, _) =>
        {
            _shutdownTcs.TrySetResult(true);
        };

        _guiManager = new GuiProcessManager();
        _guiManager.Configure(port, guiPath, autoLaunch);

        _httpServer = new HttpProxyServer(port, _guiManager, GracefulShutdownAsync);

        try
        {
            _httpServer.Start();

            if (autoLaunch && guiPath != null)
            {
                Console.Error.WriteLine("[controller] Auto-launching GUI...");
                _guiManager.LaunchGui(initialUrl);
            }
            else
            {
                Console.Error.WriteLine("[controller] Waiting for GUI to register...");
            }

            // Block until shutdown is requested
            await _shutdownTcs.Task;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[controller] Fatal error: {ex.Message}");
            return 1;
        }
        finally
        {
            Console.Error.WriteLine("[controller] Shutting down...");

            if (_guiManager != null)
                await _guiManager.ShutdownAsync();

            if (_httpServer != null)
                await _httpServer.DisposeAsync();

            if (_guiManager != null)
                await _guiManager.DisposeAsync();

            Console.Error.WriteLine("[controller] Shutdown complete.");
        }

        return 0;
    }

    private static async Task GracefulShutdownAsync()
    {
        await Task.CompletedTask;
        _shutdownTcs.TrySetResult(true);
    }
}
