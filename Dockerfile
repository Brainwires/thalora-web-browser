# Stage 1: Build with desktop features
# The toolchain the Brainwires deno fork pins (its page runtime, which
# thalora links, is built with it).
FROM rust:1.99.1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake pkg-config clang nasm perl python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
# The `v8` crate links a prebuilt V8 from Brainwires/rusty_v8. The script
# downloads it, checks it against scripts/rusty_v8.sha256 and points the
# build at it; without it the build refuses to download anything
# (.cargo/config.toml).
RUN eval "$(python3 scripts/fetch_rusty_v8.py --print-env)" \
    && cargo build --release \
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

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s CMD curl -sf http://localhost:8080/health || exit 1

ENV THALORA_SILENT=1
ENTRYPOINT ["thalora", "server", "--transport", "http", "--host", "0.0.0.0", "--port", "8080"]
