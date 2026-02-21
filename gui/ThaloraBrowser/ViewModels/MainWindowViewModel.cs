using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.ViewModels;

/// <summary>
/// Main window ViewModel — manages tabs, address bar, and navigation commands.
/// </summary>
public partial class MainWindowViewModel : ViewModelBase, IDisposable
{
    private readonly Func<IThaloraBrowserEngine>? _engineFactory;
    private bool _disposed;

    public ObservableCollection<BrowserTabViewModel> Tabs { get; } = new();

    [ObservableProperty]
    private BrowserTabViewModel? _activeTab;

    [ObservableProperty]
    private string _addressBarText = "";

    [ObservableProperty]
    private string _statusBarText = "Ready";

    [ObservableProperty]
    private bool _canGoBack;

    [ObservableProperty]
    private bool _canGoForward;

    [ObservableProperty]
    private bool _isLoading;

    public MainWindowViewModel() : this(initialUrl: null) { }

    public MainWindowViewModel(string? initialUrl) : this(initialUrl, engineFactory: null) { }

    public MainWindowViewModel(Func<IThaloraBrowserEngine> engineFactory) : this(initialUrl: null, engineFactory) { }

    public MainWindowViewModel(string? initialUrl, Func<IThaloraBrowserEngine>? engineFactory)
    {
        _engineFactory = engineFactory;

        // Start with one new tab
        NewTab();

        // If an initial URL was provided, navigate to it after the tab is ready
        if (!string.IsNullOrWhiteSpace(initialUrl))
        {
            AddressBarText = initialUrl;
            _ = Navigate();
        }
    }

    partial void OnActiveTabChanged(BrowserTabViewModel? value)
    {
        if (value != null)
        {
            AddressBarText = value.Url;
            StatusBarText = value.StatusText;
            UpdateNavigationState();
        }
    }

    private void UpdateNavigationState()
    {
        if (ActiveTab?.Engine != null)
        {
            CanGoBack = ActiveTab.Engine.CanGoBack;
            CanGoForward = ActiveTab.Engine.CanGoForward;
            IsLoading = ActiveTab.IsLoading;
        }
        else
        {
            CanGoBack = false;
            CanGoForward = false;
            IsLoading = false;
        }
    }

    [RelayCommand]
    private void NewTab()
    {
        var tab = _engineFactory != null
            ? new BrowserTabViewModel(_engineFactory())
            : new BrowserTabViewModel();
        tab.PropertyChanged += (_, e) =>
        {
            if (tab == ActiveTab)
            {
                switch (e.PropertyName)
                {
                    case nameof(BrowserTabViewModel.Url):
                        AddressBarText = tab.Url;
                        break;
                    case nameof(BrowserTabViewModel.StatusText):
                        StatusBarText = tab.StatusText;
                        break;
                    case nameof(BrowserTabViewModel.IsLoading):
                        IsLoading = tab.IsLoading;
                        UpdateNavigationState();
                        break;
                }
            }
        };

        Tabs.Add(tab);
        ActiveTab = tab;
    }

    [RelayCommand]
    private void CloseTab(BrowserTabViewModel? tab)
    {
        tab ??= ActiveTab;
        if (tab == null) return;

        var index = Tabs.IndexOf(tab);
        Tabs.Remove(tab);
        tab.Dispose();

        if (Tabs.Count == 0)
        {
            // Always keep at least one tab
            NewTab();
        }
        else if (ActiveTab == null || ActiveTab == tab)
        {
            // Select an adjacent tab
            ActiveTab = Tabs[Math.Min(index, Tabs.Count - 1)];
        }
    }

    [RelayCommand]
    private async Task Navigate()
    {
        if (ActiveTab == null || string.IsNullOrWhiteSpace(AddressBarText))
            return;

        try
        {
            await ActiveTab.NavigateAsync(AddressBarText);
        }
        catch (Exception ex)
        {
            System.Console.Error.WriteLine($"[Navigate] Error: {ex.Message}");
        }
        UpdateNavigationState();
    }

    [RelayCommand]
    private async Task GoBack()
    {
        if (ActiveTab == null) return;
        try
        {
            await ActiveTab.GoBackAsync();
        }
        catch (Exception ex)
        {
            System.Console.Error.WriteLine($"[GoBack] Error: {ex.Message}");
        }
        UpdateNavigationState();
    }

    [RelayCommand]
    private async Task GoForward()
    {
        if (ActiveTab == null) return;
        try
        {
            await ActiveTab.GoForwardAsync();
        }
        catch (Exception ex)
        {
            System.Console.Error.WriteLine($"[GoForward] Error: {ex.Message}");
        }
        UpdateNavigationState();
    }

    [RelayCommand]
    private async Task Reload()
    {
        if (ActiveTab == null) return;
        try
        {
            await ActiveTab.ReloadAsync();
        }
        catch (Exception ex)
        {
            System.Console.Error.WriteLine($"[Reload] Error: {ex.Message}");
        }
        UpdateNavigationState();
    }

    [RelayCommand]
    private void GoHome()
    {
        if (ActiveTab == null) return;
        AddressBarText = "https://www.google.com";
        _ = Navigate();
    }

    /// <summary>
    /// Called when a link is clicked in the rendered content.
    /// </summary>
    public async Task NavigateToUrlAsync(string url)
    {
        if (ActiveTab == null) return;

        AddressBarText = url;
        await ActiveTab.NavigateAsync(url);
        UpdateNavigationState();
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        foreach (var tab in Tabs)
            tab.Dispose();
        Tabs.Clear();
    }
}
