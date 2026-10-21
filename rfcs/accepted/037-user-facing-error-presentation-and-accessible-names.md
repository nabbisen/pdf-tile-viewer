---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Accepted (planned for 2.0.1)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 013, RFC 016, RFC 017, RFC 025
amends: RFC 017 (§ error mapping), RFC 025 (§ error semantics)
origin: architect onboarding review 2026-10-21 (internal)
---

# RFC 037 — User-Facing Error Presentation and Accessible Names

## 1. Summary

Users must never see Rust `Debug` output or untranslated text, and the type
system should make "wrong message for this failure" unrepresentable. This RFC
centralizes failure-to-message mapping, removes now-misleading error states left
by RFC 025, and gives every icon-only control an accessible name.

## 2. Motivation (evidence)

- `{e:?}` is rendered to users in four places: search failure
  (`search_panel.rs`), zoom render failure (`zoom_overlay.rs`), tile render
  failure (`viewer/render.rs`), reveal-in-file-manager (`viewer.rs`). Results
  look like `SearchFailed("…")`: not localized (violates the GUI i18n rule), not
  understandable, and potentially revealing internals.
- Jump-to-page errors are hard-coded strings (`"?"`, `"1–N"`).
- After RFC 025, `DocumentError::EncryptedUnsupported` is unreachable from the
  engine, yet `open_error_key` still maps both it and `PasswordRequired` to
  "Password-protected PDFs are not supported yet" — a **false statement** if it
  were ever shown. A `debug_assert!` gated by an environment variable papers
  over this.
- Several icon-only buttons (search, reveal, Zen, close) have a `title` but no
  `aria-label`; the outline and zoom buttons do. The convention is inconsistent.

## 3. Goals

- No `Debug`/internal text in any user-visible string.
- One mapping function per failure family, exhaustive by construction.
- Illegal combinations (e.g. "password required" shown as a failure) cannot be
  expressed.
- Consistent accessible names for all icon-only controls.

## 4. Non-goals

- A diagnostics panel or log export (separate, future).
- Replacing emoji icons with a drawn icon set.
- New languages.

## 5. Design

### 5.1 Failure families and keys

Add exhaustive mappers next to `open_error_key` (module `i18n::failure`), one
`match` without wildcard arms per type so a new variant fails compilation:

| Source type | Mapper |
|---|---|
| `SearchError` | `search_error_key` |
| `RenderError` | `render_error_key` |
| `TextLayerError` | `text_layer_error_key` |
| `NavigationError` | `navigation_error_key` |
| `PlatformError` | `platform_error_key` |

State types store a `MessageKey`, not a `String`:
`SearchState::Failed(MessageKey)`, `TileImageState::Failed(MessageKey)`,
`ZoomImageState::Failed(MessageKey)`, `reveal_error: Option<MessageKey>`.
Messages are written for users ("Search could not be completed. Try again.").
Internal detail is shown only in debug builds, appended after the localized text
using the existing `DiagnosticDetailsLabel` pattern; release builds never show
it. No message contains a file path (RFC 016 § 8).

Add English and Japanese catalog entries and extend the completeness tests.

### 5.2 Make illegal states unrepresentable

Introduce `OpenFailure`, the subset of `DocumentError` that can be *displayed*
as an open failure (everything except `PasswordRequired`). `OpenError::Engine`
carries `OpenFailure`; `document_service` converts `PasswordRequired` into
`OpenDocumentOutcome::PasswordRequired` before this point. Remove
`DocumentError::EncryptedUnsupported`, the `debug_assert!`/environment-variable
hack, and `MessageKey::ErrEncryptedUnsupported`. The RFC 024 smoke test name and
the i18n test that reference the old state are updated; RFC 024 receives an
"amended by RFC 025/037" note. `TooLargeForPolicy` stays (reserved for
resource-limit work, documented as such).

### 5.3 Jump-to-page feedback

Replace the hard-coded strings with one localized message ("Enter a page number
between 1 and the last page.") shown for any invalid input; the numeric input
already carries `min`/`max`. No interpolation is needed.

### 5.4 Accessible names

Every icon-only control sets `aria-label` equal to its localized `title`.
A grep-based unit test fails if a `button` whose only child is a single
glyph/emoji lacks `aria-label`.

### 5.5 Lint

Enable `#![warn(clippy::use_debug)]` in the `app` crate (tests exempt) so
`{:?}` formatting in UI code is flagged. Whether this becomes blocking depends on
the owner decision recorded in RFC 036 § 5.6.

## 6. Alternatives considered

- **Keep strings in state and format at the edge.** Rejected: the bug *is*
  formatting at the edge.
- **Localize inside `domain`/`pdf_engine`.** Rejected: violates RFC 017 § 6
  (those crates carry no localized text).

## 7. Security and privacy

Reduces accidental disclosure of internal error text and paths. No new data
flows.

## 8. Tests and acceptance

- Mapper tests (exhaustive by compilation) and catalog completeness for `en`/`ja`.
- `OpenFailure` conversion tests (password path never reaches the display path).
- Grep-based guard tests (no `{e:?}`-style formatting in `crates/app/src`
  outside tests; icon-only buttons have `aria-label`).
- Manual: force each failure (stop engine thread in a debug build, bad query,
  missing `xdg-open`) and verify localized, path-free messages in both languages.

## 9. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Large mechanical change touching several components | Low | One slice per component; compiler-guided |
| Message wording needs owner/translator review | Low | Wording table in the handoff |
