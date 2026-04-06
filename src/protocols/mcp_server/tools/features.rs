use std::env;

/// Feature flag management for tool availability
/// Check if the BrainClaw preset is active.
///
/// `THALORA_PRESET=brainclaw` enables scraping, search, sessions, and CDP in one shot
/// without needing five separate environment variables. It also disables minimal mode
/// and registers agent-friendly alias tools.
pub(crate) fn is_brainclaw_preset() -> bool {
    env::var("THALORA_PRESET")
        .map(|v| v.eq_ignore_ascii_case("brainclaw"))
        .unwrap_or(false)
}

/// Check if sessions are enabled (either directly or through CDP dependency or brainclaw preset)
pub(super) fn is_sessions_enabled() -> bool {
    is_brainclaw_preset()
        || env::var("THALORA_ENABLE_SESSIONS").unwrap_or_else(|_| "false".to_string()) == "true"
        || env::var("THALORA_ENABLE_CDP").unwrap_or_else(|_| "false".to_string()) == "true"
}

/// Check if AI memory tools are enabled
pub(super) fn is_ai_memory_enabled() -> bool {
    env::var("THALORA_ENABLE_AI_MEMORY").unwrap_or_else(|_| "false".to_string()) == "true"
}

/// Check if CDP tools are enabled
pub(super) fn is_cdp_enabled() -> bool {
    is_brainclaw_preset()
        || env::var("THALORA_ENABLE_CDP").unwrap_or_else(|_| "false".to_string()) == "true"
}

/// Check if scraping tools are enabled (enabled by default)
pub(super) fn is_scraping_enabled() -> bool {
    env::var("THALORA_ENABLE_SCRAPING").unwrap_or_else(|_| "true".to_string()) == "true"
}

/// Check if search tools are enabled
pub(super) fn is_search_enabled() -> bool {
    is_brainclaw_preset()
        || env::var("THALORA_ENABLE_SEARCH").unwrap_or_else(|_| "false".to_string()) == "true"
}

/// Get MCP mode - minimal (default for MCP) or full (all features)
pub(super) fn get_mcp_mode() -> String {
    env::var("THALORA_MCP_MODE").unwrap_or_else(|_| "minimal".to_string())
}

/// Check if we're in minimal mode.
///
/// Never true in brainclaw preset — brainclaw always runs full mode.
pub(super) fn is_minimal_mode() -> bool {
    !is_brainclaw_preset() && get_mcp_mode() == "minimal"
}

/// Check if WASM debug tools are enabled (requires wasm-debug feature at compile time)
#[cfg(feature = "wasm-debug")]
pub(crate) fn is_wasm_debug_enabled() -> bool {
    env::var("THALORA_ENABLE_WASM_DEBUG").unwrap_or_else(|_| "false".to_string()) == "true"
}

#[cfg(not(feature = "wasm-debug"))]
pub(crate) fn is_wasm_debug_enabled() -> bool {
    false
}
