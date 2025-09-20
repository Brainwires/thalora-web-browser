#![allow(dead_code)]

//! Local WebSocket echo server for testing
//!
//! This creates a simple WebSocket server that echoes back messages
//! for use in tests without depending on external services

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

pub struct WebSocketEchoServer {
    port: u16,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,
}

impl WebSocketEchoServer {
    /// Create a new WebSocket echo server on an available port
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

        let server_handle = tokio::spawn(async move {
            Self::run_server(listener, shutdown_receiver).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(Self {
            port,
            server_handle: Some(server_handle),
            shutdown_sender: Some(shutdown_sender),
        })
    }

    /// Get the WebSocket URL for this server
    pub fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Run the WebSocket echo server
    async fn run_server(listener: TcpListener, mut shutdown_receiver: tokio::sync::oneshot::Receiver<()>) {
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = &mut shutdown_receiver => {
                    println!("WebSocket echo server shutting down");
                    break;
                }
                // Handle new connections
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            println!("New WebSocket connection from: {}", addr);
                            tokio::spawn(Self::handle_connection(stream));
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Handle a single WebSocket connection
    async fn handle_connection(stream: TcpStream) {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                return;
            }
        };

        println!("WebSocket connection established");

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Echo loop
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {}", text);
                    if let Err(e) = ws_sender.send(Message::Text(text)).await {
                        eprintln!("Failed to send text message: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    println!("Received binary data: {} bytes", data.len());
                    if let Err(e) = ws_sender.send(Message::Binary(data)).await {
                        eprintln!("Failed to send binary message: {}", e);
                        break;
                    }
                }
                Ok(Message::Ping(data)) => {
                    println!("Received ping");
                    if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                        eprintln!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    println!("Received pong");
                    // Just acknowledge pongs, don't echo them back
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed by client");
                    break;
                }
                Ok(Message::Frame(_)) => {
                    // Handle raw frames (usually handled automatically)
                    continue;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        println!("WebSocket connection ended");
    }

    /// Shutdown the server gracefully
    pub async fn shutdown(mut self) {
        println!("🔄 Shutting down WebSocket server on port {}", self.port);

        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                handle
            ).await {
                Ok(_) => println!("✅ WebSocket server shut down successfully"),
                Err(_) => println!("⚠️ WebSocket server shutdown timed out"),
            }
        }
    }
}

impl Drop for WebSocketEchoServer {
    fn drop(&mut self) {
        println!("🗑️ WebSocket server on port {} being dropped", self.port);
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
    }
}

/// Create a new isolated WebSocket echo server for a single test
/// This ensures complete test isolation and eliminates race conditions
pub async fn create_isolated_test_server() -> Result<WebSocketEchoServer, Box<dyn std::error::Error>> {
    let server = WebSocketEchoServer::new().await?;
    let port = server.port;
    println!("🚀 Started isolated WebSocket echo server on port: {}", port);
    Ok(server)
}

/// Get the URL for a test server
pub fn get_server_url(server: &WebSocketEchoServer) -> String {
    server.url()
}

/// Test utility to create a server and return the URL
pub async fn start_echo_server() -> Result<(WebSocketEchoServer, String), Box<dyn std::error::Error>> {
    let server = WebSocketEchoServer::new().await?;
    let url = server.url();
    Ok((server, url))
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};

    #[tokio::test]
    async fn test_echo_server() {
        // Create isolated test server
        let server = create_isolated_test_server().await.unwrap();
        let url = get_server_url(&server);

        // Connect to the server
        let (ws_stream, _) = connect_async(&url).await.unwrap();
        let (mut sender, mut receiver) = ws_stream.split();

        // Send a text message
        sender.send(Message::Text("Hello".to_string())).await.unwrap();

        // Receive the echo
        let response = receiver.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Text("Hello".to_string()));

        // Send a binary message
        let binary_data = vec![1, 2, 3, 4, 5];
        sender.send(Message::Binary(binary_data.clone())).await.unwrap();

        // Receive the binary echo
        let response = receiver.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Binary(binary_data));

        // Test ping-pong
        let ping_data = vec![10, 20, 30];
        sender.send(Message::Ping(ping_data.clone())).await.unwrap();

        // Receive the pong
        let response = receiver.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Pong(ping_data));

        // Close connection
        sender.send(Message::Close(None)).await.unwrap();

        // Shutdown the isolated server
        server.shutdown().await;
    }
}