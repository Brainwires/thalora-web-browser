using System.Text.Json.Serialization;

namespace ThaloraBrowser.Rendering;

/// <summary>
/// JSON deserialization models for the Rust styled element tree.
/// These mirror the Rust StyledTreeResult / StyledElement / ResolvedStyles structs
/// from src/engine/renderer/styled_tree.rs.
/// </summary>

/// <summary>
/// Top-level result from Rust's compute_styled_tree().
/// </summary>
internal class StyledTreeResult
{
    [JsonPropertyName("root")]
    public StyledElement? Root { get; set; }

    [JsonPropertyName("viewport_width")]
    public float ViewportWidth { get; set; }

    [JsonPropertyName("viewport_height")]
    public float ViewportHeight { get; set; }

    [JsonPropertyName("element_selectors")]
    public Dictionary<string, string>? ElementSelectors { get; set; }
}

/// <summary>
/// A styled DOM element with resolved CSS properties but no layout positions.
/// </summary>
internal class StyledElement
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = "";

    [JsonPropertyName("tag")]
    public string Tag { get; set; } = "";

    [JsonPropertyName("text_content")]
    public string? TextContent { get; set; }

    [JsonPropertyName("img_src")]
    public string? ImgSrc { get; set; }

    [JsonPropertyName("img_alt")]
    public string? ImgAlt { get; set; }

    [JsonPropertyName("link_href")]
    public string? LinkHref { get; set; }

    [JsonPropertyName("styles")]
    public ResolvedStyles Styles { get; set; } = new();

    [JsonPropertyName("hover_styles")]
    public ResolvedStyles? HoverStyles { get; set; }

    [JsonPropertyName("children")]
    public List<StyledElement> Children { get; set; } = new();
}

/// <summary>
/// Resolved CSS styles as strings. C# parses these into Avalonia-specific types.
/// </summary>
internal class ResolvedStyles
{
    // Display & Layout
    [JsonPropertyName("display")]
    public string? Display { get; set; }

    [JsonPropertyName("position")]
    public string? Position { get; set; }

    [JsonPropertyName("flex_direction")]
    public string? FlexDirection { get; set; }

    [JsonPropertyName("flex_wrap")]
    public string? FlexWrap { get; set; }

    [JsonPropertyName("justify_content")]
    public string? JustifyContent { get; set; }

    [JsonPropertyName("align_items")]
    public string? AlignItems { get; set; }

    [JsonPropertyName("align_self")]
    public string? AlignSelf { get; set; }

    [JsonPropertyName("gap")]
    public string? Gap { get; set; }

    [JsonPropertyName("flex_grow")]
    public string? FlexGrow { get; set; }

    [JsonPropertyName("flex_shrink")]
    public string? FlexShrink { get; set; }

    [JsonPropertyName("flex_basis")]
    public string? FlexBasis { get; set; }

    // Box Model
    [JsonPropertyName("width")]
    public string? Width { get; set; }

    [JsonPropertyName("height")]
    public string? Height { get; set; }

    [JsonPropertyName("min_width")]
    public string? MinWidth { get; set; }

    [JsonPropertyName("min_height")]
    public string? MinHeight { get; set; }

    [JsonPropertyName("max_width")]
    public string? MaxWidth { get; set; }

    [JsonPropertyName("max_height")]
    public string? MaxHeight { get; set; }

    [JsonPropertyName("margin")]
    public StyleBoxSides? Margin { get; set; }

    [JsonPropertyName("padding")]
    public StyleBoxSides? Padding { get; set; }

    // Typography
    [JsonPropertyName("font_size")]
    public string? FontSize { get; set; }

    [JsonPropertyName("font_family")]
    public string? FontFamily { get; set; }

    [JsonPropertyName("font_weight")]
    public string? FontWeight { get; set; }

    [JsonPropertyName("font_style")]
    public string? FontStyle { get; set; }

    [JsonPropertyName("line_height")]
    public string? LineHeight { get; set; }

    [JsonPropertyName("text_align")]
    public string? TextAlign { get; set; }

    [JsonPropertyName("text_decoration")]
    public string? TextDecoration { get; set; }

    [JsonPropertyName("text_transform")]
    public string? TextTransform { get; set; }

    [JsonPropertyName("white_space")]
    public string? WhiteSpace { get; set; }

    [JsonPropertyName("letter_spacing")]
    public string? LetterSpacing { get; set; }

    [JsonPropertyName("word_spacing")]
    public string? WordSpacing { get; set; }

    // Colors & Visual
    [JsonPropertyName("color")]
    public string? Color { get; set; }

    [JsonPropertyName("background_color")]
    public string? BackgroundColor { get; set; }

    // Borders
    [JsonPropertyName("border_width")]
    public StyleBoxSides? BorderWidth { get; set; }

    [JsonPropertyName("border_color")]
    public string? BorderColor { get; set; }

    [JsonPropertyName("border_style")]
    public string? BorderStyle { get; set; }

    [JsonPropertyName("border_radius")]
    public string? BorderRadius { get; set; }

    // Miscellaneous
    [JsonPropertyName("opacity")]
    public float? Opacity { get; set; }

    [JsonPropertyName("overflow")]
    public string? Overflow { get; set; }

    [JsonPropertyName("visibility")]
    public string? Visibility { get; set; }

    [JsonPropertyName("z_index")]
    public int? ZIndex { get; set; }

    [JsonPropertyName("list_style_type")]
    public string? ListStyleType { get; set; }

    [JsonPropertyName("cursor")]
    public string? Cursor { get; set; }

    [JsonPropertyName("grid_template_columns")]
    public string? GridTemplateColumns { get; set; }

    [JsonPropertyName("grid_template_rows")]
    public string? GridTemplateRows { get; set; }

    [JsonPropertyName("grid_template_areas")]
    public string? GridTemplateAreas { get; set; }

    [JsonPropertyName("grid_area")]
    public string? GridArea { get; set; }
}

/// <summary>
/// Box model sides with CSS string values.
/// </summary>
internal class StyleBoxSides
{
    [JsonPropertyName("top")]
    public string Top { get; set; } = "0px";

    [JsonPropertyName("right")]
    public string Right { get; set; } = "0px";

    [JsonPropertyName("bottom")]
    public string Bottom { get; set; } = "0px";

    [JsonPropertyName("left")]
    public string Left { get; set; } = "0px";
}
