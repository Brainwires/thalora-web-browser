using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Net.Sockets;
using System.Reflection;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Media.Fonts;

namespace ThaloraBrowser;

class Program
{
    [STAThread]
    public static void Main(string[] args)
    {
        // Register custom native library resolver so the runtime can find libthalora
        NativeLibrary.SetDllImportResolver(typeof(Services.ThaloraNative).Assembly, ResolveThaloraNative);

        // Parse CLI arguments
        string? initialUrl = null;
        int? controlPort = null;
        int? windowWidth = null;
        int? windowHeight = null;

        for (int i = 0; i < args.Length; i++)
        {
            // --url <url> or --url=<url>
            if (args[i] == "--url" && i + 1 < args.Length)
            {
                initialUrl = args[i + 1];
                i++;
            }
            else if (args[i].StartsWith("--url="))
            {
                initialUrl = args[i]["--url=".Length..];
            }
            // --control-port <port> or --control-port=<port>
            else if (args[i] == "--control-port" && i + 1 < args.Length)
            {
                if (int.TryParse(args[i + 1], out var p))
                    controlPort = p;
                i++;
            }
            else if (args[i].StartsWith("--control-port="))
            {
                if (int.TryParse(args[i]["--control-port=".Length..], out var p))
                    controlPort = p;
            }
            // --width <pixels>
            else if (args[i] == "--width" && i + 1 < args.Length)
            {
                if (int.TryParse(args[i + 1], out var w))
                    windowWidth = w;
                i++;
            }
            else if (args[i].StartsWith("--width="))
            {
                if (int.TryParse(args[i]["--width=".Length..], out var w))
                    windowWidth = w;
            }
            // --height <pixels>
            else if (args[i] == "--height" && i + 1 < args.Length)
            {
                if (int.TryParse(args[i + 1], out var h))
                    windowHeight = h;
                i++;
            }
            else if (args[i].StartsWith("--height="))
            {
                if (int.TryParse(args[i]["--height=".Length..], out var h))
                    windowHeight = h;
            }
        }

        // If no --url flag, treat the first non-flag argument as a URL
        initialUrl ??= args.FirstOrDefault(a => !a.StartsWith("-") && !int.TryParse(a, out _));

        App.InitialUrl = initialUrl;
        App.InitialWidth = windowWidth;
        App.InitialHeight = windowHeight;

        // When --control-port is specified, it's the CONTROLLER's external port.
        // The internal BrowserControlServer runs on an auto-assigned ephemeral port.
        // The GUI ensures the controller is running, then registers with it.
        if (controlPort.HasValue)
        {
            App.ControllerPort = controlPort.Value;
            App.InternalServerPort = FindFreePort();
            EnsureControllerRunning(controlPort.Value);
            Console.Error.WriteLine($"[gui] Controller port: {controlPort.Value}, internal server port: {App.InternalServerPort}");
        }

        // Ensure the control server socket is cleaned up on any process exit path.
        // ShutdownControlServer() is thread-safe and idempotent — safe to call from
        // signal handler threads. We do NOT touch the view model here (it's UI-bound).
        //
        // Coverage:
        // 1. ShutdownRequested (window close) — handled in App.axaml.cs
        // 2. SIGTERM (kill) — PosixSignalRegistration callback before default termination
        // 3. ProcessExit — universal fallback for any exit path we missed
        // Note: SIGINT (Ctrl+C) is intercepted by Avalonia's Cocoa backend on macOS
        // at the native level — .NET handlers never fire. Use SIGTERM instead.
        using var sigterm = PosixSignalRegistration.Create(PosixSignal.SIGTERM, _ =>
        {
            (Application.Current as App)?.ShutdownControlServer();
        });
        AppDomain.CurrentDomain.ProcessExit += (_, _) =>
        {
            (Application.Current as App)?.ShutdownControlServer();
        };

        // Global exception handler — prevents Avalonia layout/rendering exceptions
        // (e.g., FontSize=0, invalid GlyphTypeface) from crashing the entire process.
        // The GUI stays alive; the page may render partially but won't take down the app.
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            Console.Error.WriteLine($"[gui] UNHANDLED EXCEPTION (terminating={e.IsTerminating}): {e.ExceptionObject}");
        };
        TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            Console.Error.WriteLine($"[gui] UNOBSERVED TASK EXCEPTION: {e.Exception}");
            e.SetObserved();
        };

        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
    }

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .ConfigureFonts(fontManager =>
            {
                // Register bundled fonts (Noto Sans, Noto Serif, Fira Mono)
                // embedded as AvaloniaResource in the Fonts/ directory.
                fontManager.AddFontCollection(new EmbeddedFontCollection(
                    new Uri("fonts:ThaloraBrowser", UriKind.Absolute),
                    new Uri("avares://ThaloraBrowser/Fonts", UriKind.Absolute)));
            })
            .LogToTrace();

    /// <summary>
    /// Check if the BrowserController is already running on the given port.
    /// If not, spawn it as a detached background process.
    /// </summary>
    private static void EnsureControllerRunning(int controllerPort)
    {
        // Check if controller is already responding
        using var client = new HttpClient { Timeout = TimeSpan.FromSeconds(2) };
        try
        {
            var response = client.GetAsync($"http://localhost:{controllerPort}/health").Result;
            if (response.IsSuccessStatusCode)
            {
                Console.Error.WriteLine($"[gui] Controller already running on port {controllerPort}");
                return;
            }
        }
        catch
        {
            // Not running, need to start it
        }

        Console.Error.WriteLine($"[gui] Starting BrowserController on port {controllerPort}...");

        var controllerProjectPath = FindControllerProjectPath();

        var psi = new ProcessStartInfo
        {
            FileName = "dotnet",
            Arguments = $"run --project \"{controllerProjectPath}\" -- --port {controllerPort}",
            UseShellExecute = false,
            RedirectStandardOutput = false,
            RedirectStandardError = false,
            CreateNoWindow = true,
        };

        var process = Process.Start(psi);
        if (process == null)
        {
            Console.Error.WriteLine("[gui] WARNING: Failed to start BrowserController process");
            return;
        }

        Console.Error.WriteLine($"[gui] BrowserController started (PID: {process.Id})");

        // Wait for the controller to become responsive (up to 15s for dotnet run + build)
        var deadline = DateTime.UtcNow.AddSeconds(15);
        while (DateTime.UtcNow < deadline)
        {
            System.Threading.Thread.Sleep(500);
            try
            {
                var response = client.GetAsync($"http://localhost:{controllerPort}/health").Result;
                if (response.IsSuccessStatusCode)
                {
                    Console.Error.WriteLine("[gui] BrowserController is ready");
                    return;
                }
            }
            catch
            {
                // Still starting up
            }
        }

        Console.Error.WriteLine("[gui] WARNING: BrowserController may not be ready yet, continuing anyway");
    }

    /// <summary>
    /// Find the BrowserController project directory, searching relative to the repo root.
    /// </summary>
    private static string FindControllerProjectPath()
    {
        // Walk up from the current directory looking for the gui/BrowserController project
        var dir = AppContext.BaseDirectory;
        for (var i = 0; i < 10; i++)
        {
            var candidate = Path.Combine(dir, "gui", "BrowserController", "BrowserController.csproj");
            if (File.Exists(candidate))
                return Path.Combine(dir, "gui", "BrowserController");

            var parent = Directory.GetParent(dir);
            if (parent == null) break;
            dir = parent.FullName;
        }

        // Also check relative to the working directory
        var cwd = Directory.GetCurrentDirectory();
        var cwdCandidate = Path.Combine(cwd, "gui", "BrowserController", "BrowserController.csproj");
        if (File.Exists(cwdCandidate))
            return Path.Combine(cwd, "gui", "BrowserController");

        throw new InvalidOperationException(
            "Could not find gui/BrowserController/BrowserController.csproj. " +
            "Run from the thalora-web-browser repository root.");
    }

    /// <summary>
    /// Find a free TCP port by binding to port 0.
    /// </summary>
    private static int FindFreePort()
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;
        listener.Stop();
        return port;
    }

    /// Resolve the "thalora" native library from the Rust build output directory.
    private static IntPtr ResolveThaloraNative(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != "thalora")
            return IntPtr.Zero;

        // Check THALORA_LIB_PATH environment variable first
        var envPath = Environment.GetEnvironmentVariable("THALORA_LIB_PATH");
        if (!string.IsNullOrEmpty(envPath))
        {
            var fullEnvPath = Path.GetFullPath(envPath);
            if (NativeLibrary.TryLoad(fullEnvPath, out var envHandle))
                return envHandle;
        }

        // Platform-specific library name
        string libName;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            libName = "thalora.dll";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            libName = "libthalora.dylib";
        else
            libName = "libthalora.so";

        // Try multiple candidate paths — prefer release over debug
        var candidates = new[]
        {
            // Release paths first
            Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", libName)),
            Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", libName)),
            // Then check bin directory (may be stale — checked after release)
            Path.Combine(AppContext.BaseDirectory, libName),
            // Debug paths last
            Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "debug", libName)),
            Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "debug", libName)),
        };

        foreach (var candidate in candidates)
        {
            if (NativeLibrary.TryLoad(candidate, out var handle))
                return handle;
        }

        // Fall back to default search
        return IntPtr.Zero;
    }
}
