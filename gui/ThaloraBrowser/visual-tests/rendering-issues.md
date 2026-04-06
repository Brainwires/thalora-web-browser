# Rendering Issues Tracker

All issues from the old pipeline have been resolved. The rendering pipeline was completely
rebuilt in Feb–Apr 2026:

- Old pipeline (taffy + cosmic_text + PaintContext manual painting) → **deleted**
- New pipeline: Rust CSS resolution → StyledElement tree JSON → C# ControlTreeBuilder → Avalonia native controls

## Visual Regression Workflow

```bash
# Take a screenshot
cargo xtask gui-screenshot https://www.google.com --out /tmp/thalora.png

# Compare against a Chrome reference
cargo xtask gui-compare https://www.google.com --ref /tmp/chrome-google.png
```

Reference images live in `gui/ThaloraBrowser/visual-tests/`.

## Known Gaps (Apr 2026)

| Area | Details |
|---|---|
| SVG rendering | Wikimedia SVGs get 403 — needs proper User-Agent in image fetcher |
| BuildFromJson blocks UI thread | Large pages (GitHub 641KB tree) freeze UI during tree build; background-thread fix blocked by Avalonia thread affinity |
| `outerHTML` for complex pages | Boa returns `undefined` for Wiktionary-style pages; fallback to original server HTML |
