---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented (2.0.0-beta.9)
baseline: PDF Tile Viewer 2.0.0-beta.8
---

# RFC 020 — Release PDFium Bundling Documentation Correction

## 1. Summary

Update user documentation that describes bundled PDFium as future behavior.
Release artifacts now bundle PDFium and users should not install it separately.

## 2. Motivation

The FAQ currently says release packaging "will bundle" PDFium. That was correct
before release packaging landed, but it is now stale and weakens the support
story for missing-PDFium errors.

## 3. Goals

- Describe PDFium as bundled in official release artifacts.
- Tell users to keep the extracted archive layout intact.
- Avoid telling packaged users to run development scripts or set development
  environment variables.
- Keep development-source instructions separate.

## 4. Non-Goals

- Implementing an in-app PDFium downloader.
- Changing the PDFium provider or release tag policy.
- Adding static PDFium linking.

## 5. Proposed Design

Update FAQ/install/troubleshooting wording:

- Official release archives include the platform PDFium dynamic library under
  `resources/pdfium/<platform>/`.
- Users should launch the app from the extracted archive and keep `bin/` and
  `resources/` together.
- Source builders may use `ci/fetch-pdfium.sh` or
  `PDF_TILE_VIEWER_PDFIUM_DIR`; packaged users should not need either.

## 6. Acceptance Criteria

- No new-user docs describe release PDFium bundling as future-only.
- Missing-PDFium troubleshooting points to re-downloading/re-extracting the
  official archive, not development scripts.
- `mdbook build docs` passes.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Source-builder and packaged-user instructions get mixed again | Keep separate "release artifact" and "build from source" sections. |
| Users move only the executable | State explicitly that `bin/` and `resources/` must remain together. |
