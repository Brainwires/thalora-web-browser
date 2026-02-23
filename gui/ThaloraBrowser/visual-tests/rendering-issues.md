# Rendering Issues Tracker

## Active Issues

### ISSUE-001: No max-width centering on main content
- **Severity**: HIGH (layout)
- **Viewports affected**: ALL
- **Description**: Content spans full 1280px width instead of being constrained to ~720px centered container
- **Root cause**: max-width + margin:auto not producing centered layout
- **Fix location**: src/engine/renderer/layout.rs (margin auto handling), page_layout.rs
- **Status**: NEW

### ISSUE-002: No background colors anywhere
- **Severity**: HIGH (visual)
- **Viewports affected**: ALL
- **Description**: Header should be dark/orange, code blocks should have gray backgrounds, etc. Everything is white.
- **Root cause**: Background colors from CSS not being applied to layout elements
- **Fix location**: src/engine/renderer/css.rs, page_layout.rs
- **Status**: NEW

### ISSUE-003: No images rendering
- **Severity**: HIGH (content)
- **Viewports affected**: 01 (hero image), others
- **Description**: Hero illustration and other images not showing — just blank space or nothing
- **Root cause**: Image elements may not be in the layout tree, or img_src not flowing through
- **Fix location**: page_layout.rs (image handling), PaintContext.cs (image painting)
- **Status**: NEW

### ISSUE-004: Navigation should be horizontal flex layout
- **Severity**: MEDIUM (layout)
- **Viewports affected**: 01
- **Description**: Top nav items stacked vertically instead of horizontal row
- **Root cause**: Flexbox layout (display:flex) not producing horizontal arrangement
- **Fix location**: layout.rs (flex handling in taffy)
- **Status**: NEW

### ISSUE-005: Link text should be colored (blue/orange)
- **Severity**: MEDIUM (visual)
- **Viewports affected**: ALL
- **Description**: All links are black; reference shows blue title link, orange category links
- **Root cause**: Link color from CSS not being resolved or applied
- **Fix location**: css.rs, page_layout.rs (color resolution)
- **Status**: NEW

### ISSUE-006: Code blocks missing monospace + background
- **Severity**: MEDIUM (visual)
- **Viewports affected**: 02-08
- **Description**: Code blocks should have gray background, monospace font, proper padding
- **Root cause**: pre/code elements not getting background-color or proper font styling
- **Fix location**: page_layout.rs, css.rs
- **Status**: NEW

## Fixed Issues

### ISSUE-000: Text line overlap
- **Severity**: HIGH (overlap)
- **Status**: FIXED
- **Fix**: Eliminated pre-split lines; Avalonia handles all wrapping
