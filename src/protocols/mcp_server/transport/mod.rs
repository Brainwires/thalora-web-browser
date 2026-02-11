/// Transport layer for the MCP server.
///
/// Supports both stdio (default, backward-compatible) and HTTP transports.
pub mod stdio;
pub mod http;

/// Available transport mechanisms for the MCP server.
#[derive(Debug, Clone, Copy)]
pub enum McpTransport {
    /// Standard I/O transport (default) — reads JSON-RPC from stdin, writes to stdout.
    Stdio,
    /// HTTP transport — serves JSON-RPC over HTTP POST /mcp.
    Http,
}

/// Configuration for the HTTP transport.
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Host to bind to (e.g. "0.0.0.0" or "127.0.0.1").
    pub host: String,
    /// Port to listen on.
    pub port: u16,
}
