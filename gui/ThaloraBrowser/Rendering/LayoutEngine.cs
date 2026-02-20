namespace ThaloraBrowser.Rendering;

/// <summary>
/// Layout computation is now done on the Rust side via taffy + lightningcss.
/// This file is kept as a placeholder for C#-side layout helpers if needed.
///
/// The Rust engine computes full CSS layout and returns it as JSON via
/// thalora_compute_layout FFI. HtmlRenderer deserializes the JSON into LayoutBox trees.
/// </summary>
internal static class LayoutEngineHelpers
{
    internal static BoxType MapDisplayToBoxType(DisplayMode display) => display switch
    {
        DisplayMode.Block => BoxType.Block,
        DisplayMode.Inline => BoxType.Inline,
        DisplayMode.InlineBlock => BoxType.InlineBlock,
        DisplayMode.Flex => BoxType.Block,
        DisplayMode.ListItem => BoxType.ListItem,
        DisplayMode.Table => BoxType.TableBox,
        DisplayMode.TableRow => BoxType.TableRowBox,
        DisplayMode.TableCell => BoxType.TableCellBox,
        _ => BoxType.Block,
    };
}
