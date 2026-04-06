# Thalora Desktop GUI

The desktop GUI consists of two .NET 9 processes:

| Process | Project | Role |
|---|---|---|
| **BrowserController** | `gui/BrowserController` | Long-lived HTTP proxy + process manager. Stays alive across GUI crashes. |
| **ThaloraBrowser** | `gui/ThaloraBrowser` | Avalonia window that renders pages. Registers with the controller on startup. |

---

## Quick Start (Development)

Start the controller once. It spawns and manages the GUI for you:

```bash
# From the repo root
dotnet run --project gui/BrowserController -- \
  --port 9222 \
  --gui-path gui/ThaloraBrowser \
  --auto-launch

# Wait ~15s for the GUI to build and register, then:
curl http://localhost:9222/health
# → {"status":"ok","gui":"healthy"}
```

All subsequent work goes through port 9222. The controller stays running even when the GUI crashes or is restarted.

---

## Development Loop

After making C# changes, restart just the GUI — no need to stop the controller:

```bash
# Rebuild
dotnet build gui/ThaloraBrowser

# Kill old GUI and spawn fresh one (one call)
curl http://localhost:9222/restart

# Wait ~15s for the new GUI to register
sleep 15 && curl http://localhost:9222/health
```

To navigate and take a screenshot:

```bash
curl -X POST http://localhost:9222/navigate \
     -H "Content-Type: application/json" \
     -d '{"url":"https://www.google.com","timeout_ms":30000}'

curl "http://localhost:9222/screenshot?delay=1000" -o /tmp/capture.png
open /tmp/capture.png
```

---

## Controller CLI Flags

```
dotnet run --project gui/BrowserController -- [options]

  --port <N>          Port to listen on (default: 9290)
  --gui-path <path>   Path to ThaloraBrowser .csproj or project directory
                      Required for /launch, /restart, and --auto-launch
  --auto-launch       Spawn the GUI immediately on startup and re-spawn
                      automatically after crashes
  --url <url>         Initial URL to open (passed to GUI on auto-launch)
```

## GUI CLI Flags

```
dotnet run --project gui/ThaloraBrowser -- [options]

  --control-port <N>  Port of a running BrowserController to register with
  --url <url>         Initial URL to navigate to on startup
  --width <N>         Initial window width (default: 1280)
  --height <N>        Initial window height (default: 800)
```

---

## Controller Endpoints

All requests go to `http://localhost:{port}/`.

### Management (controller-handled)

| Endpoint | Method | Description |
|---|---|---|
| `/health` | GET | `{"status":"ok","gui":"healthy"}` — overall health |
| `/status` | GET | Detailed state: GUI PID, port, registration count, log path |
| `/register` | POST | Called automatically by ThaloraBrowser on startup |
| `/unregister` | POST | Called automatically by ThaloraBrowser on clean shutdown |
| `/launch` | GET / POST | Spawn a new GUI (no-op if one already running). POST body: `{"url":"..."}` |
| `/restart` | GET / POST | Kill current GUI + spawn fresh one. POST body: `{"url":"..."}` |
| `/shutdown` | POST | Stop the controller process |

### Browser control (proxied to GUI)

| Endpoint | Method | Description |
|---|---|---|
| `/navigate` | POST | Body: `{"url":"...","timeout_ms":30000}` — navigate and wait for load |
| `/screenshot` | GET | Query: `?delay=ms` — capture viewport as PNG |
| `/state` | GET | URL, title, is_loading, html_length |
| `/html` | GET | Raw HTML of the current page |
| `/layout` | GET | Full styled-tree JSON (CSS-resolved element tree from Rust) |
| `/elements` | GET | List all interactive elements (links, hover targets) |
| `/find-element` | POST | Body: `{"tag":"a","text":"Sign in"}` — search by criteria |
| `/click-element` | POST | Body: `{"element_id":"e42"}` |
| `/hover-element` | POST | Body: `{"element_id":"e42"}` |
| `/unhover-element` | POST | Body: `{"element_id":"e42"}` |
| `/scroll` | POST | Body: `{"y":500}` |
| `/content-height` | GET | Content height, viewport dimensions, scroll position |
| `/wait-for-images` | POST | Body: `{"wait_ms":3000}` — wait for async image loads |
| `/type` | POST | Body: `{"text":"hello","target":"addressbar"}` |

---

## Crash Debugging

When the GUI is launched via the controller (with `--auto-launch` or `/launch`), its stdout and stderr are written to a timestamped log file:

```bash
# Find the log path
curl -s http://localhost:9222/status | python3 -c \
  "import sys,json; print(json.load(sys.stdin)['last_log'])"

# Read it
cat /tmp/thalora-gui-20260406-191942.log | tail -80
```

The log contains the full .NET runtime output including unhandled exception stack traces, Boa panic messages, and `[WebContentControl]` / `[TIMING]` debug lines.

---

## Architecture

```
External Client / Claude Code
         │
         │  All requests → port 9222
         ▼
 ┌──────────────────────────┐
 │    BrowserController     │  Long-lived process
 │    HttpProxyServer       │  Handles /health, /status, /launch, /restart
 │    GuiProcessManager     │  Tracks GUI PID, watches for crashes
 └──────────┬───────────────┘
            │ spawns + monitors
            │ PID watch via WaitForExitAsync (<1s crash detection)
            ▼
 ┌──────────────────────────┐
 │    ThaloraBrowser        │  Avalonia window process
 │    BrowserControlServer  │  Internal HTTP server (ephemeral port)
 │    ControlTreeBuilder    │  Rust JSON → Avalonia control tree
 │    ThaloraBrowserEngine  │  P/Invoke wrapper → libthalora.dylib
 └──────────────────────────┘
            │
            │  FFI (SemaphoreSlim-serialized — engine is not thread-safe)
            ▼
 ┌──────────────────────────┐
 │    libthalora            │  Rust: HTML parse, CSS resolve, Boa JS engine
 └──────────────────────────┘
```

### Key files

| File | Purpose |
|---|---|
| `gui/BrowserController/GuiProcessManager.cs` | PID watching, crash detection, GUI spawning |
| `gui/BrowserController/HttpProxyServer.cs` | HTTP server, proxy logic, /launch /restart |
| `gui/ThaloraBrowser/Services/ThaloraBrowserEngine.cs` | FFI wrapper with engine lock |
| `gui/ThaloraBrowser/Controls/WebContentControl.cs` | Render pipeline orchestration |
| `gui/ThaloraBrowser/Rendering/ControlTreeBuilder.cs` | Avalonia control tree builder |
| `gui/ThaloraBrowser/Rendering/StyleParser.cs` | CSS string → Avalonia type converters |
| `src/engine/renderer/page_layout.rs` | Rust: DOM → StyledElement tree |
| `src/engine/renderer/styled_tree.rs` | Rust: StyledElement / ResolvedStyles structs |

---

## Visual Testing

```bash
# Screenshot comparison with a reference image
cargo xtask gui-compare https://www.google.com --ref /tmp/chrome-google.png

# Just take a screenshot
cargo xtask gui-screenshot https://www.google.com --out /tmp/thalora.png
```

Reference images for key pages live in `gui/ThaloraBrowser/visual-tests/`.
