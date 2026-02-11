# Stage 1: Build
FROM rust:1.84-bookworm AS builder

RUN apt-get update && apt-get install -y \
    cmake pkg-config libclang-dev clang nasm perl \
    libavcodec-dev libavformat-dev libavutil-dev \
    libswscale-dev libavdevice-dev libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build

# Stage 2: Runtime (binary only, no source code)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates curl \
    libavcodec59 libavformat59 libavutil57 \
    libswscale6 libavdevice59 libasound2 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m thalora
COPY --from=builder /build/target/debug/thalora /usr/local/bin/thalora

USER thalora
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s CMD curl -f http://localhost:8080/health || exit 1

ENV THALORA_SILENT=1
ENTRYPOINT ["thalora", "server", "--transport", "http", "--host", "0.0.0.0", "--port", "8080"]
