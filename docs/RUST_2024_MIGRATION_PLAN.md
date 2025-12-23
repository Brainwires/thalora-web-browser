# Thalora Headless Web Browser - Rust 2024 Migration Plan

## Overview

This plan outlines the steps to migrate Thalora Headless Web Browser from Rust 2021 to Rust 2024 edition. The migration affects two working copies and must be coordinated:

- **Primary**: `/home/nightness/brainwires-cli/crates/thalora-web-browser` (submodule)
- **Dependent**: `/home/nightness/dev/brainwires-studio/rust/thalora-web-browser` (mirror copy)

## Current State

| Component | Location | Current Edition | Target Edition |
|-----------|----------|-----------------|----------------|
| Thalora (main) | `Cargo.toml` | 2021 | 2024 |
| thalora-browser-apis | `engines/thalora-browser-apis/Cargo.toml` | 2021 | 2024 |
| thalora-constants | `shared/thalora-constants/Cargo.toml` | 2021 | 2024 |
| vfs | `vfs/Cargo.toml` | 2021 | 2024 |
| Boa Engine (submodule) | `engines/boa/Cargo.toml` | **2024** | 2024 ✓ |

**Note**: The Boa JavaScript engine submodule is already on Rust 2024, which is a significant advantage.

## Pre-Migration Requirements

- **Rust Toolchain**: Version 1.85+ required (Rust 2024 stabilization)
- **Boa Submodule**: Requires Rust 1.88.0 minimum (as specified in its Cargo.toml)

## Migration Steps

### Phase 1: Preparation

1. **Ensure Rust toolchain is up to date**
   ```bash
   rustup update stable
   rustup default stable
   rustc --version  # Should be 1.85+ for Rust 2024
   ```

2. **Verify Boa submodule is initialized**
   ```bash
   cd crates/thalora-web-browser
   git submodule update --init --recursive
   ```

3. **Create a migration branch**
   ```bash
   git checkout -b feature/rust-2024-migration
   ```

4. **Run initial build to establish baseline**
   ```bash
   cargo build --all-features
   cargo test
   ```

### Phase 2: Edition Updates

Update the `edition` field in all Thalora Cargo.toml files:

1. **Main Thalora crate** (`Cargo.toml`)
   - Change `edition = "2021"` to `edition = "2024"`

2. **thalora-browser-apis** (`engines/thalora-browser-apis/Cargo.toml`)
   - Change `edition = "2021"` to `edition = "2024"`

3. **thalora-constants** (`shared/thalora-constants/Cargo.toml`)
   - Change `edition = "2021"` to `edition = "2024"`

4. **vfs** (`vfs/Cargo.toml`)
   - Change `edition = "2021"` to `edition = "2024"`

### Phase 3: Rust 2024 Language Changes

Address breaking changes introduced in Rust 2024:

#### 3.1 Unsafe Blocks and Attributes
- Review all `unsafe` blocks - Rust 2024 may require `unsafe(...)` attribute syntax
- Check for any unsafe trait implementations

#### 3.2 Lifetime Elision Changes
- Review functions with complex lifetime patterns
- Verify lifetime inference still works as expected

#### 3.3 Pattern Matching Changes
- Check for exclusive range patterns (`..`) which have new semantics
- Verify match exhaustiveness

#### 3.4 Closure Captures
- Review closures capturing references - capture rules changed slightly

#### 3.5 Async/Generator Syntax
- If using any experimental features, verify compatibility

### Phase 4: Compilation and Testing

1. **Initial compile check**
   ```bash
   cargo check --all-features
   ```

2. **Fix any compilation errors**
   - Document each fix for reproducibility
   - Focus on edition-specific changes

3. **Run full test suite**
   ```bash
   cargo test --all-features
   ```

4. **Build in release mode**
   ```bash
   cargo build --release --all-features
   ```

5. **Test WASM build** (if applicable)
   ```bash
   cargo build --target wasm32-unknown-unknown --features wasm
   ```

### Phase 5: Dependency Compatibility

Verify all dependencies support Rust 2024:

**Critical Dependencies to Check**:
- `wasmtime` (21.0)
- `webrtc` (0.11)
- `ffmpeg-next` (7.0)
- `lightningcss` (1.0.0-alpha.67)
- `wgpu` (0.20)
- `egui` (0.28)

**Action**: Run `cargo update` to get latest compatible versions, then retest.

### Phase 6: Sync to brainwires-studio

After successful migration in brainwires-cli:

1. **Copy updated files to brainwires-studio**
   ```bash
   # From the thalora-web-browser directory
   rsync -av --exclude='.git' --exclude='target' \
     /home/nightness/brainwires-cli/crates/thalora-web-browser/ \
     /home/nightness/dev/brainwires-studio/rust/thalora-web-browser/
   ```

2. **Verify brainwires-studio builds**
   ```bash
   cd /home/nightness/dev/brainwires-studio/rust
   cargo build
   ```

3. **Note**: brainwires-studio's `computational-engine` is already on edition 2024

### Phase 7: Final Validation

1. **Full integration test in brainwires-cli**
   ```bash
   cd /home/nightness/brainwires-cli
   cargo build --all-features
   cargo test
   ```

2. **Full integration test in brainwires-studio**
   ```bash
   cd /home/nightness/dev/brainwires-studio
   # Run whatever build/test process is standard
   ```

3. **Commit and push changes**
   ```bash
   git add -A
   git commit -m "chore: migrate Thalora to Rust 2024 edition"
   git push origin feature/rust-2024-migration
   ```

## Rollback Plan

If migration fails:

```bash
git checkout main
git branch -D feature/rust-2024-migration
```

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | `edition = "2021"` → `edition = "2024"` |
| `engines/thalora-browser-apis/Cargo.toml` | `edition = "2021"` → `edition = "2024"` |
| `shared/thalora-constants/Cargo.toml` | `edition = "2021"` → `edition = "2024"` |
| `vfs/Cargo.toml` | `edition = "2021"` → `edition = "2024"` |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Dependency incompatibility | Low | High | Check each dep before migration |
| Unsafe code changes | Low | Medium | Boa already 2024, likely compatible |
| Build time increase | Low | Low | Expected with new edition |
| WASM build breaks | Medium | Medium | Test separately with `--features wasm` |

## Success Criteria

- [ ] All Cargo.toml files updated to edition 2024
- [ ] `cargo check --all-features` passes
- [ ] `cargo test --all-features` passes
- [ ] `cargo build --release --all-features` succeeds
- [ ] WASM build works (if applicable)
- [ ] brainwires-studio builds with updated Thalora
- [ ] No regressions in functionality

## Notes

- The Boa engine submodule is already on Rust 2024, which should make this migration smoother
- brainwires-cli depends on Thalora via git (not path), so the submodule must be pushed after migration
- brainwires-studio has a mirror copy that needs manual sync
