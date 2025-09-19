# VFS (Virtual Filesystem) Integration — README

## Purpose
This document summarizes the work done to sandbox Thalora from the host filesystem by introducing a small in-repo Virtual Filesystem (VFS) crate, the files changed, how to use the VFS, current limitations, and recommended next steps to fully sandbox the embedded Boa engine.

This repository change was implemented to "replace ALL real file-system access with a virtual filesystem (VFS) crate" for the browser runtime while minimizing breakage and keeping APIs compatible.

---

## High-level summary
- Added an in-repo crate `vfs/` that provides `vfs::fs` API. By default it's an in-memory VFS. It has a feature `real_fs` to delegate to `std::fs` if developers need the real filesystem for development/testing.
- Switched Thalora runtime code to use `vfs::fs` instead of `std::fs`: localStorage, ServiceWorker persistence, AI memory
- Audited Boa (the modified JavaScript engine) and identified all places where it accesses the real filesystem. Boa includes build-time macros, CLI tooling and tests that require special handling; I did not convert all Boa filesystem usage yet because that is a larger, workspace-wide change that needs careful gating.

This approach sandboxes the browser runtime (the parts exposed via Thalora's APIs) immediately while leaving Boa's developer tooling and tests intact until we plan the full conversion.

---

## Files added and edited
Added

- `vfs/Cargo.toml`
- `vfs/src/lib.rs` — in-memory VFS implementation with minimal `File`/`OpenOptions` and an opt-in `real_fs` feature

Edited (Thalora runtime files now use `vfs::fs`)

- `Cargo.toml` (root) — added `vfs = { path = "vfs", default-features = false }`
- `src/apis/storage.rs` (replaced `std::fs` -> `vfs::fs`)
- `src/apis/service_worker.rs` (replaced `std::fs` -> `vfs::fs`)
- `src/features/ai_memory.rs` (replaced `std::fs` -> `vfs::fs`)
- `src/protocols/browser_tools/core.rs` (replaced `std::fs` -> `vfs::fs`)

Edited (Boa macros conditional import)

- `engines/boa/core/macros/src/embedded_module_loader.rs` — added a `cfg`-guarded import so the macro can use `vfs::fs` when the `use_vfs` feature is enabled; otherwise it keeps `std::fs`.

Files discovered (Boa workspace) that use the filesystem and will need work before a full engine sandbox:

- `engines/boa/core/parser/src/source/mod.rs` — `File::open` / `BufReader<File>` (parser expects File-like readers)
- `engines/boa/cli/src/main.rs` — `OpenOptions` for CLI history and uses `Source::from_filepath`
- `engines/boa/core/macros/src/embedded_module_loader.rs` — reads directory and files at build-time for embedding
- tools/tests under `engines/boa/tests/` and `engines/boa/tools/` (wpt/tester/test harness, `gen-icu4x-data`, etc.)

---

## How the VFS works
- Default: in-memory VFS (no disk access). This provides a small compatibility layer exposing common APIs used by Thalora code:
  - `create_dir_all(path)` — no-op (directories are implicit in-memory)
  - `read_to_string(path)`, `read(path)` — read in-memory bytes for the path
  - `write(path, bytes)` — write to in-memory storage
  - `remove_file(path)`, `copy(src, dst)`, `exists(path)` — supported in-memory
  - Minimal `File` type implementing `Read`, `Write`, `Seek`
  - Minimal `OpenOptions` (read/write/create/truncate) with `open`
- Feature `real_fs`: if you enable `vfs` crate feature `real_fs`, `vfs::fs` is a thin re-export of `std::fs` and behavior returns to the real filesystem. This is intended for development when you need persistent storage or to run packages/tests that expect the real FS.

Notes:
- `fs::metadata` currently returns an `io::Error` (not implemented in the in-memory VFS). If your code depends on file metadata (size/modified times), the VFS must be extended.
- `read_dir` and `DirEntry` are not implemented in the in-memory VFS (yet). Code that enumerates directories will need them implemented to work under VFS mode.

---

## Quick usage / commands
From the repository root:

- Check the project compiles (fast, typecheck):

```bash
cargo check
```

- Run the Thalora binary (development):

```bash
cargo run
```

By default, the Thalora runtime uses the in-memory VFS (so it will not write persistent files to disk). To enable the real filesystem for development (use with care):

```bash
cargo run --features "vfs/real_fs"
```

(If you only want to run the `thalora` binary within a workspace that has multiple packages or target a specific package, use `-p thalora` or adjust accordingly.)

To run tests (note: Boa tests and many integration tests rely on real filesystem access; some tests may fail under default in-memory VFS):

```bash
cargo test
# or to use the real filesystem
cargo test --features "vfs/real_fs"
```

Quick smoke check to verify no host files were created by the browser (default VFS is in-memory):

```bash
# After running Thalora with default features
find "$HOME/.thalora" -maxdepth 2 -type f || echo "No persistent Thalora files found (expected with in-memory VFS)"
```

---

## Limitations & caveats
- Metadata: `fs::metadata` is a minimal stub that returns an error for now. Many code paths that rely on file size / modified time may break unless metadata is implemented.
- Directory enumeration: `read_dir`/`ReadDir`/`DirEntry` support is not implemented in the in-memory VFS. This affects build-time macros and tools that embed files from the repository (e.g., the `embedded_module_loader` macro and tooling that collects files).
- Proc macros / build-time tooling: `engines/boa` contains proc-macros / build.rs / tools that use the filesystem at build time. Procedural macros have special dependency and compile-time constraints — adding a runtime workspace crate dependency to a proc-macro requires careful handling and may not be practical.
- Path normalization and canonicalization are not implemented. Relative vs absolute path behavior may differ from `std::fs`.
- Persistence: the VFS is in-memory and not persistent across runs unless `real_fs` is enabled.
- Concurrency & scalability: the in-memory VFS uses a single `Mutex<HashMap<PathBuf, Vec<u8>>>`. For heavy concurrency you may want a concurrent map or sharded locks.

---

## Recommended next steps (options)
Pick one depending on your goals.

1) Conservative (recommended first step):
   - Keep the Thalora runtime using `vfs` (in-memory default) — this already sandboxes the browser.
   - Implement missing VFS features needed by Thalora itself (metadata size if AI memory or other code examines file size).
   - Add tests to ensure Thalora does not write to host FS with default features.

2) Full engine sandbox (more invasive):
   - Implement a more complete `vfs` API: `read_dir`, `metadata`, `canonicalize`, `remove_dir_all`, and match more of `std::fs` behavior.
   - Add `vfs` as a dependency to Boa crates that need it, gate with a `use_vfs` feature, and replace `use std::fs` calls with `use vfs::fs` under that feature.
   - Handle proc-macro/build-time crates specially (they might still need real `std::fs` or an additional path to a build-time VFS shim).
   - Run full test-suite and fix issues.

3) Hybrid approach:
   - Use `vfs` for Thalora runtime by default.
   - Leave Boa tools and tests to run with `vfs/real_fs` in development environments. Gradually convert individual Boa crates to `vfs` behind feature flags and validate.

---

## Mapping to the todo list / implementation status
- Audit repository for filesystem usage — Done
- Add VFS abstraction module — Done (`vfs/` added)
- Replace filesystem calls in Thalora core — In-progress / Mostly done (storage, service_worker, ai_memory, browser tools switched to `vfs::fs`)
- Replace filesystem calls in Boa engine — Not done (identified the locations; requires further work and careful feature gating)
- Run build and tests — Partial (ran `cargo check`; many warnings from Boa but no VFS-related errors were introduced in the quick check). Full `cargo test` not run because Boa test harness expects real FS in many places.

---

## Quality gates (what I ran)
- Build / Typecheck: `cargo check` — PASS (compilation performed; warnings present)
- Lint: Not run separately. Rust compiler emitted warnings (predominantly from Boa codebase) — these are existing.
- Unit tests: Did not run the whole test suite here. `vfs` contains a small unit test; run `cargo test -p vfs` to run it.
- Smoke test: suggested manual check to verify no persistent files under `~/.thalora` (see Quick usage).

---

## How I can continue (if you want me to do more)
- Implement `read_dir`, `metadata`, `canonicalize` and `DirEntry` for the in-memory VFS so build-time macros and tools can operate in-memory. I can iteratively implement them and run tests.
- Convert Boa crates to optionally use the VFS behind a `use_vfs` feature (will require adding `vfs` to many `Cargo.toml` files and safe feature gating for proc macros). This is larger and I will perform it step-by-step and run tests.
- Add persistent-backend for VFS if you need sandboxed but persistent storage.

If you want me to proceed with any of these, tell me which option (1/2/3 above) to implement first and I will mark the corresponding todo as `in-progress` and continue.

---

## Contact / Notes
- The VFS crate is intentionally small and conservative. It is designed to be a safe default: sandbox Thalora runtime now and allow opt-in to real FS for development.
- If you want immediate full sandboxing of the Boa engine as well, expect a multi-step conversion and test pass — I can start that once you confirm.


---

Generated on: 2025-09-19

