//! HTTP MCP transport — stateful, thread-per-session model.
//!
//! Each MCP session (identified by `Mcp-Session-Id`) runs on a dedicated OS thread
//! with its own single-threaded tokio runtime and `McpServer` instance. This sidesteps
//! the `!Send` constraint imposed by `boa_engine::Context` (which uses `Rc` internally)
//! while still providing true per-session isolation.
//!
//! Protocol: MCP Streamable HTTP (2025-03-26) — JSON-only responses (no SSE streaming).
//! The spec permits returning `Content-Type: application/json` directly for non-streaming
//! tools, which covers all of Thalora's current tool set.

#![cfg(feature = "http-transport")]

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::engine::EngineConfig;
use crate::protocols::mcp_server::core::McpServer;

// ---------------------------------------------------------------------------
// Session message types
// ---------------------------------------------------------------------------

struct SessionMsg {
    method: String,
    params: Value,
    response_tx: oneshot::Sender<Value>,
}

// `SyncSender<SessionMsg>` is `Send` because `SessionMsg: Send`:
// - `String: Send`, `Value: Send`, `oneshot::Sender<Value>: Send`
struct SessionHandle {
    tx: std::sync::mpsc::SyncSender<SessionMsg>,
}

// ---------------------------------------------------------------------------
// Shared axum state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    sessions: Arc<RwLock<HashMap<String, SessionHandle>>>,
    engine_config: EngineConfig,
}

// ---------------------------------------------------------------------------
// Session worker thread
// ---------------------------------------------------------------------------

/// Spawn an OS thread that owns a `McpServer` and processes requests via a
/// sync channel.  Returns a `SessionHandle` for routing requests to this session.
fn spawn_session(engine_config: EngineConfig) -> SessionHandle {
    let (tx, rx) = std::sync::mpsc::sync_channel::<SessionMsg>(64);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("session tokio runtime");

        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let mut server = McpServer::new_with_engine(engine_config);

            while let Ok(msg) = rx.recv() {
                let result = dispatch(&mut server, &msg.method, msg.params).await;
                // Ignore send errors (client may have disconnected)
                let _ = msg.response_tx.send(result);
            }
        });
    });

    SessionHandle { tx }
}

/// Dispatch a single JSON-RPC method call on the session's `McpServer`.
async fn dispatch(server: &mut McpServer, method: &str, params: Value) -> Value {
    match method {
        "initialize" => {
            json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "thalora-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })
        }
        "tools/list" => {
            let defs = server.get_tool_definitions();
            json!({ "tools": defs })
        }
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = params.get("arguments").cloned().unwrap_or_default();
            let result = server.call_tool(name, arguments).await;
            // Convert McpResponse to a JSON-RPC-friendly Value
            let is_error = result.is_error;
            let content = result.content;
            json!({ "content": content, "isError": is_error })
        }
        "notifications/initialized" | "notifications/cancelled" | "notifications/progress" => {
            // Notifications don't require a response; return a sentinel
            json!(null)
        }
        other => {
            json!({
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {other}")
                }
            })
        }
    }
}

// ---------------------------------------------------------------------------
// axum route handlers
// ---------------------------------------------------------------------------

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let id = req.get("id").cloned().unwrap_or(json!(null));
    let method = req
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let params = req.get("params").cloned().unwrap_or_default();

    let is_initialize = method == "initialize";

    // Route / create session
    let (result, new_session_id): (Value, Option<String>) = if is_initialize {
        // New session
        let sid = uuid::Uuid::new_v4().to_string();
        let handle = spawn_session(state.engine_config.clone());
        let (response_tx, response_rx) = oneshot::channel();
        let _ = handle.tx.send(SessionMsg {
            method,
            params,
            response_tx,
        });
        state.sessions.write().insert(sid.clone(), handle);
        let result = response_rx.await.unwrap_or_default();
        (result, Some(sid))
    } else {
        // Existing session
        let sid = match headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
        {
            Some(s) => s,
            None => {
                return axum::response::Response::builder()
                    .status(400)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_string(&json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32600, "message": "Missing Mcp-Session-Id header" }
                        }))
                        .unwrap(),
                    ))
                    .unwrap();
            }
        };

        let (response_tx, response_rx) = oneshot::channel();
        let sent = {
            let sessions = state.sessions.read();
            if let Some(handle) = sessions.get(&sid) {
                handle
                    .tx
                    .send(SessionMsg {
                        method,
                        params,
                        response_tx,
                    })
                    .is_ok()
            } else {
                false
            }
        };

        if !sent {
            return axum::response::Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32001, "message": "Session not found or closed" }
                    }))
                    .unwrap(),
                ))
                .unwrap();
        }

        (response_rx.await.unwrap_or_default(), None)
    };

    // Notifications return null result — send 202 Accepted with no body
    if result.is_null() {
        let mut builder = axum::response::Response::builder()
            .status(202)
            .header("Content-Type", "application/json");
        if let Some(ref sid) = new_session_id {
            builder = builder.header("Mcp-Session-Id", sid.as_str());
        }
        return builder.body(axum::body::Body::empty()).unwrap();
    }

    // Build JSON-RPC response
    let body = if result.get("error").is_some() {
        json!({ "jsonrpc": "2.0", "id": id, "error": result["error"] })
    } else {
        json!({ "jsonrpc": "2.0", "id": id, "result": result })
    };

    let mut builder = axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "application/json");
    if let Some(ref sid) = new_session_id {
        builder = builder.header("Mcp-Session-Id", sid.as_str());
    }
    builder
        .body(axum::body::Body::from(
            serde_json::to_string(&body).unwrap(),
        ))
        .unwrap()
}

async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Start the HTTP MCP server.  Each session runs on a dedicated OS thread
/// so the `!Send` boa engine context is never sent across thread boundaries.
pub async fn run_http_transport(
    engine_config: EngineConfig,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let state = AppState {
        sessions: Arc::new(RwLock::new(HashMap::new())),
        engine_config,
    };

    let app = Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/health", get(health_handler))
        .with_state(state);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("🌐 Thalora HTTP MCP server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
