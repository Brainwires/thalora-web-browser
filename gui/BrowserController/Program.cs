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
        }

        Console.Error.WriteLine($"[controller] BrowserController starting (port: {port})");
        Console.Error.WriteLine("[controller] Waiting for GUI to register...");

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
        _httpServer = new HttpProxyServer(port, _guiManager, GracefulShutdownAsync);

        try
        {
            _httpServer.Start();

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
