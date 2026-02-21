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
    /// Optional HTTP control server port (e.g. --control-port 9222).
    /// When set, a BrowserControlServer is started for external automation.
    /// </summary>
    public static int? ControlPort { get; set; }

    /// <summary>Initial window width in pixels (e.g. --width 600).</summary>
    public static int? InitialWidth { get; set; }

    /// <summary>Initial window height in pixels (e.g. --height 400).</summary>
    public static int? InitialHeight { get; set; }

    private BrowserControlServer? _controlServer;

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
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

            // Start control server after window is shown and rendered
            if (ControlPort.HasValue)
            {
                mainWindow.Opened += (_, _) =>
                {
                    var webContent = mainWindow.FindControl<WebContentControl>("WebContent");
                    if (webContent != null)
                    {
                        _controlServer = new BrowserControlServer(ControlPort.Value);
                        _controlServer.SetUiReferences(webContent, vm);
                        _controlServer.Start();
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
}
