# Test-Fix Agent Instructions — Thalora Headless Browser

You are an autonomous coding agent. Your job is to run tests, find failures, fix the underlying bugs, and retest until all tests pass.

**Scope: Rust headless browser only. Do NOT touch .NET GUI code.**

## Setup

- Project root: `/home/nightness/dev/thalora-web-browser`
- Test runner: `./scripts/test-fix-loop/run_tests.sh`
- Report: `./scripts/test-fix-loop/test-report.json`
- Known failures: `./scripts/test-fix-loop/known-failures.json`
- Progress log: `./scripts/test-fix-loop/progress.txt`

## Workflow (per iteration)

1. **Check progress.txt** for context from previous iterations (especially "Codebase Patterns" and "Needs Human Review" sections)

2. **Run tests:**
   - First iteration or no report exists: `./scripts/test-fix-loop/run_tests.sh --all`
   - After a fix: `./scripts/test-fix-loop/run_tests.sh --retest-failures`
   - For targeted runs: `./scripts/test-fix-loop/run_tests.sh --test <binary>`

3. **Read report** (`test-report.json`) and **filter out**:
   - Tests listed in `known-failures.json` (flaky, network-dependent, release-only)
   - Tests already marked "needs human review" in `progress.txt`

4. **Prioritize** remaining failures:
   - Build/compilation errors (fix these FIRST — they block everything)
   - Panics / crashes
   - Assertion failures
   - Timeouts

5. **Analyze** the top-priority failure:
   - Read the failing test file to understand what it expects
   - Read the source code being tested
   - Understand the error message and stack trace

6. **Implement ONE fix** per iteration:
   - Fix the source code, not the test (unless the test is genuinely wrong)
   - Keep changes minimal and focused
   - If unsure, add a comment explaining your reasoning

7. **Verify the fix:**
   - Run `./scripts/test-fix-loop/run_tests.sh --retest-failures`
   - If the fix passes: commit with `fix: resolve <test_name> - <brief description>`
   - If the fix breaks OTHER tests: `git checkout -- .` to revert and try a different approach
   - Max 3 attempts per failure before marking "needs human review"

8. **Update progress.txt** (append, never replace):
   ```
   ## [Date/Time] - <test_name>
   - Status: FIXED / NEEDS HUMAN REVIEW / SKIPPED
   - What was done
   - Files changed
   - **Learnings:** patterns or gotchas discovered
   ---
   ```

9. **Check completion:**
   - If all non-known failures are fixed: output `<promise>COMPLETE</promise>`
   - If failures remain: end normally (next iteration picks up next failure)

## Source-to-Test Mapping

Use this to run targeted tests after fixing specific source files:

| Source change | Test binaries to rerun |
|---|---|
| `src/protocols/` | `mcp_tests`, `security_tests` |
| `src/engine/browser/` | `api_tests`, `compatibility_tests`, `integration_tests` |
| `src/apis/` | `api_tests`, `chrome_features_tests` |
| `src/engine/renderer/` | `chrome_features_tests`, `compatibility_tests` |
| `src/features/ai_memory/` | `ai_memory_tests` |
| `src/engine/security/` | `security_tests`, `security_bypass_tests`, `security_audit_tests` |
| `engines/boa/` or `engines/thalora-browser-apis/` | ALL tests (engine changes are broad) |

## Test Binaries (22 total)

`api_tests`, `api_implementation_tests`, `ai_memory_tests`, `chrome_features_tests`, `compatibility_tests`, `complete_networking_test`, `debug_worker_context`, `debug_worker_context_steps`, `engine_comparison_test`, `engine_switching_test`, `error_stack_test`, `integration_tests`, `mcp_tests`, `minimal_chrome_test`, `native_websocket_test`, `quick_networking_test`, `sandbox_fs_test`, `search_parser_tests`, `security_tests`, `security_audit_tests`, `security_bypass_tests`, `vfs_session_integration`

## Critical Rules

- **NEVER** add `#[ignore]` to tests to "fix" them
- **NEVER** weaken assertions (e.g., changing `assert_eq!` to a weaker check)
- **NEVER** add timeouts to `cargo build`, `cargo check`, or `cargo test` commands (wasmtime compilation is slow)
- **NEVER** delete or disable tests
- **NEVER** touch GUI/C# code (gui/ directory)
- If a test is genuinely testing wrong behavior, explain WHY in the commit message and fix the test
- If a failure is flaky (passes on rerun), add it to `known-failures.json` instead of "fixing" it
- When fixing a test that uses `McpTestHarness`, the harness spawns a subprocess — errors may be in the server code, not the test

## Architecture Quick Reference

- **Engine trait**: `src/engine/engine_trait.rs` — `ThaloraBrowserEngine` trait
- **Browser core**: `src/engine/browser/core.rs` — `HeadlessWebBrowser`
- **Navigation**: `src/engine/browser/navigation/` — core, javascript, forms, cookies, state
- **MCP server**: `src/protocols/mcp_server/` — tools in `tools/definitions/`, routing in `routing.rs`
- **APIs**: `src/apis/` — DOM, geolocation, media, polyfills, etc.
- **Renderer**: `src/engine/renderer/` — CSS, layout, styled tree
- **Security**: `src/engine/security/` — SSRF prevention, origin policies
- **AI Memory**: `src/features/ai_memory/` — encrypted persistent storage
- **Test harness**: `tests/protocols/mcp_harness.rs` — `McpTestHarness`
- **Boa JS engine**: `engines/boa/` (custom fork)
- **Browser APIs**: `engines/thalora-browser-apis/` (modular JS API implementations)

## Environment Variables

- `THALORA_MCP_MODE="full"` — enable all MCP tools
- `THALORA_ENABLE_AI_MEMORY="true"` — enable AI memory feature
- `THALORA_ENABLE_CDP="true"` — enable Chrome DevTools Protocol
- `THALORA_ENABLE_SCRAPING="true"` — enable web scraping
- `THALORA_ENABLE_SEARCH="true"` — enable web search
- `THALORA_ENABLE_SESSIONS="true"` — enable session management
- `THALORA_SILENT="true"` — suppress debug output
- `THALORA_TEST_ENGINE="boa"` — engine selection

## Wasmtime Build Notes

- Wasmtime 21.0 compilation is SLOW on first build (5-10+ minutes)
- Incremental builds after small fixes are fast (seconds)
- `Instance` is `Copy`, `exports()` requires `&mut Store`, `get_func()` requires `&mut Store`
- Use `set_epoch_deadline()` + `epoch_deadline_trap()` for timeout

## Stop Condition

After fixing a failure, check remaining actionable failures (excluding known-failures.json entries).

- If ALL actionable failures are fixed: `<promise>COMPLETE</promise>`
- If failures remain: end normally (next iteration continues)
