using System;
using System.IO;
using System.Linq;
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
        App.ControlPort = controlPort;
        App.InitialWidth = windowWidth;
        App.InitialHeight = windowHeight;

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
