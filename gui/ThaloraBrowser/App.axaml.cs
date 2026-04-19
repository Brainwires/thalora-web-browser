using System.Net.Http;
using System.Text;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using ThaloraBrowser.Controls;
using ThaloraBrowser.Services;
using ThaloraBrowser.ViewModels;
using ThaloraBrowser.Views;

namespace ThaloraBrowser;

public partial class App : Application
{
    /// <summary>
    /// Optional startup URL passed via CLI (e.g. --url https://example.com or bare argument).
    /// Set by Program.Main before Avalonia starts.
    /// </summary>
    public static string? InitialUrl { get; set; }

    /// <summary>
    /// The external BrowserController port (e.g. --control-port 9290).
    /// When set, the GUI registers with the controller after starting its internal server.
    /// </summary>
    public static int? ControllerPort { get; set; }

    /// <summary>
    /// The ephemeral port for the internal BrowserControlServer.
    /// Auto-assigned by Program.Main when --control-port is specified.
    /// </summary>
    public static int? InternalServerPort { get; set; }

    /// <summary>Initial window width in pixels (e.g. --width 600).</summary>
    public static int? InitialWidth { get; set; }

    /// <summary>Initial window height in pixels (e.g. --height 400).</summary>
    public static int? InitialHeight { get; set; }

    private BrowserControlServer? _controlServer;

    /// <summary>
    /// Shut down the control server and unregister from the controller.
    /// Thread-safe — safe to call from signal handler threads.
    /// Does NOT touch UI-bound objects like the view model.
    /// </summary>
    public void ShutdownControlServer()
    {
        _controlServer?.Dispose();

        // Best-effort unregister from controller
        if (ControllerPort.HasValue)
        {
            try
            {
                using var client = new HttpClient { Timeout = TimeSpan.FromSeconds(2) };
                client.PostAsync(
                    $"http://localhost:{ControllerPort.Value}/unregister",
                    new StringContent("")).Wait(TimeSpan.FromSeconds(2));
            }
            catch { /* ignore — controller may already be gone */ }
        }
    }

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        // Tell the CSS engine which color scheme the OS/app is using so pages
        // that respond to `@media (prefers-color-scheme: dark)` render correctly.
        // Update again whenever the user or OS changes the theme at runtime.
        SyncPrefersDarkFromTheme();
        this.ActualThemeVariantChanged += (_, _) => SyncPrefersDarkFromTheme();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            var vm = new MainWindowViewModel(InitialUrl);
            var mainWindow = new MainWindow
            {
                DataContext = vm,
            };

            if (InitialWidth.HasValue)
                mainWindow.Width = InitialWidth.Value;
            if (InitialHeight.HasValue)
                mainWindow.Height = InitialHeight.Value;

            desktop.MainWindow = mainWindow;

            desktop.ShutdownRequested += (_, _) =>
            {
                _controlServer?.Dispose();
                vm.Dispose();
            };

            // Start internal control server after window is shown and rendered
            if (InternalServerPort.HasValue)
            {
                mainWindow.Opened += (_, _) =>
                {
                    var webContent = mainWindow.FindControl<WebContentControl>("WebContent");
                    if (webContent != null)
                    {
                        _controlServer = new BrowserControlServer(InternalServerPort.Value);
                        _controlServer.SetUiReferences(webContent, vm);
                        try
                        {
                            _controlServer.Start();

                            // Register with the BrowserController if one is running
                            if (ControllerPort.HasValue)
                            {
                                RegisterWithController(ControllerPort.Value, InternalServerPort.Value);
                            }
                        }
                        catch (System.Net.HttpListenerException ex)
                        {
                            Console.Error.WriteLine($"[App] WARNING: Control server failed to start (port {InternalServerPort.Value} may be in use): {ex.Message}");
                            _controlServer.Dispose();
                            _controlServer = null;
                        }
                    }
                    else
                    {
                        Console.Error.WriteLine("[App] WARNING: Could not find WebContent control for control server");
                    }
                };
            }
        }

        base.OnFrameworkInitializationCompleted();
    }

    /// <summary>
    /// Push the current Avalonia ActualThemeVariant into the Rust CSS engine
    /// so `prefers-color-scheme` media queries match the OS theme.
    /// </summary>
    private void SyncPrefersDarkFromTheme()
    {
        bool dark = ActualThemeVariant == Avalonia.Styling.ThemeVariant.Dark;
        ThaloraNative.thalora_set_prefers_dark(dark ? 1 : 0);
    }

    /// <summary>
    /// Register this GUI instance with the BrowserController.
    /// Sends the internal server port and PID so the controller can proxy requests.
    /// </summary>
    private static void RegisterWithController(int controllerPort, int internalPort)
    {
        _ = Task.Run(async () =>
        {
            using var client = new HttpClient { Timeout = TimeSpan.FromSeconds(5) };
            var payload = JsonSerializer.Serialize(new
            {
                port = internalPort,
                pid = Environment.ProcessId,
            });

            // Retry registration a few times (controller may still be starting)
            for (int attempt = 1; attempt <= 5; attempt++)
            {
                try
                {
                    var response = await client.PostAsync(
                        $"http://localhost:{controllerPort}/register",
                        new StringContent(payload, Encoding.UTF8, "application/json"));

                    if (response.IsSuccessStatusCode)
                    {
                        Console.Error.WriteLine($"[App] Registered with controller on port {controllerPort}");
                        return;
                    }

                    Console.Error.WriteLine($"[App] Registration attempt {attempt} failed: HTTP {(int)response.StatusCode}");
                }
                catch (Exception ex)
                {
                    Console.Error.WriteLine($"[App] Registration attempt {attempt} failed: {ex.Message}");
                }

                await Task.Delay(1000 * attempt);
            }

            Console.Error.WriteLine("[App] WARNING: Could not register with controller after 5 attempts");
        });
    }
}
