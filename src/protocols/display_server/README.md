# Display Server Module Structure

This directory contains the modular implementation of Thalora's Display Server Protocol.

## Module Organization

### `mod.rs` (79 lines)
**Public API & Server Coordination**
- Main `DisplayServer` struct
- Public exports for external use
- Module declarations and re-exports

### `messages.rs` (206 lines)
**Message Types & Serialization**
- `DisplayMessage` enum - Messages sent from Thalora to clients
- `DisplayCommand` enum - Commands sent from clients to Thalora
- `ScreencastFrameMetadata` - CDP screencast metadata
- `current_timestamp()` utility function
- Serde serialization/deserialization
- Message validation tests

### `sessions.rs` (74 lines)
**Session Management**
- `DisplayClient` struct - Client connection tracking
- `ClientRegistry` - Client storage and management
- Client registration/removal
- Session-to-client mapping
- Message routing to specific clients

### `server.rs` (247 lines)
**WebSocket Server Core**
- `WebSocketServer` struct - Main server implementation
- TCP listener and connection acceptance
- WebSocket upgrade handling
- Connection lifecycle management
- Message routing (client → broadcast → individual)
- Session cleanup on disconnect

### `handlers.rs` (460 lines)
**Message Handlers & Processing**
- `CommandHandler` - Processes client commands
- `processing` module - HTML content processing utilities:
  - `strip_security_meta_tags()` - Removes X-Frame-Options, CSP headers
  - `rewrite_image_urls()` - Proxies images through local server
  - `inject_proxy_script()` - Intercepts fetch/XHR, suppresses History API errors
  - `process_html()` - Complete HTML processing pipeline
- Command handlers:
  - Navigation (navigate, back, forward, reload)
  - Interaction (click, type, execute script)
  - Screencast (start, stop, frame acknowledgment)
  - Content refresh

## Architecture

```
User Browser ←→ WebSocket ←→ Display Server ←→ Browser Session
                              │
                              ├─ messages.rs (Protocol)
                              ├─ server.rs (Transport)
                              ├─ sessions.rs (State)
                              ├─ handlers.rs (Logic)
                              └─ mod.rs (API)
```

## Key Features

1. **WebSocket Communication**: Real-time bidirectional messaging
2. **Session Management**: Maps display clients to browser sessions
3. **HTML Processing**: Strips security headers, proxies resources
4. **Broadcast Support**: Send messages to all clients or specific ones
5. **CDP Screencast**: Efficient frame streaming (planned)

## Migration Notes

- Original file: `display_server.rs` (845 lines)
- New modular structure: 5 files (1,066 lines total)
- All functionality preserved
- Improved separation of concerns
- Better testability and maintainability

## Usage

```rust
use thalora::protocols::{DisplayServer, DisplayMessage, DisplayCommand};

// Create server
let server = DisplayServer::new(session_manager);

// Start WebSocket server
server.start("127.0.0.1:9222".parse()?).await?;

// Broadcast to all clients
server.broadcast(DisplayMessage::Ping { timestamp: now() })?;

// Send to specific client
server.send_to_client(&client_id, msg)?;
```

## Testing

All original tests preserved in `messages.rs`:
- Message serialization/deserialization
- Command parsing
- JSON format validation

## Future Enhancements

- [ ] Complete CDP screencast integration
- [ ] Implement back/forward navigation
- [ ] Add reload command
- [ ] Frame acknowledgment handling
- [ ] Enhanced error reporting
