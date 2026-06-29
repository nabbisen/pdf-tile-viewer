---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Implemented
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# RFC-015 — CI Smoke Tests for Rendering and Search

## 1. Summary

This RFC defines CI smoke tests that protect the migrated app from regressions in PDFium loading, document opening, page rendering, search, highlight transformation, and packaged artifact structure.

## 2. Motivation

The migration changes the critical runtime dependency from PDF.js-in-WebView rendering to PDFium-native rendering. CI must catch failures that normal Rust unit tests may miss, especially missing dynamic libraries, platform path issues, and fixture rendering/search regressions.

## 3. Goals

- Add fixture PDFs.
- Test PDFium binding.
- Test document open and metadata extraction.
- Test page render output dimensions.
- Test text search page results.
- Test package structure and PDFium presence.

## 4. Non-Goals

- Pixel-perfect rendering approval for every platform.
- Full GUI automation for all workflows.
- Performance benchmark suite in the first version.

## 5. Test Layers

```text
Unit tests
├── domain layout calculation
├── page range formatting
├── settings migration
└── coordinate transforms

Integration tests
├── bind PDFium
├── open fixture PDF
├── render page image
└── search fixture term

Package smoke tests
├── artifact contains app binary
├── artifact contains PDFium library
├── packaged app/test helper binds PDFium
└── fixture PDF opens in packaged layout
```

## 6. Fixture PDF Policy

Fixture PDFs should be small, redistributable, and deterministic.

Minimum fixtures:

| Fixture | Purpose |
|---|---|
| `single-page-basic.pdf` | Open/render smoke test. |
| `multi-page-search.pdf` | Search page result test. |
| `rotated-page.pdf` | Highlight coordinate rotation test. |
| `mixed-page-sizes.pdf` | Tile layout and geometry test. |

If redistributable fixture PDFs cannot be sourced, generate them in repository tooling and commit only if license-safe.

## 7. Smoke Test Commands

Recommended conceptual commands:

```text
cargo test --workspace
cargo test -p pdf_engine --test pdfium_bind
cargo test -p pdf_engine --test render_fixture
cargo test -p pdf_engine --test search_fixture
cargo xtask package-smoke --artifact <path>
```

`xtask` is optional but recommended for release-specific checks.

## 8. Render Test Criteria

A render smoke test should verify:

- Render returns success.
- Pixel width and height are greater than zero.
- Output byte length is greater than a small threshold.
- The result belongs to expected document id/generation.

It should not assert exact pixel bytes unless the project intentionally pins PDFium version and platform rendering behavior.

## 9. Search Test Criteria

A search smoke test should verify:

- Known query returns expected matched page numbers.
- No-match query returns zero results.
- Search does not mutate the file.
- Stale generation result is rejectable.

## 10. Package Smoke Test Criteria

Package test should verify:

```text
artifact unpacked
required binary exists
required PDFium library exists
license/notices exist
smoke helper binds PDFium from packaged location
fixture document opens
page 1 renders
```

## 11. CI Matrix

Recommended staged matrix:

1. Primary developer OS: full unit + integration + package smoke.
2. Other target OS: unit + PDFium bind smoke.
3. Release branches: full matrix before publishing.

## 12. Acceptance Criteria

- CI fails when PDFium cannot be loaded.
- CI fails when fixture page cannot render.
- CI fails when fixture search result changes unexpectedly.
- Package smoke test catches missing PDFium binary.
- Layout and coordinate transform tests do not require GUI.

## 13. Risks

| Risk | Mitigation |
|---|---|
| CI cannot access PDFium binaries reliably | Cache/stage versioned binaries or build artifacts. |
| Pixel output differs across OS | Test dimensions and search, not exact pixels initially. |
| Fixture PDFs introduce license issues | Generate simple fixtures or use clearly licensed samples. |
