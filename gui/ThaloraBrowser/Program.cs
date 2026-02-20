using System;
using System.IO;
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
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", "thalora.dll")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", "thalora.dll")),
            };
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "libthalora.dylib"),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "target", "release", "libthalora.dylib")),
                Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "target", "release", "libthalora.dylib")),
            };
        }
        else
        {
            candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "libthalora.so"),
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
