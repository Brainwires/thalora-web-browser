# Stage 1: Build with lean mcp-server feature (no FFmpeg/audio/video/WebRTC/wasmtime)
FROM rust:1.91.0-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake pkg-config clang nasm perl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build --release --no-default-features --features mcp-server \
    && strip target/release/thalora

# Stage 2: Minimal runtime (binary only, no source code)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m thalora
COPY --from=builder /build/target/release/thalora /usr/local/bin/thalora

USER thalora
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s CMD curl -f http://localhost:8080/health || exit 1

ENV THALORA_SILENT=1
ENTRYPOINT ["thalora", "server", "--transport", "http", "--host", "0.0.0.0", "--port", "8080"]
