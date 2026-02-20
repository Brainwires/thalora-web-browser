namespace ThaloraBrowser.Models;

/// <summary>
/// Immutable snapshot of current navigation state.
/// </summary>
public record NavigationState(
    string? Url,
    string? Title,
    bool CanGoBack,
    bool CanGoForward,
    bool IsLoading
);
