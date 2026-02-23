using System.Collections.Concurrent;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// Registry of interactive elements and their programmatic actions.
/// Built during ControlTreeBuilder.BuildFromJson() and exposed via WebContentControl
/// for BrowserControlServer endpoints to invoke hover/click actions by element ID.
/// </summary>
public class ElementActionRegistry
{
    /// <summary>
    /// Actions and metadata for a single interactive element.
    /// </summary>
    public class ElementActions
    {
        public string ElementId { get; init; } = "";
        public string Tag { get; init; } = "";
        public string? TextContent { get; init; }
        public string? Href { get; init; }
        public bool HasHoverStyles { get; init; }
        public bool IsLink { get; init; }
        public Action? OnHover { get; init; }
        public Action? OnUnhover { get; init; }
        public Action? OnClick { get; init; }
    }

    private readonly ConcurrentDictionary<string, ElementActions> _elements = new();

    /// <summary>
    /// Register an interactive element with its actions.
    /// </summary>
    public void Register(ElementActions actions)
    {
        _elements[actions.ElementId] = actions;
    }

    /// <summary>
    /// Look up an element by its ID.
    /// </summary>
    public bool TryGet(string elementId, out ElementActions? actions)
    {
        return _elements.TryGetValue(elementId, out actions);
    }

    /// <summary>
    /// Get all interactive elements (links + elements with hover styles).
    /// </summary>
    public IEnumerable<ElementActions> GetInteractiveElements()
    {
        return _elements.Values.Where(e => e.IsLink || e.HasHoverStyles);
    }

    /// <summary>
    /// Find elements matching optional criteria. All provided criteria must match (AND logic).
    /// </summary>
    public IEnumerable<ElementActions> Find(string? tag, string? textContains, string? hrefContains)
    {
        var results = _elements.Values.AsEnumerable();

        if (!string.IsNullOrEmpty(tag))
            results = results.Where(e => string.Equals(e.Tag, tag, StringComparison.OrdinalIgnoreCase));

        if (!string.IsNullOrEmpty(textContains))
            results = results.Where(e =>
                e.TextContent != null &&
                e.TextContent.Contains(textContains, StringComparison.OrdinalIgnoreCase));

        if (!string.IsNullOrEmpty(hrefContains))
            results = results.Where(e =>
                e.Href != null &&
                e.Href.Contains(hrefContains, StringComparison.OrdinalIgnoreCase));

        return results;
    }

    /// <summary>
    /// Total number of registered elements.
    /// </summary>
    public int Count => _elements.Count;
}
