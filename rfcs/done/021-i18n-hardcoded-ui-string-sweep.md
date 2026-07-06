---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented (2.0.0-beta.9)
baseline: PDF Tile Viewer 2.0.0-beta.8
---

# RFC 021 — Hardcoded UI String i18n Sweep

## 1. Summary

Remove remaining hardcoded user-visible English strings from Dioxus UI
components before 2.0 RC by moving them into the i18n catalog.

## 2. Motivation

RFC 017 requires labels, tooltips, errors, and accessible names to resolve
through the locale catalog. Some strings remain hardcoded in UI components,
including examples such as "Close search", "Page zoom view", "Zoom in",
"Zoom out", and "More controls". These are visible or accessibility-facing UI
strings and should not ship to RC outside the catalog.

## 3. Goals

- Inventory hardcoded user-visible strings in `crates/app/src`.
- Add missing `MessageKey` variants.
- Add English and Japanese translations.
- Keep diagnostic/internal-only strings out of scope unless they are shown in
  the UI.
- Preserve existing i18n completeness tests.

## 4. Non-Goals

- Adding new languages.
- Introducing pluralization frameworks.
- Localizing logs or developer diagnostics.
- Rewriting existing UI copy beyond what is required to catalog it.

## 5. Proposed Design

Audit the app crate for string literals in:

- button text and titles
- aria labels
- placeholders
- dialog labels
- toast/error summaries
- compact visible control labels

For each user-visible string:

1. Add a `MessageKey`.
2. Add English text in `i18n/en.rs`.
3. Add Japanese text in `i18n/ja.rs`.
4. Replace the literal with `t(locale(), MessageKey::...)`.

When text needs interpolation, keep formatting close to the component until a
small typed helper is justified.

## 6. Acceptance Criteria

- No obvious user-visible English literals remain in Dioxus components.
- Existing i18n completeness tests pass.
- `cargo test -p app` passes.
- Japanese catalog contains entries for every new key.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Over-cataloging internal strings creates noise | Only catalog strings rendered to users or assistive technology. |
| Japanese translations lag | Keep the RFC scoped and require every new key to have a Japanese entry. |
| Dynamic strings become awkward | Use small formatting helpers where repeated. |
