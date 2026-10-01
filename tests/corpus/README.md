# Real-world pattern corpus

`pages/` holds small, self-authored pages that each reproduce a pattern common
on real sites (client-side rendering from fetched JSON, web components,
module scripts, delayed "skeleton" rendering, JS-built tables, forms,
MutationObserver UIs, `document.write`, event delegation). Each `X.html` has
an `X.expect.json`:

```json
{"text": ["must appear"], "absent": ["must not appear"], "min_refs": 3}
```

`text`/`absent` are checked against `browser_get_page_content` after a
navigation that runs the page's JavaScript; `min_refs` is the minimum number
of interactive `[ref=…]` entries in `browser_snapshot`.

The corpus test (`cargo test --test mcp_tests corpus`) serves these pages
through the fixture server and fails if any bundled page misses its
expectations.

## Local real pages

Save real pages (with their own `.expect.json`) under `local/`. It is
git-ignored, so copies of third-party sites never get committed. Local pages
are scored but never fail the test.

## Scoreboard

`THALORA_CORPUS_WRITE=1 cargo test --test mcp_tests corpus -- --nocapture`
rewrites `scoreboard.json` (per page: pass/fail, missing text, ref count,
navigation time) so improvements are measurable between runs.
