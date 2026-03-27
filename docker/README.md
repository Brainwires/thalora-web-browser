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

## Claude Desktop Integration (HTTP Transport)

```json
{
  "mcpServers": {
    "thalora": {
      "url": "http://localhost:8080",
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

## Health Check

The container exposes a health endpoint at `GET http://localhost:8080/health`. Docker's built-in `HEALTHCHECK` polls this every 30 seconds.
