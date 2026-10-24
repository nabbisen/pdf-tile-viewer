# Handoff — RFC 040 Tile Retention Window

Governing RFC: [`../../accepted/040-tile-retention-window.md`](../../accepted/040-tile-retention-window.md) — accepted by the owner on 2026-10-24.
Executes the RFC; must not redefine it. Slice R, after RFC 035 (slice D) is committed. Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases (DEC-001). **GUI rules apply** — `.git-exclude/rules/project-instructions-gui-common.md`, section *GUI Evidence and Automation Rules*: state the question each capture answers and prefer machine comparison or recorded state over pictures; isolated `XDG_CONFIG_HOME` (this app's config folder is fixed to `pdf-tile-viewer` in `packaging::app_dirs`, it does not depend on the executable's file name, so the rule's binary-name sentence does not apply here); zero running instances before you start, exactly one during the run, chosen by process id; state the backend that produced each piece of evidence and compare before/after only on the same one; assert after every step and **abort at the first anomaly**; no frame-sampling loops; if a capture set must be repeated a second time, stop and ask the architect before a third; verify the focused window before every synthetic input..

## Purpose
Bound UI-side tile memory by distance from the viewport (RFC 040 § 4).

## Change scope
1. `crates/domain/src/layout.rs` (+ `layout/tests.rs`): `RetentionLimits`, `retention_victims` exactly per RFC 040 § 4.1, plus `RetentionLimits::for_viewport(viewport_height_px)` returning `window_px = max(2 × h, 1500)`, `max_ready_tiles = 80`. Constants live next to the function.
2. `crates/app/src/screens/viewer/render.rs`: after computing visibility, collect the pages currently `Ready` (RFC 035's `TileImageState::Ready { .. }`), call `retention_victims`, and reset the victims to `Pending` in one `tile_images.write()` block. `Rendering`/`Pending`/`Failed` tiles are untouched. Extract the decision into a pure function `apply_retention(states: &mut TileImages, victims: &[PageIndex])` with tests.
3. `docs/src/new-users/faq.md`: one entry explaining the loading placeholder for pages far from the view; `CHANGELOG.md` `[Unreleased]` → Fixed: "Memory use no longer grows with the number of pages scrolled past (RFC 040)".

## Non-change scope
Image transport, render cache, scheduler priorities, prefetch margin, search, zoom overlay, settings.

## Required tests and evidence
1. Unit tests per RFC 040 § 7 (boundaries, visible never evicted, cap farthest-first, mixed sizes, determinism).
2. Gate: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace`; PDFium smoke.
3. **Measured acceptance** with the WP0 method on a **release build** (staged package layout, scratch config): documents text-1000 and image-100 (regenerate with the WP0 scripts in `.git-exclude/review-request/wp0-evidence-run/scripts/`); record RSS (app and WebKit children) after open, after a full scroll down, after scrolling back to the top, and after a **second** full scroll. Acceptance: ≤ 55 % of the 2.0.1 numbers in RFC 040 § 2 and a second pass adds < 10 %. Include the before/after table.
4. Manual: scroll to the end and back on image-100; tiles within two viewport heights never blink; far tiles show the placeholder and fill in; search highlights and zoom still work; responsiveness: one timed check — start a steady scroll with the window capture at the start and one capture after the scroll's expected duration, and confirm the view moved and the app answered the next input; no sampling loop (GUI rules).
5. Update `docs/src/contributors/release-qa.md`: add a line for the memory plateau.

## Acceptance criteria
RFC 040 § 7. Prohibited: unbounded caches elsewhere to "make up" for eviction; changing prefetch constants to hide flicker without recording it; skipping the measured acceptance.

## Review request
`.git-exclude/review-request/040-tile-retention/README.md` (single entry point): summary, commits, changed files, deviations, commands and results, the RSS table, manual observations, unresolved issues, requested review focus.
