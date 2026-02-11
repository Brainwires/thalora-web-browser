#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Core modules organized by functionality
pub mod engine;
pub mod apis;
pub mod features;
pub mod protocols;

// GUI module for graphical browser interface
#[cfg(feature = "gui")]
pub mod gui;

use protocols::mcp_server::McpServer;
use engine::EngineConfig;

/// Global flag to signal shutdown
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Check if shutdown was requested
pub fn is_shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Request shutdown
pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Install a custom panic hook that logs panics to stderr instead of crashing
/// This helps prevent "broken pipe" errors when panics occur in async tasks
fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Log the panic to stderr with full details
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        eprintln!("╔══════════════════════════════════════════════════════════════╗");
        eprintln!("║                    🚨 PANIC CAUGHT 🚨                        ║");
        eprintln!("╠══════════════════════════════════════════════════════════════╣");
        eprintln!("║ Location: {}", location);
        eprintln!("║ Message: {}", payload);
        eprintln!("╚══════════════════════════════════════════════════════════════╝");

        // Also call the default hook for backtrace if RUST_BACKTRACE is set
        if std::env::var("RUST_BACKTRACE").is_ok() {
            default_hook(panic_info);
        }
    }));
}

#[derive(Parser)]
#[command(name = "thalora")]
#[command(about = "Headless web browser written entirely in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as MCP server (default mode)
    Server {
        /// MCP mode: 'minimal' for basic scraping (default), 'full' for all features
        #[arg(long, default_value = "minimal")]
        mcp_mode: String,
        /// Transport: 'stdio' (default) or 'http'
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Port for HTTP transport (ignored for stdio)
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Host to bind for HTTP transport (ignored for stdio)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Run as browser session process
    Session {
        /// Session identifier
        #[arg(long)]
        session_id: String,
        /// Unix socket path for communication
        #[arg(long)]
        socket_path: String,
        /// Whether this is a persistent session
        #[arg(long)]
        persistent: bool,
    },
    /// Run as graphical web browser
    Browser {
        /// Initial URL to load
        #[arg(long)]
        url: Option<String>,
        /// Window width in pixels
        #[arg(long, default_value = "1200")]
        width: u32,
        /// Window height in pixels
        #[arg(long, default_value = "800")]
        height: u32,
        /// Enable fullscreen mode
        #[arg(long)]
        fullscreen: bool,
        /// Enable debug mode with developer tools
        #[arg(long)]
        debug: bool,
    },
    /// Run as display server for remote browser UI
    DisplayServer {
        /// Port to listen on
        #[arg(long, default_value = "9090")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install custom panic hook early to catch all panics
    install_panic_hook();

    let cli = Cli::parse();

    // Create engine configuration
    let engine_config = EngineConfig::new();

    // Log the selected engine
    if std::env::var("THALORA_SILENT").is_err() {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();

        tracing::info!("Using {} JavaScript engine", engine_config.engine_type);
    } else {
        // Still configure tracing but silent
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();
    }

    // Set up signal handling for graceful shutdown
    let shutdown_signal = setup_signal_handler();

    match cli.command {
        Some(Commands::Session { session_id, socket_path, persistent }) => {
            // Run as browser session process with signal handling
            tokio::select! {
                result = run_browser_session(session_id, socket_path, persistent, engine_config) => result,
                _ = shutdown_signal => {
                    eprintln!("🛑 Session received shutdown signal");
                    Ok(())
                }
            }
        }
        Some(Commands::Browser { url, width, height, fullscreen, debug }) => {
            // Run as graphical web browser
            run_graphical_browser(url, width, height, fullscreen, debug, engine_config).await
        }
        Some(Commands::DisplayServer { port, host }) => {
            // Run as display server
            run_display_server(host, port).await
        }
        Some(Commands::Server { mcp_mode, transport, port, host }) => {
            // Run as MCP server with specified mode
            // SAFETY: This is called at program startup before any threads are spawned
            unsafe { std::env::set_var("THALORA_MCP_MODE", &mcp_mode) };

            match transport.as_str() {
                "http" => {
                    eprintln!("Starting Thalora MCP Server in '{}' mode (HTTP transport on {}:{})", mcp_mode, host, port);
                    let server = McpServer::new_with_engine(engine_config);
                    // HTTP transport handles its own graceful shutdown internally
                    server.run_http(&host, port).await
                }
                _ => {
                    // Default to stdio
                    eprintln!("Starting Thalora MCP Server in '{}' mode (stdio transport)", mcp_mode);
                    let mut server = McpServer::new_with_engine(engine_config);
                    // Run server with signal handling
                    tokio::select! {
                        result = server.run_stdio() => result,
                        _ = shutdown_signal => {
                            eprintln!("MCP Server received shutdown signal, cleaning up...");
                            request_shutdown();
                            server.cleanup().await;
                            eprintln!("MCP Server shutdown complete");
                            Ok(())
                        }
                    }
                }
            }
        }
        None => {
            // Run as MCP server (default mode, stdio transport)
            // SAFETY: This is called at program startup before any threads are spawned
            unsafe { std::env::set_var("THALORA_MCP_MODE", "minimal") };
            eprintln!("Starting Thalora MCP Server in 'minimal' mode (stdio transport)");

            let mut server = McpServer::new_with_engine(engine_config);

            // Run server with signal handling
            tokio::select! {
                result = server.run_stdio() => result,
                _ = shutdown_signal => {
                    eprintln!("MCP Server received shutdown signal, cleaning up...");
                    request_shutdown();
                    server.cleanup().await;
                    eprintln!("MCP Server shutdown complete");
                    Ok(())
                }
            }
        }
    }
}

/// Set up signal handlers for graceful shutdown
async fn setup_signal_handler() {
    use tokio::signal;

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to set up SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt())
            .expect("Failed to set up SIGINT handler");
        let mut sighup = signal(SignalKind::hangup())
            .expect("Failed to set up SIGHUP handler");

        tokio::select! {
            _ = sigterm.recv() => {
                eprintln!("📡 Received SIGTERM");
            }
            _ = sigint.recv() => {
                eprintln!("📡 Received SIGINT");
            }
            _ = sighup.recv() => {
                eprintln!("📡 Received SIGHUP");
            }
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, just wait for Ctrl+C
        signal::ctrl_c().await.expect("Failed to set up Ctrl+C handler");
        eprintln!("📡 Received Ctrl+C");
    }
}

/// Run as graphical web browser
async fn run_graphical_browser(
    initial_url: Option<String>, 
    width: u32, 
    height: u32, 
    fullscreen: bool, 
    debug_mode: bool, 
    engine_config: EngineConfig
) -> Result<()> {
    tracing::info!("Starting graphical browser mode");
    tracing::info!("Window size: {}x{}", width, height);
    tracing::info!("Fullscreen: {}", fullscreen);
    tracing::info!("Debug mode: {}", debug_mode);
    
    if let Some(url) = &initial_url {
        tracing::info!("Initial URL: {}", url);
    }

    // Initialize the GUI browser
    // This will be implemented in the gui module
    #[cfg(feature = "gui")]
    {
        use crate::gui::GraphicalBrowser;
        let mut browser = GraphicalBrowser::new(width, height, fullscreen, debug_mode, engine_config)?;
        if let Some(url) = initial_url {
            browser.navigate_to(&url).await?;
        }
        browser.run().await
    }
    
    #[cfg(not(feature = "gui"))]
    {
        anyhow::bail!("GUI mode not available. Recompile with --features gui to enable graphical browser mode.");
    }
}

/// Run as browser session process
async fn run_browser_session(session_id: String, socket_path: String, persistent: bool, engine_config: EngineConfig) -> Result<()> {
    use protocols::session_manager::{BrowserCommand, BrowserResponse};
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
    use tokio::net::{UnixListener, UnixStream};
    use tracing::{info, error, debug};
    use anyhow::Context;
    use engine::browser::HeadlessWebBrowser;

    /// Browser session handler (moved from separate binary)
    struct BrowserSessionHandler {
        session_id: String,
        browser: Arc<Mutex<HeadlessWebBrowser>>,
        persistent: bool,
        engine_config: EngineConfig,
    }

    impl BrowserSessionHandler {
        fn new(session_id: String, persistent: bool, engine_config: EngineConfig) -> Self {
            let browser = HeadlessWebBrowser::new(); // This already returns Arc<Mutex<HeadlessWebBrowser>>

            Self {
                session_id,
                browser,
                persistent,
                engine_config,
            }
        }

        /// Handle a browser command and return a response
        async fn handle_command(&self, command: BrowserCommand) -> BrowserResponse {
            debug!("Handling command: {:?}", command);

            match command {
                BrowserCommand::Navigate { url } => {
                    match self.navigate(&url).await {
                        Ok(content) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "content": content,
                                "url": url
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to navigate to {}: {}", url, e)
                        }
                    }
                },

                BrowserCommand::NavigateBack => {
                    match self.go_back().await {
                        Ok(Some(url)) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "navigated": true,
                                "url": url
                            })
                        },
                        Ok(None) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "navigated": false,
                                "reason": "At beginning of history"
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to navigate back: {}", e)
                        }
                    }
                },

                BrowserCommand::NavigateForward => {
                    match self.go_forward().await {
                        Ok(Some(url)) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "navigated": true,
                                "url": url
                            })
                        },
                        Ok(None) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "navigated": false,
                                "reason": "At end of history"
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to navigate forward: {}", e)
                        }
                    }
                },

                BrowserCommand::Reload => {
                    match self.reload().await {
                        Ok(content) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "reloaded": true,
                                "content_length": content.len()
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to reload page: {}", e)
                        }
                    }
                },

                BrowserCommand::Stop => {
                    // For a headless browser, stop is largely a no-op since we don't have
                    // persistent async loading in the same way graphical browsers do
                    BrowserResponse::Success {
                        data: serde_json::json!({
                            "stopped": true
                        })
                    }
                },

                BrowserCommand::ExecuteJs { code } => {
                    match self.execute_javascript(&code).await {
                        Ok(result) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "result": result
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to execute JavaScript: {}", e)
                        }
                    }
                },

                BrowserCommand::GetContent => {
                    match self.get_page_content().await {
                        Ok(content) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "content": content
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to get page content: {}", e)
                        }
                    }
                },

                BrowserCommand::Click { selector } => {
                    match self.click_element(&selector).await {
                        Ok(success) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "clicked": success,
                                "selector": selector
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to click element {}: {}", selector, e)
                        }
                    }
                },

                BrowserCommand::Fill { selector, value } => {
                    match self.fill_element(&selector, &value).await {
                        Ok(success) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "filled": success,
                                "selector": selector,
                                "value": value
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to fill element {}: {}", selector, e)
                        }
                    }
                },

                BrowserCommand::Screenshot => {
                    BrowserResponse::Success {
                        data: serde_json::json!({
                            "screenshot": "Not implemented yet",
                            "note": "Screenshot functionality will be implemented in a future update"
                        })
                    }
                },

                BrowserCommand::GetCookies => {
                    match self.get_cookies().await {
                        Ok(cookies) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "cookies": cookies
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to get cookies: {}", e)
                        }
                    }
                },

                BrowserCommand::SetCookies { cookies } => {
                    match self.set_cookies(cookies).await {
                        Ok(count) => BrowserResponse::Success {
                            data: serde_json::json!({
                                "set_count": count
                            })
                        },
                        Err(e) => BrowserResponse::Error {
                            message: format!("Failed to set cookies: {}", e)
                        }
                    }
                },

                BrowserCommand::Close => {
                    info!("Received close command for session: {}", self.session_id);
                    BrowserResponse::Success {
                        data: serde_json::json!({
                            "closed": true,
                            "session_id": self.session_id
                        })
                    }
                },
            }
        }

        /// Navigate to a URL
        async fn navigate(&self, url: &str) -> Result<String> {
            let mut browser = self.browser.lock().unwrap();
            browser.navigate_to(url).await
                .context("Failed to navigate")
        }

        /// Go back in navigation history
        async fn go_back(&self) -> Result<Option<String>> {
            let mut browser = self.browser.lock().unwrap();
            browser.go_back().await
                .context("Failed to go back")
        }

        /// Go forward in navigation history
        async fn go_forward(&self) -> Result<Option<String>> {
            let mut browser = self.browser.lock().unwrap();
            browser.go_forward().await
                .context("Failed to go forward")
        }

        /// Reload the current page
        async fn reload(&self) -> Result<String> {
            let mut browser = self.browser.lock().unwrap();
            browser.reload().await
                .context("Failed to reload")
        }

        /// Execute JavaScript
        async fn execute_javascript(&self, code: &str) -> Result<serde_json::Value> {
            let mut browser = self.browser.lock().unwrap();
            let result = browser.execute_javascript(code).await?;
            Ok(serde_json::json!(result))
        }

        /// Get page content
        async fn get_page_content(&self) -> Result<String> {
            let browser = self.browser.lock().unwrap();
            Ok(browser.get_current_content())
        }

        /// Click an element (simplified implementation)
        async fn click_element(&self, selector: &str) -> Result<bool> {
            // For now, use JavaScript to click elements
            let click_js = format!(
                "document.querySelector('{}')?.click(); true",
                selector.replace("'", "\\'")
            );

            let mut browser = self.browser.lock().unwrap();
            let result = browser.execute_javascript(&click_js).await?;
            Ok(result.contains("true"))
        }

        /// Fill a form element (simplified implementation)
        async fn fill_element(&self, selector: &str, value: &str) -> Result<bool> {
            // Use JavaScript to fill form elements
            let fill_js = format!(
                "var el = document.querySelector('{}'); if(el) {{ el.value = '{}'; el.dispatchEvent(new Event('input')); el.dispatchEvent(new Event('change')); true }} else {{ false }}",
                selector.replace("'", "\\'"),
                value.replace("'", "\\'")
            );

            let mut browser = self.browser.lock().unwrap();
            let result = browser.execute_javascript(&fill_js).await?;
            Ok(result.contains("true"))
        }

        /// Get cookies (simplified implementation)
        async fn get_cookies(&self) -> Result<Vec<String>> {
            // Use JavaScript to get cookies
            let mut browser = self.browser.lock().unwrap();
            let result = browser.execute_javascript("document.cookie").await?;

            // Parse cookie string into individual cookies
            let cookies: Vec<String> = result
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Ok(cookies)
        }

        /// Set cookies (simplified implementation)
        async fn set_cookies(&self, cookies: Vec<String>) -> Result<usize> {
            let mut count = 0;
            let mut browser = self.browser.lock().unwrap();

            for cookie in cookies {
                let set_cookie_js = format!("document.cookie = '{}'", cookie.replace("'", "\\'"));
                if browser.execute_javascript(&set_cookie_js).await.is_ok() {
                    count += 1;
                }
            }

            Ok(count)
        }
    }

    /// Handle a single connection from the session manager
    async fn handle_connection(
        stream: UnixStream,
        handler: Arc<BrowserSessionHandler>,
    ) -> Result<()> {
        let (reader, writer) = split(stream);
        let mut reader = BufReader::new(reader);
        let mut writer = writer;

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Parse command
                    match serde_json::from_str::<BrowserCommand>(line) {
                        Ok(command) => {
                            // Handle close command specially
                            let should_exit = matches!(command, BrowserCommand::Close);

                            // Process command
                            let response = handler.handle_command(command).await;

                            // Send response
                            let response_json = serde_json::to_string(&response)
                                .context("Failed to serialize response")?;

                            writer.write_all(response_json.as_bytes()).await
                                .context("Failed to write response")?;
                            writer.write_all(b"\n").await
                                .context("Failed to write newline")?;

                            // Exit if close command was received
                            if should_exit {
                                info!("Closing session: {}", handler.session_id);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse command: {}", e);
                            let error_response = BrowserResponse::Error {
                                message: format!("Invalid command: {}", e),
                            };

                            let response_json = serde_json::to_string(&error_response)
                                .context("Failed to serialize error response")?;

                            writer.write_all(response_json.as_bytes()).await
                                .context("Failed to write error response")?;
                            writer.write_all(b"\n").await
                                .context("Failed to write newline")?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from connection: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    info!("Starting browser session: {}", session_id);
    info!("Socket path: {}", socket_path);
    info!("Persistent: {}", persistent);

    // Create session handler
    let handler = Arc::new(BrowserSessionHandler::new(
        session_id.clone(),
        persistent,
        engine_config,
    ));

    // Remove existing socket file if it exists
    if std::path::Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)
            .context("Failed to remove existing socket file")?;
    }

    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)
        .context("Failed to bind Unix socket")?;

    info!("Browser session listening on socket: {}", socket_path);

    // Handle incoming connections sequentially (avoiding Send requirements)
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                if let Err(e) = handle_connection(stream, handler.clone()).await {
                    error!("Error handling connection for session {}: {}", session_id, e);
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                break;
            }
        }
    }

    info!("Browser session {} shutting down", session_id);
    Ok(())
}

/// Run as display server for remote browser UI
async fn run_display_server(host: String, port: u16) -> Result<()> {
    use protocols::{DisplayServer, SessionManager};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tracing::info;
    use anyhow::Context;

    info!("Starting Thalora Display Server");
    info!("Listening on {}:{}", host, port);

    // Parse socket address
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .context("Failed to parse socket address")?;

    // Create session manager
    let session_manager = Arc::new(SessionManager::new()?);

    // Create display server
    let display_server = DisplayServer::new(session_manager);

    // Start server (this runs indefinitely)
    info!("Display server ready to accept connections");
    display_server.start(addr).await?;

    Ok(())
}