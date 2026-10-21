---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-22
status: Accepted (emergency fix; planned for 2.0.1)
accepted: 2026-10-22
baseline: PDF Tile Viewer 2.0.0
priority: Emergency (severity high, narrow scope)
depends_on: RFC 008, RFC 013
origin: WP0 evidence run 2026-10-22 (internal), finding P0
---

# RFC 039 — Viewer Scale Shortcut Crash (Emergency Fix)

## 1. Summary

The released 2.0.0 viewer **aborts the whole application** when the user presses
`+`, `=`, `-`, or scrolls with `Ctrl` held — gestures documented in
`shortcuts.md` and `tile-grid.md`. This RFC defines the narrowest correct fix and
the regression protection. It deliberately changes nothing else; the broader
keyboard redesign remains RFC 033.

## 2. Evidence (reproduced and verified)

- `crates/app/src/screens/viewer.rs` calls
  `change_scale(*scale.peek() + 0.2)` (lines 239, 240) and
  `change_scale(*scale.peek() + …)` in the wheel handler (line 258). The
  temporary borrow created by `peek()` lives until the end of the enclosing
  statement, i.e. **during** the call. `change_scale` then calls `scale.set(c)`
  (line 205), which needs a write borrow → `AlreadyBorrowed` panic → the panic
  occurs in a callback that cannot unwind → process abort.
- Reproduced by the dev team in the debug build (`+`, `=`, `-`) and in the
  **release build** (`-`, `Ctrl+wheel`), with logs
  (`panicked at crates/app/src/screens/viewer.rs:205:19 … AlreadyBorrowed`).
- The `0` key (`change_scale(default_scale)`, no `peek()` in the call), the toolbar
  `+`/`−` buttons, the slider and the zoom overlay's own scale controls are not
  affected (they read the value in a separate statement).
- No other call site with the same pattern exists in `crates/app/src` (verified
  by search; the dev team re-verifies).

Impact: any user pressing the zoom shortcuts or using `Ctrl+scroll` (a very
common gesture) loses the running session. Likelihood is high; the fix is small.

## 3. Goals / Non-goals

Goals: remove the crash; make the scale-change logic a pure, tested function;
document the Dioxus signal rule so the class of bug is not reintroduced.
Non-goals: shortcut routing, Escape behavior, input guards (RFC 033); stale-scale
tiles (RFC 035); any UI change.

## 4. Design

1. **Pure scale arithmetic in `domain`** (`layout::ViewerScale`):
   ```rust
   pub enum ScaleChange { To(f32), By(f32) }
   impl ViewerScale {
       /// Next scale for `change` starting from `current`: apply, round to 0.1,
       /// clamp to [MIN, MAX]. Returns `None` when the result differs from
       /// `current` by less than 0.001.
       pub fn next(current: f32, change: ScaleChange) -> Option<f32>;
   }
   ```
2. **Viewer**: `change_scale` takes a `ScaleChange`. Inside it, read the current
   value in its own `let` statement (`let current = *scale.peek();`) so no borrow
   is alive when `scale.set` runs; then `ViewerScale::next(current, change)`; if
   `Some(next)`, `scale.set(next); invalidate();`. Call sites become
   `change_scale(ScaleChange::By(0.2))`, `By(-0.2)`, `By(±0.2)` for the wheel, and
   `To(default_scale)` for `0`.
3. **Documentation**: add a short "Signal borrow rule" to
   `docs/src/contributors/dev.md`: never keep a `read()`/`peek()` guard (including
   temporaries inside call arguments) alive across a `set()`/`write()` of the same
   signal; read into a `let` first.
4. **Release QA protocol**: add `docs/src/contributors/release-qa.md` with a
   manual smoke checklist that exercises **every documented shortcut and gesture**
   on a release build (this crash shipped because documented shortcuts were not in
   any checklist).

## 5. Alternatives considered

- **Use `scale.with_mut(...)`.** Rejected as the only fix: it removes this bug but
  leaves the arithmetic untestable inside a closure.
- **Fold the fix into RFC 033.** Rejected: the bug is in the shipped release and
  the fix is independent of the routing redesign; waiting would leave a known abort
  live for weeks.

## 6. Security / robustness

Removes a denial-of-service-by-keystroke in the shipped application. No new input
surfaces.

## 7. Tests and acceptance

- `domain` unit tests for `ViewerScale::next`: step up/down from 1.0, rounding to
  0.1, clamping at 0.2 and 5.0, `None` for no-change, `To` equal to current.
- Manual on a **release build** (staged package layout): `+`, `=`, `-`, `0`,
  `Ctrl+wheel` up/down × 20 each on a 100-page document; no abort; scale changes
  by 0.2 per step; toolbar and slider still work. Same checks with the zoom
  overlay open (its own `+`/`-` must still work; bubbling to the viewer remains
  RFC 033's problem but must not crash).
- No crash in the application log (`stderr`) during the run.

## 8. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Fix changes scale rounding behavior | Low | Same arithmetic, now tested |
| Reintroduction elsewhere | Medium | Dev-guide rule + release QA checklist |
