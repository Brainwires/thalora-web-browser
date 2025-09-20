Tests directory organization
===========================

This repository contains a large number of tests under the `tests/` directory. This README explains recommended conventions, how to find duplicate files, and next steps for reorganization.

Goals
-----
- Make tests easy to find by feature and category
- Avoid near-duplicate or exact-duplicate files
- Provide clear naming conventions so new tests end up in the right place

High-level recommended layout
-----------------------------
- `tests/apis/` - tests that target specific web platform APIs (fetch, geolocation, service worker, etc.)
- `tests/features/` - higher-level feature tests and transformations (transpilation, engine behaviors)
- `tests/compatibility/` - cross-engine or browser compatibility suites
- `tests/integration/` - full-app or end-to-end style tests relying on multiple subsystems
- `tests/engine/` - engine-specific behavior and performance tests
- `tests/utils/` - test helpers and shared utilities

Naming conventions
------------------
- Use snake_case and end filenames with `_test.rs` when they contain Rust test harness code.
- For grouped tests in subfolders, use a clear prefix (e.g. `geolocation/` contains geolocation-specific tests).

Deduplication workflow
----------------------
1. Use the included script `scripts/dedupe_tests.py` to scan for duplicate files.
   - Dry-run: `python3 scripts/dedupe_tests.py --root tests --output scripts/dedupe_report.json`
   - Normalize whitespace variant: `python3 scripts/dedupe_tests.py --root tests --output scripts/dedupe_report.json --normalize`
2. Inspect `scripts/dedupe_report.json` to see groups of duplicate files.
3. For exact duplicates decide on a canonical location to keep one copy. You can either:
   - Replace duplicates with a relative symlink to the canonical copy using `--apply` (fast), or
   - Move duplicates into an `archive/` dir while creating a symlink pointing to the kept copy: `--apply --archive archived_tests`

Recommendations for non-exact duplicates
--------------------------------------
- If tests are nearly identical but differ in small details, prefer merging them into a single parametrized test, or add small helper functions in `tests/utils/`.
- If tests have different names but same content because of copying for historical reasons, prefer keeping one and removing copies.

Edge cases and safety
---------------------
- The script does not modify files by default. Use `--apply` to make changes.
- When applying, backups are created with a `.bak` suffix or moved to the `archive` folder.
- Symlinks are relative, preserving repository portability.

Next steps for maintainers
-------------------------
1. Run the dedupe script and review the generated JSON report.
2. Decide canonical locations for duplicate groups.
3. Run `--apply` during a branch so changes can be reviewed in a PR.
4. Optionally add CI checks to prevent duplicate test content (hash-based check) if desired.

Contact
-------
If you want help applying the dedupe changes or designing a concrete reorganization plan, open an issue or ask here and include the generated report.
