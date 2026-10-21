# Handoff — RFC 039 Viewer Scale Shortcut Crash (Emergency Fix)

Governing RFC: [`../../accepted/039-scale-shortcut-crash-hotfix.md`](../../accepted/039-scale-shortcut-crash-hotfix.md) — accepted by the owner on 2026-10-22; the dev team may start.
Executes the RFC; must not redefine it. Conflicts or gaps → design request to the architect.

Schedule: one working day, then architect review (the owner decides when to hand over; the planned release go/no-go for the hotfix stays with the owner). Working model: directly on `main`, no branches, you are the only actor on the tree; start from a clean `git status` at the hand-over commit; hand over with the `main` commit hash. **No tags, no pushes of tags, no releases (DEC-001).**

## Purpose
Remove the abort on `+`, `=`, `-` and `Ctrl+wheel` in the viewer (RFC 039 § 2) with the smallest correct change, plus regression protection.

## Change scope
1. `crates/domain/src/layout.rs` (tests in `layout/tests.rs`): add `ScaleChange { To(f32), By(f32) }` and `ViewerScale::next(current, change) -> Option<f32>` exactly as in RFC 039 § 4.1 (round to 0.1, clamp to `[MIN, MAX]`, `None` when the difference to `current` is < 0.001). Reuse the existing constants.
2. `crates/app/src/screens/viewer.rs`: change `change_scale` to take `ScaleChange`; read the current value in its own `let` statement (no `peek()`/`read()` temporary in any call argument); update the four call sites (keys `+ =`, `-`, `0`, wheel). Do **not** touch anything else in this file.
3. `docs/src/contributors/dev.md`: add the "Signal borrow rule" (RFC 039 § 4.3) as a short subsection.
4. `docs/src/contributors/release-qa.md` (new; add to `docs/src/SUMMARY.md` under Contributors): manual smoke checklist for a **release build** that exercises every shortcut and gesture documented in `docs/src/intermediate/shortcuts.md` and `docs/src/new-users/tile-grid.md` (viewer: `Z`, `+`, `=`, `-`, `0`, `Ctrl+wheel`, `Escape`; zoom overlay: arrows, `PageUp/Down`, `Home/End`, `+`, `-`, `Escape`; search: `Enter`, `Escape`), each with the expected result and a pass/fail box; plus open/close of a document, search, outline, Zen, password prompt, external-link dialog. Keep each line to one observable expectation. Where the current behavior is a known defect covered by an accepted RFC (for example `Escape` leaving the document, RFC 033), write the *currently correct documented* expectation and mark the line "known failing until RFC 033".

## Non-change scope
Shortcut routing, input guards, Escape behavior, tile freshness, zoom overlay code, CHANGELOG, version numbers, CI, any other file.

## GUI test protocol (mandatory; lessons from the evidence run)
- Run the app with a scratch `XDG_CONFIG_HOME` so **your real settings file is not touched**; start the app (and any helper) as a transient `systemd --user` unit.
- Before **every** `wtype`/`xdotool` call confirm the focused window is the app (`niri msg --json windows`); use `wtype -d 50` for typing.
- Use the staged release layout you built for the evidence run (release binary + bundled PDFium) for the final verification.
- Close the app afterwards; leave no window or unit running.

## Required tests and evidence
1. `domain` tests listed in RFC 039 § 7 (step up/down, rounding, both clamps, `None` cases, `To` equal to current).
2. Gate green: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace` (expect 133 + the new tests); smoke with `PDF_TILE_VIEWER_PDFIUM_DIR`.
3. Manual verification on a **release build** per RFC 039 § 7, including the zoom-overlay variant; paste the app's stderr log (no `panicked`) and record counts of presses/steps. Fill in the new release-QA checklist for the lines you executed.
4. A short statement that the search for the same pattern (`peek()`/`read()` temporaries in call arguments that reach a `set`/`write` of the same signal) found no other site, with the command used.

## Acceptance criteria
RFC 039 § 7 satisfied; no change outside the four listed files (plus tests and `SUMMARY.md`); review request delivered.

## Release preparation (separate, last commit — only if the architect orders it after review)
The owner decides whether and when to release. If ordered: (a) record the commit of the stray local tag `2.0.1` (`5c69522`) in the review request and delete the **local** tag (`git tag -d 2.0.1`; owner decision D7) — the release tag name must be free; (b) bump the workspace version to `2.0.1` in `Cargo.toml` and `Cargo.lock`; (c) add a dated `## [2.0.1]` section to `CHANGELOG.md` (Fixed: the abort); (d) update user-doc statements that name the "2.0.0 release line". Do **not** create or push any tag.

## Prohibited shortcuts
Wrapping the call in `catch_unwind` or ignoring the panic; changing handler structure beyond the call-site change; skipping the release-build verification; editing the owner's real settings file.

## Review request
`.git-exclude/review-request/039-scale-crash-hotfix/README.md` (single entry point): summary, commits, changed files, deviations, commands and results, release-build verification log, search command for other sites, unresolved issues, requested review focus.
