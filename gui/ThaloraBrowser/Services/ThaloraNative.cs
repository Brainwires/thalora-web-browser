using System.Runtime.InteropServices;

namespace ThaloraBrowser.Services;

/// <summary>
/// Raw P/Invoke declarations for the Thalora Rust browser engine.
/// All string parameters are marshalled as UTF-8.
/// All returned IntPtr strings must be freed with thalora_free_string().
/// </summary>
internal static partial class ThaloraNative
{
    const string LibName = "thalora";

    // --- Lifecycle ---

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_init();

    [LibraryImport(LibName)]
    internal static partial void thalora_destroy(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_last_error(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial void thalora_free_string(IntPtr ptr);

    // --- Navigation ---

    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr thalora_navigate(IntPtr inst, string url);

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_get_current_url(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_get_page_html(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial int thalora_go_back(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial int thalora_go_forward(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_reload(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial int thalora_can_go_back(IntPtr inst);

    [LibraryImport(LibName)]
    internal static partial int thalora_can_go_forward(IntPtr inst);

    // --- Interaction ---

    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr thalora_execute_js(IntPtr inst, string code);

    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial int thalora_click_element(IntPtr inst, string selector);

    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial int thalora_type_text(IntPtr inst, string selector, string text, int clearFirst);

    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial int thalora_submit_form(IntPtr inst, string formSelector, string? jsonData);

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_get_page_title(IntPtr inst);

    // --- Layout ---

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_compute_layout(IntPtr inst, float viewportW, float viewportH);
}
