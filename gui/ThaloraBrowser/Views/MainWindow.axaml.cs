using Avalonia.Controls;
using Avalonia.Input;
using ThaloraBrowser.Controls;
using ThaloraBrowser.ViewModels;

namespace ThaloraBrowser.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        // Wire up keyboard shortcuts
        KeyDown += OnWindowKeyDown;

        // Wire up address bar Enter key
        var addressBar = this.FindControl<TextBox>("AddressBar");
        if (addressBar != null)
        {
            addressBar.KeyDown += OnAddressBarKeyDown;
        }

        // Wire up WebContentControl events
        var webContent = this.FindControl<WebContentControl>("WebContent");
        if (webContent != null)
        {
            webContent.LinkClicked += OnLinkClicked;
            webContent.HoveredLinkChanged += OnHoveredLinkChanged;
        }

        // Wire up tab click handling
        AddHandler(Avalonia.Controls.Button.ClickEvent, OnButtonClick);
    }

    private void OnAddressBarKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.Enter && DataContext is MainWindowViewModel vm)
        {
            vm.NavigateCommand.Execute(null);
            e.Handled = true;

            // Move focus to content area
            var webContent = this.FindControl<WebContentControl>("WebContent");
            webContent?.Focus();
        }
    }

    private void OnWindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (DataContext is not MainWindowViewModel vm) return;

        // Ctrl+T: New tab
        if (e.KeyModifiers.HasFlag(KeyModifiers.Control) && e.Key == Key.T)
        {
            vm.NewTabCommand.Execute(null);
            e.Handled = true;
        }
        // Ctrl+W: Close current tab
        else if (e.KeyModifiers.HasFlag(KeyModifiers.Control) && e.Key == Key.W)
        {
            vm.CloseTabCommand.Execute(vm.ActiveTab);
            e.Handled = true;
        }
        // Ctrl+L or F6: Focus address bar
        else if ((e.KeyModifiers.HasFlag(KeyModifiers.Control) && e.Key == Key.L) || e.Key == Key.F6)
        {
            var addressBar = this.FindControl<TextBox>("AddressBar");
            addressBar?.Focus();
            addressBar?.SelectAll();
            e.Handled = true;
        }
        // F5: Reload
        else if (e.Key == Key.F5)
        {
            vm.ReloadCommand.Execute(null);
            e.Handled = true;
        }
        // Alt+Left: Back
        else if (e.KeyModifiers.HasFlag(KeyModifiers.Alt) && e.Key == Key.Left)
        {
            vm.GoBackCommand.Execute(null);
            e.Handled = true;
        }
        // Alt+Right: Forward
        else if (e.KeyModifiers.HasFlag(KeyModifiers.Alt) && e.Key == Key.Right)
        {
            vm.GoForwardCommand.Execute(null);
            e.Handled = true;
        }
    }

    private async void OnLinkClicked(object? sender, LinkClickedEventArgs e)
    {
        if (DataContext is MainWindowViewModel vm)
        {
            await vm.NavigateToUrlAsync(e.Url);
        }
    }

    private void OnHoveredLinkChanged(object? sender, string? url)
    {
        if (DataContext is MainWindowViewModel vm)
        {
            vm.StatusBarText = url ?? vm.ActiveTab?.StatusText ?? "Ready";
        }
    }

    private void OnButtonClick(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    {
        if (DataContext is not MainWindowViewModel vm) return;

        if (e.Source is Avalonia.Controls.Button button)
        {
            // Handle tab close button clicks
            if (button.Classes.Contains("tab-close") && button.Tag is BrowserTabViewModel tabToClose)
            {
                vm.CloseTabCommand.Execute(tabToClose);
                e.Handled = true;
                return;
            }

            // Handle tab selection clicks
            if ((button.Classes.Contains("tab-button") || button.Classes.Contains("tab-button-active"))
                && button.Tag is BrowserTabViewModel tabToSelect)
            {
                vm.ActiveTab = tabToSelect;
                e.Handled = true;
            }
        }
    }
}
