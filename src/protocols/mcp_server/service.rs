use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
};

use crate::engine::EngineConfig;
use crate::protocols::mcp_server::core::McpServer;

/// rmcp `ServerHandler` wrapper for `McpServer`.
///
/// rmcp's `&self` trait methods drive `McpServer`'s `&mut self` internals
/// through an async mutex. rmcp's `"local"` feature runs handlers with
/// `spawn_local`, which does **not** serialize them across `.await` points, so
/// overlapping requests (e.g. `tools/list` while a navigation is running)
/// wait for the lock instead of finding the server missing.
pub struct McpServerService(tokio::sync::Mutex<McpServer>);

impl McpServerService {
    pub fn new(server: McpServer) -> Self {
        Self(tokio::sync::Mutex::new(server))
    }

    pub fn with_engine(config: EngineConfig) -> Self {
        Self::new(McpServer::new_with_engine(config))
    }

    /// Run one tool call (serialized with all other calls on this server).
    pub(crate) async fn call_tool_inner(
        &self,
        name: String,
        arguments: serde_json::Value,
    ) -> crate::protocols::mcp::McpResponse {
        let mut server = self.0.lock().await;
        server.call_tool(name, arguments).await
    }
}

impl ServerHandler for McpServerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("thalora-mcp-server", env!("CARGO_PKG_VERSION")),
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let defs = self.0.lock().await.get_tool_definitions();
        let tools = defs
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.to_string();
        let arguments = request
            .arguments
            .map(serde_json::Value::Object)
            .unwrap_or_default();
        Ok(self.call_tool_inner(name, arguments).await.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Overlapping calls used to panic with "McpServer taken".
    #[test]
    fn overlapping_tool_calls_queue_instead_of_panicking() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();
        local.block_on(&runtime, async {
            let service = McpServerService::new(McpServer::new());
            let (a, b) = futures::join!(
                service.call_tool_inner("no_such_tool_a".into(), serde_json::json!({})),
                service.call_tool_inner("no_such_tool_b".into(), serde_json::json!({})),
            );
            assert!(a.is_error && b.is_error);
        });
    }
}
