---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented (2.0.0-beta.9)
baseline: PDF Tile Viewer 2.0.0-beta.8
---

# RFC 022 — Contributor Test Layout Documentation Correction

## 1. Summary

Correct contributor documentation that still recommends central
`src/tests.rs` or `src/tests/` test layout. The project now requires
co-located module tests such as `foo/tests.rs`.

## 2. Motivation

The beta.6 cleanup moved library/service tests to the repository's preferred
co-located structure. The contributor documentation now conflicts with both
the project working agreement and the actual source tree. This can cause new
RC fixes to reintroduce the older layout.

## 3. Goals

- Update contributor docs to state the co-located test rule.
- Clarify that crate-level integration tests remain valid under `tests/` when
  they exercise public crate behavior or real external resources.
- Keep examples aligned with the current source tree.

## 4. Non-Goals

- Moving any tests.
- Changing Rust's standard integration-test layout.
- Adding a lint or custom enforcement tool.

## 5. Proposed Design

Replace the stale code-style bullet with:

- Unit tests live next to the module they test:
  `src/foo.rs` may use `src/foo/tests.rs` plus `#[cfg(test)] mod tests;`.
- Avoid central `src/tests.rs` for ordinary module tests.
- Use crate-level `tests/` only for integration tests that intentionally
  exercise the crate from the outside, such as PDFium smoke tests.

## 6. Acceptance Criteria

- `docs/src/contributors/dev.md` no longer recommends central `src/tests.rs`
  for ordinary unit tests.
- Contributor docs mention the valid integration-test exception.
- `mdbook build docs` passes.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Rule is read as banning all `tests/` directories | Explicitly distinguish module co-location from crate-level integration tests. |
| Future contributors miss the pattern | Include a concrete `src/foo.rs` → `src/foo/tests.rs` example. |
