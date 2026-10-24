# Handoff — RFC 033 Keyboard Routing, Modal Layers, and Dismissal Model

Governing RFC: [`../../accepted/033-keyboard-routing-and-dismissal-model.md`](../../accepted/033-keyboard-routing-and-dismissal-model.md) (incl. the 2026-10-24 amendment in § 5.4). Evidence: [`evidence-run.md`](evidence-run.md) and its review package (`.git-exclude/review-request/wp0-evidence-run/`).
Executes the RFC; must not redefine it. Slice E, after RFC 040 (slice R). Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases (DEC-001). **GUI test protocol applies** (scratch `XDG_CONFIG_HOME`, transient `systemd --user` unit, verify the focused window before every `wtype`/`xdotool` call, `wtype -d 50`). Line numbers have moved since the RFC was written: re-read the files.

## Release context (important)
This ships in a **patch** release (2.0.2). The documented behavior "Escape with nothing to dismiss returns to the dashboard" **stays** (RFC 033 § 5.4 two-step rollout); only the bug that one Escape both closes a layer *and* leaves the document is fixed. `domain::shortcuts::ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE` is `true` in this release. Do not change documented behavior beyond the RFC.

## Change scope
1. **Resolver** — `crates/domain/src/shortcuts.rs` (+ `shortcuts/tests.rs`): `Layer`, `ShortcutContext { top_layer, zen }`, `KeyDescriptor`, `ShortcutAction`, `resolve(...)`, the const above, and the dismissal order of RFC § 5.4. Pure; no Dioxus. Tests: full context × key matrix; "no base shortcut resolves while a Dialog or Zoom is on top"; both values of the const (the `false` branch is exercised in tests only); a test that every row of the shortcut table in `docs/src/intermediate/shortcuts.md` resolves (parse the table in the test or mirror it in a const table next to the resolver and assert equality).
2. **Viewer root** (`crates/app/src/screens/viewer.rs`): exactly one `onkeydown` that converts the event with a small `key_descriptor(&KeyboardData)` helper, calls `resolve`, dispatches. Use RFC 039's `change_scale(ScaleChange::…)`; keep every `read()/peek()` in its own `let` (signal borrow rule).
3. **Modal layers own the keyboard**: zoom overlay root, external-link dialog, password prompt each handle their keys via the resolver and **always `stop_propagation()` at their root**. Verify with the evidence-run checks (A2): `+`/`-`/`z` inside the overlay no longer reach the viewer; one Escape closes only the overlay.
4. **`crates/app/src/components/modal.rs`** (new, outside `zoom_overlay`): the shared modal focus contract of RFC § 5.5 — focus the first meaningful control on open, trap Tab inside, restore focus to the opener on close — implemented once (minimal script helper keyed by element ids). Migrate the external-link dialog's `focus_*`/`trap_*` functions into it and apply it to the password prompt and the zoom overlay. While the password prompt or any dialog is open the background content is **inert** (`inert` attribute on the sibling content; fall back to `aria-hidden` if the WebView does not support `inert` — record which one worked in the evidence).
5. **Panel and Zen focus rules** (WP0 C4 failures): after opening a document focus the viewer container; opening search focuses the input and closing it focuses the search button; opening the outline focuses its first entry and closing returns to the outline button; leaving Zen focuses the toolbar's Zen button (or the viewer container if the toolbar is absent); closing the overlay returns focus to the originating tile or the viewer container (tiles are not focusable in this release — RFC 042 later).
6. **Editable-field helper** `editable_field_keys(on_escape)` (in `modal.rs` or `keys.rs`) applied to the search entry, jump input, columns number input and the password input; plus the grep-based guard test in `crates/app/src/tests/` that fails when an `input`/`textarea` is built without it.
7. **Auto / Fixed columns control** (RFC § 5.6): segmented control `[Auto | Fixed]` with a number stepper enabled only in Fixed; switching to Fixed starts at the **currently displayed column count**; Fixed → Auto restores auto. Remove the click-on-disabled-input hack. Range 1–24. i18n keys in English and Japanese.
8. **Duplication cleanup** (Slice A finding): the "open external-link dialog" code (increment the sequence, set the state) exists three times in `zoom_overlay/link_layer.rs`; extract **one** helper and use it for mouse-up, Enter and Space.
9. **Docs**: `docs/src/intermediate/shortcuts.md` matches the resolver (including the corrected Escape order and that one press closes one layer); `docs/src/contributors/release-qa.md`: re-check the lines marked "known failing until RFC 033", tick them or keep the marker with a reason; `docs/src/contributors/dioxus-desktop-lessons.md` §§ 2–3: update the "tracked in" statements to "fixed in 2.0.2 (RFC 033)" only for what is verified; `CHANGELOG.md` `[Unreleased]` entries.

## Non-change scope
Keyboard navigation of tiles (RFC 042), document title/language/live regions (RFC 037), the idle-Escape behavior (stays), any new shortcut, settings, geometry.

## Required tests and evidence
1. Unit tests above; gate green (`cargo fmt --check`, `cargo check --workspace` 0 warnings, `cargo test --workspace`, smoke).
2. **Re-run WP0 Part A items A1, A2, A4 and Part C items C4, C5** on the final build (debug acceptable for focus/propagation; the shortcut crash-class checks on a **release** build) and report each as before/after with logs, AT-SPI dumps and screenshots. The key-leak matrix of A1 (search, jump, columns × `a z 0 - + =`) must show no scale/zen change and no abort.
3. Manual matrix on the owner's other platforms is **not** yours; list the cases for the owner (focus trap, inert background, Escape in every layer combination) in the review request.
4. Re-run the whole release-QA checklist section 3–5 on a release build and attach the filled lines.

## Acceptance criteria
RFC 033 § 10 (as amended): no viewer shortcut fires while an editable control has focus; no overlay or dialog key reaches the viewer root; one Escape closes at most one layer and never both closes a layer and leaves the document; the idle fallback still works; Auto → Fixed starts at the visible count; docs match the resolver.

## Prohibited shortcuts
Per-input `stop_propagation` scattered in components; JavaScript key logic; keeping the old handler "alongside" the resolver; changing idle Escape; skipping the before/after evidence.

## Review request
`.git-exclude/review-request/033-keyboard/README.md` (single entry point), with the standard fields, the before/after tables, dumps, screenshots, and the owner's cross-platform case list.
