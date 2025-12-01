#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use anyhow::Result;
use clap::{Parser, Subcommand};

// Core modules organized by functionality
pub mod engine;
pub mod apis;
pub mod features;
pub mod protocols;

// GUI module for graphical browser interface
#[cfg(feature = "gui")]
pub mod gui;

use protocols::mcp_server::McpServer;
use engine::{EngineType, EngineFactory, EngineConfig};

#[derive(Parser)]
#[command(name = "thalora")]
#[command(about = "Pure Rust headless browser for AI models with MCP integration")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Use V8 JavaScript engine instead of the default Boa engine
    #[arg(long = "use-v8-engine", help = "Use V8 JavaScript engine for execution")]
    use_v8_engine: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as MCP server (default mode)
    Server {
        /// MCP mode: 'minimal' for basic scraping (default), 'full' for all features
        #[arg(long, default_value = "minimal")]
        mcp_mode: String,
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
    let cli = Cli::parse();

    // Determine which engine to use based on CLI flags or default
    let use_v8 = if cli.use_v8_engine {
        true  // Override to use V8
    } else {
        // No flags specified, use the default from EngineFactory
        EngineFactory::default_engine() == EngineType::V8
    };

    // Create engine configuration
    let engine_config = EngineConfig::new(use_v8)?;
    
    // Log the selected engine
    if std::env::var("THALORA_SILENT").is_err() {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();
        
        tracing::info!("Using {} JavaScript engine", engine_config.engine_type);
        
        // Display available engines for info
        let available = EngineFactory::available_engines();
        let available_names: Vec<String> = available.iter().map(|e| e.to_string()).collect();
        tracing::debug!("Available engines: {}", available_names.join(", "));
    } else {
        // Still configure tracing but silent
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();
    }

    match cli.command {
        Some(Commands::Session { session_id, socket_path, persistent }) => {
            // Run as browser session process
            run_browser_session(session_id, socket_path, persistent, engine_config).await
        }
        Some(Commands::Browser { url, width, height, fullscreen, debug }) => {
            // Run as graphical web browser
            run_graphical_browser(url, width, height, fullscreen, debug, engine_config).await
        }
        Some(Commands::DisplayServer { port, host }) => {
            // Run as display server
            run_display_server(host, port).await
        }
        Some(Commands::Server { mcp_mode }) => {
            // Run as MCP server with specified mode
            // SAFETY: This is called at program startup before any threads are spawned
            unsafe { std::env::set_var("THALORA_MCP_MODE", &mcp_mode) };
            eprintln!("🚀 Starting Thalora MCP Server in '{}' mode", mcp_mode);

            let mut server = McpServer::new_with_engine(engine_config);
            server.run().await
        }
        None => {
            // Run as MCP server (default mode)
            // SAFETY: This is called at program startup before any threads are spawned
            unsafe { std::env::set_var("THALORA_MCP_MODE", "minimal") };
            eprintln!("🚀 Starting Thalora MCP Server in 'minimal' mode");

            let mut server = McpServer::new_with_engine(engine_config);
            server.run().await
        }
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