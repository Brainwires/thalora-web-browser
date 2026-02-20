namespace ThaloraBrowser.Models;

/// <summary>
/// Data model for a browser tab.
/// </summary>
public record BrowserTab(
    Guid Id,
    string Title,
    string Url
);
