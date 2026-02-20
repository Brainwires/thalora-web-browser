using Avalonia;
using Avalonia.Headless;
using Avalonia.Themes.Fluent;

namespace ThaloraBrowser.Tests.Helpers;

/// <summary>
/// Thread-safe Avalonia headless bootstrap for tests that need FormattedText measurement.
/// </summary>
public static class AvaloniaTestApp
{
    private static readonly object Lock = new();
    private static bool _initialized;

    public static void EnsureInitialized()
    {
        if (_initialized) return;

        lock (Lock)
        {
            if (_initialized) return;

            AppBuilder.Configure<TestApp>()
                .UseHeadless(new AvaloniaHeadlessPlatformOptions())
                .SetupWithoutStarting();

            _initialized = true;
        }
    }

    private class TestApp : Application
    {
        public override void Initialize()
        {
            Styles.Add(new FluentTheme());
        }
    }
}
