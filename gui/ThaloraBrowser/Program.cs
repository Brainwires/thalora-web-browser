using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using Avalonia;

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

        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
    }

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace();

    /// Resolve the "thalora" native library from the Rust build output directory.
    private static IntPtr ResolveThaloraNative(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != "thalora")
            return IntPtr.Zero;

        // Try multiple candidate paths
        string[] candidates;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "thalora.dll"),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "debug", "thalora.dll")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "debug", "thalora.dll")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", "thalora.dll")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", "thalora.dll")),
            };
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "libthalora.dylib"),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "debug", "libthalora.dylib")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "debug", "libthalora.dylib")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", "libthalora.dylib")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", "libthalora.dylib")),
            };
        }
        else
        {
            candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "libthalora.so"),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "debug", "libthalora.so")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "debug", "libthalora.so")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", "libthalora.so")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", "libthalora.so")),
            };
        }

        foreach (var candidate in candidates)
        {
            if (NativeLibrary.TryLoad(candidate, out var handle))
                return handle;
        }

        // Fall back to default search
        return IntPtr.Zero;
    }
}
