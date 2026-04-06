use anyhow::Result;
use futures::FutureExt;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use vfs::VfsInstance;

use crate::engine::browser::HeadlessWebBrowser;
// websocket API is now natively implemented in Boa engine
// DOM is now natively handled by Boa engine
use crate::engine::EngineConfig;
use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::browser_tools::BrowserTools;
use crate::protocols::cdp::CdpServer;
use crate::protocols::cdp_tools::CdpTools;
use crate::protocols::memory_tools::MemoryTools;
use crate::protocols::rate_limiter::RateLimiter;
use crate::protocols::security::sanitize_session_id;
use crate::protocols::session_manager::SessionManager;
#[cfg(feature = "wasm-debug")]
use crate::protocols::wasm_debug_tools::WasmDebugTools;

#[allow(dead_code)]
pub struct McpServer {
    // websocket API is now natively implemented in Boa engine
    pub(super) ai_memory: AiMemoryHeap,
    pub(super) cdp_server: CdpServer,
    pub(super) memory_tools: MemoryTools,
    pub(super) cdp_tools: CdpTools,
    /// Shared browser tools - wrapped in Rc to share with CdpTools
    pub(super) browser_tools: Rc<BrowserTools>,
    pub(super) session_manager: SessionManager,
    /// Optional session-scoped persistent VFS instances. Keyed by session_id.
    pub(super) session_vfs: Arc<Mutex<HashMap<String, Arc<VfsInstance>>>>,
    /// Engine configuration for JavaScript execution
    pub(super) engine_config: EngineConfig,
    /// Rate limiter for DoS prevention
    pub(super) rate_limiter: RateLimiter,
    /// WASM debug tools (optional, requires wasm-debug feature + env var)
    #[cfg(feature = "wasm-debug")]
    pub(super) wasm_debug_tools: Option<WasmDebugTools>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    pub fn new() -> Self {
        // Default to Boa engine for backward compatibility
        Self::new_with_engine(EngineConfig::new(false).unwrap_or(EngineConfig {
            engine_type: crate::engine::EngineType::Boa,
        }))
    }

    pub fn new_with_engine(engine_config: EngineConfig) -> Self {
        // Only enable AI memory if THALORA_ENABLE_AI_MEMORY is set to a truthy value
        // Accepts: "1", "true", "yes", "on" (case-insensitive)
        // Rejects: "0", "false", "no", "off", "" (empty), or unset
        let ai_memory_enabled = std::env::var("THALORA_ENABLE_AI_MEMORY")
            .map(|v| {
                let val = v.trim().to_lowercase();
                !val.is_empty() && val != "0" && val != "false" && val != "no" && val != "off"
            })
            .unwrap_or(false);

        let ai_memory = if ai_memory_enabled {
            tracing::info!("AI memory enabled via THALORA_ENABLE_AI_MEMORY");
            AiMemoryHeap::new_default().unwrap_or_else(|e| {
                tracing::warn!("Failed to load AI memory heap: {}, creating new one", e);
                AiMemoryHeap::new("/tmp/thalora_ai_memory.json")
                    .expect("Failed to create AI memory heap")
            })
        } else {
            // Create disabled AI memory (in-memory only, no persistence)
            tracing::info!(
                "AI memory disabled. Set THALORA_ENABLE_AI_MEMORY=1 to enable persistent storage"
            );
            AiMemoryHeap::new("/dev/null").unwrap_or_else(|_| {
                // Fallback to temp file if /dev/null fails (Windows)
                AiMemoryHeap::new(std::env::temp_dir().join("thalora_disabled.json"))
                    .expect("Failed to create disabled AI memory")
            })
        };

        let session_manager = SessionManager::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create session manager: {}", e);
            panic!("Could not initialize session manager: {}", e);
        });

        // Create shared BrowserTools instance
        let browser_tools = Rc::new(BrowserTools::new());

        #[cfg(feature = "wasm-debug")]
        let wasm_debug_tools = {
            use crate::protocols::mcp_server::tools::features::is_wasm_debug_enabled;
            if is_wasm_debug_enabled() {
                match WasmDebugTools::new() {
                    Ok(tools) => {
                        tracing::info!("WASM debug tools enabled via THALORA_ENABLE_WASM_DEBUG");
                        Some(tools)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize WASM debug tools: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };

        Self {
            // websocket API is now natively implemented in Boa engine
            ai_memory,
            cdp_server: CdpServer::new(),
            memory_tools: MemoryTools::new(),
            // Share BrowserTools instance with CdpTools to avoid duplicate Drop calls
            cdp_tools: CdpTools::with_browser_tools(browser_tools.clone()),
            browser_tools,
            session_manager,
            session_vfs: Arc::new(Mutex::new(HashMap::new())),
            engine_config,
            rate_limiter: RateLimiter::new(),
            #[cfg(feature = "wasm-debug")]
            wasm_debug_tools,
        }
    }

    /// Get or create a session-scoped VFS. If `backing_dir` is provided, the backing file will be created there.
    ///
    /// # Security
    /// - The session_id is validated to prevent path traversal attacks (CWE-22).
    /// - Session data is encrypted at rest using ChaCha20-Poly1305.
    /// - Only alphanumeric characters, hyphens, and underscores are allowed.
    pub fn get_or_create_session_vfs(
        &self,
        session_id: &str,
        backing_dir: Option<&std::path::Path>,
    ) -> Result<Arc<VfsInstance>> {
        // SECURITY: Validate session_id to prevent path traversal attacks
        let safe_session_id = sanitize_session_id(session_id)?;

        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.get(&safe_session_id) {
            return Ok(v.clone());
        }

        // Build a backing path: backing_dir/vfs-session-<session_id>.bin.enc
        // SECURITY: Use .bin.enc extension for encrypted session files
        let file = if let Some(dir) = backing_dir {
            dir.join(format!("vfs-session-{}.bin.enc", safe_session_id))
        } else {
            // fallback to temp dir
            std::env::temp_dir().join(format!("vfs-session-{}.bin.enc", safe_session_id))
        };

        // SECURITY: Derive encryption key from session_id and secret
        let key = vfs::derive_session_key(&safe_session_id);

        // Try to open existing encrypted file or create new
        let v = match VfsInstance::open_file_backed_encrypted(&file, &key) {
            Ok(inst) => Arc::new(inst),
            Err(e) => {
                // Log decryption failure but create fresh VFS
                tracing::warn!("Failed to open encrypted session VFS: {}. Creating new.", e);
                Arc::new(
                    VfsInstance::new_temp_in_dir(std::env::temp_dir()).expect("create temp vfs"),
                )
            }
        };
        guard.insert(safe_session_id, v.clone());
        Ok(v)
    }

    /// Remove and optionally delete the backing file for a session VFS.
    ///
    /// # Security
    /// The session_id is validated to prevent path traversal attacks (CWE-22).
    pub fn remove_session_vfs(&self, session_id: &str, delete_backing: bool) -> Result<()> {
        // SECURITY: Validate session_id to prevent path traversal attacks
        let safe_session_id = sanitize_session_id(session_id)?;

        let mut guard = self.session_vfs.lock().unwrap();
        if let Some(v) = guard.remove(&safe_session_id)
            && delete_backing
        {
            drop(v.delete_backing_file());
        }
        Ok(())
    }

    /// Cleanup all sessions and resources
    pub async fn cleanup(&self) {
        self.session_manager.shutdown().await;
    }

    /// Run the MCP server over stdio using rmcp.
    ///
    /// Uses `tokio::task::LocalSet` so the `!Send` boa engine (`Rc`-based) is safe.
    #[cfg(feature = "http-transport")]
    pub async fn run(self) -> Result<()> {
        use crate::protocols::mcp_server::service::McpServerService;
        use rmcp::ServiceExt;

        let local = tokio::task::LocalSet::new();
        let service = McpServerService::new(self);
        let (stdin, stdout) = rmcp::transport::io::stdio();
        local
            .run_until(async move {
                let running = service
                    .serve((stdin, stdout))
                    .await
                    .map_err(|e| anyhow::anyhow!("MCP server error: {e}"))?;
                running
                    .waiting()
                    .await
                    .map_err(|e| anyhow::anyhow!("MCP server wait error: {e}"))?;
                Ok(())
            })
            .await
    }
}
