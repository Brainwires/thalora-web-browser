// Chrome Features Integration Tests
// This file runs all Chrome feature compatibility tests

use thalora::HeadlessWebBrowser;

// NOTE: websocketstream_api.rs file is missing - disabled for now
// mod chrome_124_websocketstream_api {
//     use super::*;
//     include!("chrome_features/124/websocketstream_api.rs");
// }

mod chrome_124_streams_async_iteration {
    use super::*;
    include!("chrome_features/124/streams_async_iteration.rs");
}

mod chrome_124_dom_html_unsafe_methods {
    use super::*;
    include!("chrome_features/124/dom_html_unsafe_methods.rs");
}

mod chrome_124_pageswap_event {
    use super::*;
    include!("chrome_features/124/pageswap_event.rs");
}

mod chrome_124_webgpu_enhancements {
    use super::*;
    include!("chrome_features/124/webgpu_enhancements.rs");
}

mod chrome_124_webmidi_permissions {
    use super::*;
    include!("chrome_features/124/webmidi_permissions.rs");
}

mod chrome_124_client_hints {
    use super::*;
    include!("chrome_features/124/client_hints.rs");
}

mod chrome_124_overall_compatibility {
    use super::*;
    include!("chrome_features/124/overall_compatibility.rs");
}

// Chrome 137 Tests
mod chrome_137_selection_api_comprehensive {
    use super::*;
    include!("chrome_features/137/selection_api_comprehensive.rs");
}

mod chrome_137_selection_frame_selection {
    use super::*;
    include!("chrome_features/137/selection_frame_selection.rs");
}

mod chrome_137_range_api_comprehensive {
    use super::*;
    include!("chrome_features/137/range_api_comprehensive.rs");
}

mod chrome_137_selection_get_composed_ranges {
    use super::*;
    include!("chrome_features/137/selection_get_composed_ranges.rs");
}

mod chrome_137_selection_direction {
    use super::*;
    include!("chrome_features/137/selection_direction.rs");
}