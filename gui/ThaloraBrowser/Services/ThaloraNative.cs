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

    /// <summary>Navigate without executing JavaScript (Phase 1 of two-phase navigation).</summary>
    [LibraryImport(LibName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial IntPtr thalora_navigate_static(IntPtr inst, string url);

    /// <summary>Execute page scripts on the already-loaded page (Phase 2).
    /// Returns 1 if DOM changed, 0 if no change, -1 on error.</summary>
    [LibraryImport(LibName)]
    internal static partial int thalora_execute_page_scripts(IntPtr inst);

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

    // --- Styled Tree (new pipeline) ---

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_compute_styled_tree(IntPtr inst, float viewportW, float viewportH);

    // --- History API ---

    [LibraryImport(LibName)]
    internal static partial IntPtr thalora_poll_history_events(IntPtr inst);

    // --- Navigation Mode ---

    [LibraryImport(LibName)]
    internal static partial int thalora_set_navigation_mode(IntPtr inst, int mode);
}
