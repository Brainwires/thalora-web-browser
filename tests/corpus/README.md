# Real-world pattern corpus

`pages/` holds small, self-authored pages that each reproduce a pattern common
on real sites (client-side rendering from fetched JSON, web components,
module scripts, delayed "skeleton" rendering, JS-built tables, forms,
MutationObserver UIs, `document.write`, event delegation, and post-load
interaction: accordions, tabs, client-side form validation, "load more"
buttons, counters, live-filtered lists). Each `X.html` has an
`X.expect.json`:

```json
{
  "text": ["must appear"],
  "absent": ["must not appear"],
  "min_refs": 3,
  "steps": [
    {"action": "click", "target": {"text": "Show details"}},
    {"expect": {"text": ["Details are visible"], "absent": ["Show details"]}}
  ],
  "known_failing": "snapshot reads the HTML captured at page load"
}
```

`text`/`absent` are checked against `browser_get_page_content` after a
navigation that runs the page's JavaScript; `min_refs` is the minimum number
of interactive `[ref=…]` entries in `browser_snapshot`.

## Interactive steps

`steps` (optional) runs in order, in the same session, after the load-time
checks. Each step is either an action or an `expect` check.

| Action | Fields | MCP tool |
| --- | --- | --- |
| `click` | `target` | `browser_click` |
| `type` | `target`, `value` (replaces the field's text) | `browser_type_text` |
| `fill` | `target`, `value` | `browser_fill` |
| `select` | `target`, `value` (option value or text) | `browser_select_option` |
| `check` | `target`, `checked` (default `true`) | `browser_check` |
| `press` | `key` (or `value`), optional `target` | `browser_press_key` |
| `wait` | `ms`; or `text` / `selector` plus `ms` as timeout | `browser_wait` |

`target` is resolved by taking a fresh `browser_snapshot` and picking the
first element whose accessible name equals `text` (or `label` / `name`;
case-insensitive, falling back to a substring match), optionally restricted
to a `role` (e.g. `{"role": "tab", "text": "Reviews"}`). `{"selector": "css"}`
skips the snapshot and passes a CSS selector instead.

`{"expect": {"text": [...], "absent": [...]}}` checks **both**
`browser_get_page_content` and `browser_snapshot`; the step result records
which of the two failed (`failed_in`), so a stale extraction path shows up
even when the other one is correct. Put text that a step must reveal in an
element the snapshot shows as text (`p`, `li`, `td`, …) or as an element name.

## Known failing pages

`"known_failing": true` (or a reason string) marks a bundled page that is
expected to fail for now. It is still scored and reported, but does not fail
the test. When such a page starts passing the test prints a notice to remove
the flag.

The corpus test (`cargo test --test mcp_tests corpus`) serves these pages
through the fixture server and fails if any bundled page that is not
`known_failing` misses its expectations.

## Local real pages

Save real pages (with their own `.expect.json`) under `local/`. It is
git-ignored, so copies of third-party sites never get committed. Local pages
are scored but never fail the test.

## Scoreboard

`THALORA_CORPUS_WRITE=1 cargo test --test mcp_tests corpus -- --nocapture`
rewrites `scoreboard.json` so improvements are measurable between runs. Per
page it records pass/fail, missing/unexpected text, ref count,
`known_failing`, and for pages with steps `steps_passed`/`steps_total`,
`first_failed_step` and one result per step (action: tool, resolved target,
error; expect: `failed_in` plus missing/unexpected text for page content and
snapshot separately). Navigation timings are printed but kept out of the file.
