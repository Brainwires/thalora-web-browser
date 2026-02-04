# Cross-Compilation Guide

This guide covers building Thalora for multiple platforms using the `brainwires-deploy` tool (bd) or manually with `cross`.

## Quick Start (Recommended)

The easiest way to cross-compile is using the `bd` (brainwires-deploy) tool:

```bash
# On macOS - builds universal binary + Linux + Windows
bd build thalora --cross

# On Linux - builds Linux + Windows
bd build thalora --cross

# Build for specific target
bd build thalora -t universal-apple-darwin

# Build and upload to Supabase Storage
bd build thalora --cross --upload --version v1.0.0
```

## Supported Targets

| Target | Platform | Build From |
|--------|----------|------------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 | Linux, macOS (via cross) |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | Linux, macOS (via cross) |
| `x86_64-pc-windows-gnu` | Windows x86_64 | Linux, macOS (via cross) |
| `x86_64-apple-darwin` | macOS Intel | macOS only |
| `aarch64-apple-darwin` | macOS Apple Silicon | macOS only |
| `universal-apple-darwin` | macOS Universal | macOS only |

## Prerequisites

### On macOS

```bash
# 1. Install Xcode Command Line Tools
xcode-select --install

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Add macOS targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 4. (Optional) Install cross for Linux/Windows targets
cargo install cross

# 5. (Optional) Install Docker for cross-compilation
# Download from https://www.docker.com/products/docker-desktop
```

### On Linux

```bash
# Install cross
cargo install cross

# Install Docker
sudo apt install docker.io
sudo systemctl start docker
sudo usermod -aG docker $USER
# Log out and back in for group changes
```

## macOS Universal Binary

On macOS, the `--cross` flag automatically builds a universal binary that works on both Intel and Apple Silicon Macs:

```bash
bd build thalora --cross
```

This:
1. Builds for `x86_64-apple-darwin` (Intel)
2. Builds for `aarch64-apple-darwin` (Apple Silicon)
3. Combines them with `lipo` into a universal binary

Output: `./exports/thalora-universal-apple-darwin`

## Manual Cross-Compilation

### Using `cross` (Linux/Windows targets)

```bash
# From the thalora-web-browser directory
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

### Using native cargo (macOS targets)

```bash
# Build for Intel Mac
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
cargo build --release --target aarch64-apple-darwin

# Create universal binary manually
lipo -create \
    target/x86_64-apple-darwin/release/thalora \
    target/aarch64-apple-darwin/release/thalora \
    -output target/thalora-universal
```

### Build Output Location

Binaries are placed in:
```
target/<target>/release/thalora
```

For example:
```
target/x86_64-unknown-linux-gnu/release/thalora
target/x86_64-pc-windows-gnu/release/thalora.exe
target/x86_64-apple-darwin/release/thalora
target/aarch64-apple-darwin/release/thalora
target/universal-apple-darwin/release/thalora  # Universal binary
```

## brainwires-deploy Commands

The `bd` (brainwires-deploy) tool provides comprehensive cross-compilation support:

```bash
# Build for all targets (platform-aware)
bd build thalora --cross

# Build specific targets
bd build thalora -t x86_64-pc-windows-gnu
bd build thalora -t universal-apple-darwin
bd build thalora -t x86_64-unknown-linux-gnu -t aarch64-unknown-linux-gnu

# Build and upload to Supabase Storage
bd build thalora --cross --upload --version v1.0.0
bd build thalora --cross --upload --bucket my-releases

# Check prerequisites
bd build thalora --cross  # Automatically checks prerequisites
```

### Available Targets

| Target String | Description |
|---------------|-------------|
| `x86_64-unknown-linux-gnu` | Linux x86_64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `x86_64-pc-windows-gnu` | Windows x86_64 |
| `x86_64-apple-darwin` | macOS Intel |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `universal-apple-darwin` | macOS Universal (Intel + ARM) |
| `universal` or `macos-universal` | Aliases for universal |

## Configuration

### Cross.toml (Optional)

Create a `Cross.toml` in the project root for custom configuration:

```toml
[build.env]
passthrough = [
    "RUST_BACKTRACE",
    "RUST_LOG",
]

[target.x86_64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main"

[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[target.x86_64-pc-windows-gnu]
image = "ghcr.io/cross-rs/x86_64-pc-windows-gnu:main"
```

## Troubleshooting

### Docker Permission Denied

```bash
sudo usermod -aG docker $USER
# Log out and back in
```

### Missing Target

```bash
rustup target add <target>
```

### Build Timeout

Cross-compilation can be slow. Increase Docker memory limits if builds fail:

```bash
# In Docker Desktop: Settings → Resources → Memory
# Or via daemon.json for Docker Engine
```

### OpenSSL/Native Dependencies

Some targets may require system libraries. Cross handles most cases, but you may need custom Docker images for complex dependencies.

## CI/CD Integration

For automated cross-compilation, consider GitHub Actions:

```yaml
name: Cross-compile

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-pc-windows-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install cross (Linux)
        if: runner.os == 'Linux'
        run: cargo install cross

      - name: Build (Linux with cross)
        if: runner.os == 'Linux'
        run: cross build --release --target ${{ matrix.target }}

      - name: Build (macOS native)
        if: runner.os == 'macOS'
        run: |
          rustup target add ${{ matrix.target }}
          cargo build --release --target ${{ matrix.target }}

      - uses: actions/upload-artifact@v4
        with:
          name: thalora-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/thalora*
```

## Distribution Packaging

After building, package binaries for distribution:

```bash
# Create distribution directory
mkdir -p dist

# Linux
cp target/x86_64-unknown-linux-gnu/release/thalora dist/thalora-linux-x86_64
cp target/aarch64-unknown-linux-gnu/release/thalora dist/thalora-linux-arm64

# Windows
cp target/x86_64-pc-windows-gnu/release/thalora.exe dist/thalora-windows-x86_64.exe

# Create checksums
cd dist
sha256sum * > checksums.txt
```

## Performance Notes

- **Build time**: Cross-compilation is slower than native builds due to Docker overhead
- **Binary size**: Release builds with LTO are ~10-15MB
- **Runtime**: No performance difference from native builds
