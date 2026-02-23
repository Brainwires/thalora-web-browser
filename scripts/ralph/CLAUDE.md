# Ralph Agent Instructions — Thalora Browser Visual Rendering

You are an autonomous coding agent working on the Thalora web browser project. Your goal is to make the browser's rendering of `https://blog.cloudflare.com/markdown-for-agents/` match a Chrome reference PDF.

## Architecture

- **Rust** (`src/engine/renderer/`): CSS parsing (css.rs), layout tree building (page_layout.rs), taffy layout engine (layout.rs), text measurement (text_measure.rs)
- **C#** (`gui/ThaloraBrowser/Rendering/`): JSON → LayoutBox tree (HtmlRenderer.cs), painting to Avalonia DrawingContext (PaintContext.cs), data models (LayoutBox.cs)
- **Data flow**: HTML → Rust CSS+layout → JSON (ElementLayout) → C# HtmlRenderer → PaintContext painting
- **Fonts**: Bundled in `src/gui/fonts/` — NotoSans, NotoSerif, FiraMono

## Your Task

1. Read `prd.json` (in this directory)
2. Read `progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run quality checks (see below)
7. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
8. Update `prd.json` to set `passes: true` for the completed story
9. Append your progress to `progress.txt`

## Quality Checks

```bash
# Rust build (NO TIMEOUT — wasmtime compilation is slow)
cargo build

# C# build
dotnet build gui/ThaloraBrowser

# Rust tests
cargo test
```

ALL THREE must pass before committing. One pre-existing test failure is expected: `test_encrypt_different_each_time` — ignore it.

## Visual Verification

To verify rendering changes visually:

```bash
# Kill any existing instance
pkill -f ThaloraBrowser 2>/dev/null; sleep 1

# Launch browser
dotnet run --project gui/ThaloraBrowser -- --url "https://blog.cloudflare.com/markdown-for-agents/" --control-port 9222 --width 1280 --height 800 &
disown

# Wait for ready
for i in $(seq 1 30); do curl -s http://localhost:9222/health && break; sleep 2; done

# Wait for page load
for i in $(seq 1 30); do STATE=$(curl -s http://localhost:9222/state); LOADING=$(echo "$STATE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('is_loading', True))" 2>/dev/null); [ "$LOADING" = "False" ] && break; sleep 2; done

# Wait for images + settle
curl -s -X POST http://localhost:9222/wait-for-images -d '{"wait_ms":5000}'
sleep 3

# Capture screenshot
curl -s "http://localhost:9222/screenshot?delay=500" -o /tmp/ralph-viewport.png

# Clean up
pkill -f ThaloraBrowser 2>/dev/null
dotnet build-server shutdown 2>/dev/null
```

Reference images are at: `gui/ThaloraBrowser/visual-tests/references/cloudflare-blog/page-01.png` through `page-11.png`

## Key File Locations

| Area | File |
|------|------|
| CSS parsing | `src/engine/renderer/css.rs` — `apply_declarations()` stores CSS values in `ComputedStyles` |
| Layout tree | `src/engine/renderer/page_layout.rs` — `build_layout_tree_from_dom()` walks DOM, applies CSS, builds `LayoutElement` tree |
| Taffy layout | `src/engine/renderer/layout.rs` — converts `LayoutElement` to taffy nodes, computes layout, extracts `ElementLayout` for JSON |
| Text measure | `src/engine/renderer/text_measure.rs` — cosmic_text font measurement |
| C# deserialize | `gui/ThaloraBrowser/Rendering/HtmlRenderer.cs` — `RustElementLayout` (JSON model), `ConvertElement()`, `BuildComputedStyle()` |
| C# painting | `gui/ThaloraBrowser/Rendering/PaintContext.cs` — `PaintBox()`, `PaintTextRun()`, background/border/text rendering |
| C# models | `gui/ThaloraBrowser/Rendering/LayoutBox.cs` — `LayoutBox`, `TextRun`, `CssComputedStyle` |
| Image cache | `gui/ThaloraBrowser/Rendering/ImageCache.cs` — async image downloading |

## Critical Rules

- **NO MOCKS** — implement real functionality, never stub things out
- **NO SIMPLIFYING** — take the proper approach even if it takes longer
- Do NOT add timeouts to cargo build/check/test commands
- Kill ThaloraBrowser processes after visual verification (memory: ~190MB each)
- Run `dotnet build-server shutdown` after C# builds to free ~300MB Roslyn compiler memory

## Progress Report Format

APPEND to progress.txt (never replace):
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
---
```

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete: reply with `<promise>COMPLETE</promise>`

If stories remain with `passes: false`: end normally (another iteration picks up next story).

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep builds green
- Read Codebase Patterns in progress.txt before starting
