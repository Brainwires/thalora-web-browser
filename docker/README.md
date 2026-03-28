# Docker Deployment

Thalora can be run as a Docker container for easy deployment in CI pipelines, servers, or containerized environments. The container runs an HTTP MCP server on port 8080.

## Quick Start

```bash
# Build the image
docker build -t thalora .

# Run the HTTP MCP server
docker run --rm -p 8080:8080 thalora

# Run with environment variables
docker run --rm -p 8080:8080 \
  -e RUST_LOG=info \
  -e THALORA_MASTER_PASSWORD="your-strong-password-minimum-32-characters!" \
  thalora
```

## Transport Modes

Thalora supports two MCP transport modes:

### HTTP Transport (default in Docker)

Each connecting AI agent gets its own isolated session running on a dedicated OS thread with its own JavaScript engine instance. Multiple agents can connect to a single container simultaneously without interfering with each other.

```bash
# HTTP transport is the default in Docker
docker run --rm -p 8080:8080 thalora

# Explicit flags (also usable outside Docker)
./thalora server --transport http --host 0.0.0.0 --port 8080
```

The HTTP transport implements the MCP Streamable HTTP protocol (JSON responses). Session identity is tracked via the `Mcp-Session-Id` response header — clients must echo this header on subsequent requests within the same session.

### stdio Transport (local use)

```bash
# Pipe JSON-RPC over stdin/stdout — one session per process
./thalora server
./thalora server --transport stdio   # explicit
```

## Claude Desktop Integration (HTTP Transport)

```json
{
  "mcpServers": {
    "thalora": {
      "url": "http://localhost:8080/mcp",
      "env": {
        "THALORA_MASTER_PASSWORD": "your-strong-password-minimum-32-characters!"
      }
    }
  }
}
```

## Persistent AI Memory

To persist the AI memory heap across container restarts, mount a volume:

```bash
docker run --rm -p 8080:8080 \
  -v thalora-memory:/home/thalora/.cache/thalora \
  -e THALORA_MASTER_PASSWORD="your-strong-password-minimum-32-characters!" \
  thalora
```

## Build Details

The Dockerfile uses a two-stage build:

1. **Builder** — compiles the release binary using `rust:1.91.0-bookworm`
2. **Runtime** — minimal `debian:bookworm-slim` with `ca-certificates` and `curl`

The binary is stripped to minimize image size. The server runs as a non-root `thalora` user.

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `THALORA_MASTER_PASSWORD` | Yes (for credentials) | Master password for AES-256-GCM encrypted credential storage (32+ chars) |
| `RUST_LOG` | No | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `THALORA_SILENT` | No | Set to `1` to suppress startup banner |

## Server Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--transport` | `stdio` | Transport mode: `stdio` or `http` |
| `--host` | `0.0.0.0` | Bind address (HTTP transport only) |
| `--port` | `8080` | Bind port (HTTP transport only) |

## Health Check

The container exposes a health endpoint at `GET http://localhost:8080/health`. Docker's built-in `HEALTHCHECK` polls this every 30 seconds.

```bash
curl http://localhost:8080/health
# {"status":"ok"}
```
