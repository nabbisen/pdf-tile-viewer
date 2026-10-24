# Handoff — RFC 037 User-Facing Error Presentation and Accessible Names

Governing RFC: [`../../accepted/037-user-facing-error-presentation-and-accessible-names.md`](../../accepted/037-user-facing-error-presentation-and-accessible-names.md) (incl. § 5.6, amended 2026-10-24). Evidence: `.git-exclude/review-request/wp0-evidence-run/` (Part C dumps and Orca logs).
Executes the RFC; must not redefine it. Slice F, after RFC 033 (slice E). Working model: directly on `main`, one actor, hand over with the commit hash, `main` green. No tags, no releases (DEC-001). **GUI test protocol applies** (scratch `XDG_CONFIG_HOME`, transient unit, focus check before every synthetic input, `wtype -d 50`). Re-read files; line numbers have moved.

## Change scope (RFC 037 §§ 5.1–5.6)
1. **Failure mapping** (`crates/app/src/i18n/failure.rs`, new): exhaustive `match` mappers `search_error_key`, `render_error_key`, `text_layer_error_key`, `navigation_error_key`, `platform_error_key` (no wildcard arms). State types store `MessageKey` (`SearchState::Failed(MessageKey)`, `TileImageState::Failed(MessageKey)`, `ZoomImageState::Failed(MessageKey)`, reveal error). Debug detail only in debug builds after the localized text (existing `DiagnosticDetailsLabel` pattern); never a path. English and Japanese strings; completeness tests extended.
2. **`OpenFailure`** per § 5.2: remove `DocumentError::EncryptedUnsupported`, the env-var `debug_assert!` in `open_error_key`, `MessageKey::ErrEncryptedUnsupported`; update the affected tests (note RFC 024/025 front matter via RFC 038).
3. **Jump-to-page feedback** per § 5.3 (one localized message).
4. **Accessibility extensions** per § 5.6, items 1–7: names (`aria-label` on search/reveal/zen/zen-exit buttons; slider name and value text; label association for jump and columns inputs), document title and `<html lang>` via one Rust function + minimal script, `role="status"` / `role="alert"` regions present before their text changes, `main` landmark in the viewer (also in zen), plural-aware messages via a pure `plural(locale, count, one, other)` helper with tests, the short-query hint (visible, localized), extended grep guard.
5. **Lint**: `#![warn(clippy::use_debug)]` in `app` (tests exempt).
6. **Docs**: `docs/src/contributors/dioxus-desktop-lessons.md` § 3 and § 6: change "tracked in" to "fixed in 2.0.2" only for verified items; `release-qa.md`: add accessibility lines (named controls, status region, title/lang); `CHANGELOG.md` `[Unreleased]` entries (Fixed: error messages, accessible names, document language).

## Non-change scope
Keyboard routing and focus (RFC 033), tile keyboard focus (RFC 042), system-locale detection (separate RFC), error recovery behavior, adding diagnostics UI.

## Required tests and evidence
1. Unit tests: mappers (exhaustive by compilation), `plural` for en/ja, `OpenFailure` conversion (password outcome never reaches the display path), catalog completeness for both languages, grep guards.
2. Gate green (`cargo fmt --check`, `cargo check --workspace` 0 warnings, `cargo test --workspace`, smoke).
3. **Re-run WP0 Part C items C1–C3, C6, C8** on the final build and deliver before/after dumps (`atspi.py` fixed copy in the WP0 package): unnamed or glyph-named controls: none; `lang` attribute `en`/`ja` per locale; the `status` node after a search; `main` landmark present; document title equals the specified text (debug-build inspector, as WP0 C8). Force each failure kind where possible (stop the engine thread in a debug build, bad query, missing `xdg-open`) and attach screenshots of the localized, path-free messages in both languages.
4. **Do not claim screen-reader announcements** in your report; Orca verification belongs to the owner's listening session (AU-8). Report the AT-SPI live attributes only.

## Acceptance criteria
RFC 037 § 8 plus § 5.6: no `Debug` text or path in any user-visible string; every interactive element has a name; document language follows the locale; results and errors are exposed as live regions; plural forms correct in English.

## Prohibited shortcuts
Formatting errors at the edge; `String` error state; hard-coded UI strings; emoji used as the only name; adding `aria-label` text that differs from the visible meaning.

## Review request
`.git-exclude/review-request/037-errors-names/README.md` (single entry point), standard fields, before/after dumps, screenshots.
