use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Json,
};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use tower_http::cors::CorsLayer;
use tracing::{error, info, trace};
use uuid::Uuid;

use crate::protocols::mcp::{McpRequest, McpNotification, McpResponse, McpMessage, McpMessageContent};
use super::super::core::McpServer;

/// A message sent from an axum handler to the server task.
enum ServerMessage {
    /// A JSON-RPC request — expects a response back.
    Request {
        request: McpRequest,
        reply: oneshot::Sender<McpResponse>,
    },
    /// A JSON-RPC notification — fire-and-forget.
    Notification {
        notification: McpNotification,
    },
    /// Cleanup request (for DELETE /mcp).
    Cleanup {
        reply: oneshot::Sender<()>,
    },
}

/// Shared application state for axum handlers.
#[derive(Clone)]
struct AppState {
    /// Channel to send messages to the server task.
    tx: mpsc::Sender<ServerMessage>,
    /// The session ID assigned on `initialize`. Protected by a tokio mutex (no Send issue here, it's just a String).
    session_id: Arc<TokioMutex<Option<String>>>,
}

/// Run the MCP server over HTTP using axum.
///
/// Implements the MCP Streamable HTTP transport:
/// - `POST /mcp` — JSON-RPC requests and notifications
/// - `GET /health` — Health check endpoint for Docker/load balancers
/// - `DELETE /mcp` — Session termination
///
/// Uses a channel-based architecture: the `McpServer` stays on a single task
/// (avoiding `Send` requirements), and axum handlers communicate via channels.
pub async fn run_http(mut server: McpServer, host: &str, port: u16) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    let state = AppState {
        tx,
        session_id: Arc::new(TokioMutex::new(None)),
    };

    let app = Router::new()
        .route("/mcp", post(handle_post).delete(handle_delete))
        .route("/health", get(handle_health))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let bind_addr = format!("{}:{}", host, port);
    eprintln!("MCP HTTP transport listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    // Spawn the server message loop on a LocalSet so it doesn't require Send.
    // We use tokio::task::spawn_local wrapped in a LocalSet.
    let local = tokio::task::LocalSet::new();

    // The server processing loop runs inside the LocalSet
    let server_handle = local.spawn_local(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                ServerMessage::Request { request, reply } => {
                    let response = server.handle_request(request).await;
                    // If the receiver dropped, just discard
                    let _ = reply.send(response);
                }
                ServerMessage::Notification { notification } => {
                    server.handle_notification(notification).await;
                }
                ServerMessage::Cleanup { reply } => {
                    server.cleanup().await;
                    let _ = reply.send(());
                }
            }
        }
        // Channel closed — cleanup
        server.cleanup().await;
    });

    // Run the LocalSet and axum server concurrently
    local.run_until(async move {
        // The axum server runs as a regular future within the LocalSet context
        let serve_result = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await;

        if let Err(e) = serve_result {
            error!("Axum server error: {}", e);
        }

        // Server is shutting down — drop the handle to close the channel
        drop(server_handle);
    }).await;

    eprintln!("MCP HTTP transport shut down");
    Ok(())
}

/// Handle POST /mcp — JSON-RPC requests and notifications.
async fn handle_post(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> axum::response::Response {
    trace!("POST /mcp body: {}", body);

    // Check if this is a request (has non-null id) or notification
    let request_id = body.get("id").filter(|id| !id.is_null());

    if let Some(request_id) = request_id {
        handle_json_rpc_request(&state, request_id.clone(), body).await
    } else {
        handle_json_rpc_notification(&state, body).await
    }
}

/// Process a JSON-RPC request (has `id`) and return a response.
async fn handle_json_rpc_request(
    state: &AppState,
    request_id: serde_json::Value,
    body: serde_json::Value,
) -> axum::response::Response {
    // Parse as McpRequest
    let request = match serde_json::from_value::<McpRequest>(body.clone()) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse MCP request: {}", e);
            let error_response = McpResponse::Error {
                error: format!("Invalid method or malformed request: {}", e),
            };
            let message = McpMessage {
                jsonrpc: "2.0".to_string(),
                id: Some(request_id),
                content: McpMessageContent::Response(error_response),
            };
            return (StatusCode::OK, Json(serde_json::to_value(&message).unwrap_or_default())).into_response();
        }
    };

    // Check if this is an Initialize request — we'll generate a session ID
    let is_initialize = matches!(&request, McpRequest::Initialize { .. });

    // Send request to server task and wait for response
    let (reply_tx, reply_rx) = oneshot::channel();
    if state.tx.send(ServerMessage::Request { request, reply: reply_tx }).await.is_err() {
        error!("Server task has shut down");
        return (StatusCode::SERVICE_UNAVAILABLE, "Server shutting down").into_response();
    }

    let response = match reply_rx.await {
        Ok(r) => r,
        Err(_) => {
            error!("Server task dropped reply channel");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Wrap in JSON-RPC 2.0 envelope
    let message = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(request_id),
        content: McpMessageContent::Response(response),
    };

    let json_value = match serde_json::to_value(&message) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // If this was an initialize request, generate and return session ID
    if is_initialize {
        let new_session_id = Uuid::new_v4().to_string();
        info!("New MCP session initialized: {}", new_session_id);
        *state.session_id.lock().await = Some(new_session_id.clone());

        return (
            StatusCode::OK,
            [("Mcp-Session-Id", new_session_id)],
            Json(json_value),
        ).into_response();
    }

    (StatusCode::OK, Json(json_value)).into_response()
}

/// Process a JSON-RPC notification (no `id`) — returns 202 Accepted.
async fn handle_json_rpc_notification(
    state: &AppState,
    body: serde_json::Value,
) -> axum::response::Response {
    match serde_json::from_value::<McpNotification>(body) {
        Ok(notification) => {
            if state.tx.send(ServerMessage::Notification { notification }).await.is_err() {
                error!("Server task has shut down");
            }
        }
        Err(e) => {
            error!("Failed to parse notification: {}", e);
        }
    }

    StatusCode::ACCEPTED.into_response()
}

/// Handle GET /health — returns 200 with "ok" body for Docker health checks.
async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// Handle DELETE /mcp — session termination.
async fn handle_delete(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut session_id = state.session_id.lock().await;
    if let Some(id) = session_id.take() {
        info!("MCP session terminated: {}", id);
        // Send cleanup to server task
        let (reply_tx, reply_rx) = oneshot::channel();
        if state.tx.send(ServerMessage::Cleanup { reply: reply_tx }).await.is_ok() {
            let _ = reply_rx.await;
        }
        (StatusCode::OK, format!("Session {} terminated", id))
    } else {
        (StatusCode::NOT_FOUND, "No active session".to_string())
    }
}

/// Wait for OS shutdown signals (SIGTERM, SIGINT) for graceful shutdown.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                eprintln!("HTTP transport received SIGTERM");
            }
            _ = sigint.recv() => {
                eprintln!("HTTP transport received SIGINT");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.expect("Failed to set up Ctrl+C handler");
        eprintln!("HTTP transport received Ctrl+C");
    }
}
