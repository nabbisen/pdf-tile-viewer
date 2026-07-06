# RFC Index — PDF Tile Viewer

Implemented RFCs live in `rfcs/done/`. Proposed 2.0 RC follow-up work lives in
`rfcs/proposed/`. The lifecycle policy is in
`rfcs/done/000-rfc-lifecycle-policy.md`.

---

## Policy

| ID | Title | Status |
|----|-------|--------|
| 000 | [RFC Lifecycle Policy](done/000-rfc-lifecycle-policy.md) | Implemented |

---

## Proposed 2.0 RC Follow-Ups

| ID | Title | Status |
|----|-------|--------|
| 021 | [Hardcoded UI String i18n Sweep](proposed/021-i18n-hardcoded-ui-string-sweep.md) | Proposed |

---

## Developer Handoffs

| RFC | Handoff |
|-----|---------|
| 023 | [Single-Page PDF Text Selection Layer](handoffs/023-pdf-text-selection-layer/implementation-handoff.md) |

---

## Migration RFCs — all implemented

| ID | Title | Milestone |
|----|-------|-----------|
| 001 | [Dioxus Application Shell and Workspace Layout](done/001-dioxus-application-shell-and-workspace-layout.md) | M2 (alpha.1) |
| 002 | [Platform File Picker, Drag-Drop, File Manager](done/002-platform-file-picker-drag-drop-and-file-manager-integration.md) | M5 (alpha.3) |
| 003 | [PDFium Loader and Bundled Dynamic Distribution](done/003-pdfium-loader-and-bundled-dynamic-distribution.md) | M0 (alpha.1) |
| 004 | [Document Session Model](done/004-document-session-model.md) | M1 (alpha.1) |
| 005 | [Single-Page Render Vertical Slice](done/005-single-page-render-vertical-slice.md) | M3 (alpha.1) |
| 006 | [Tile Layout Engine](done/006-tile-layout-engine.md) | M4 (alpha.2) |
| 007 | [Lazy Render Queue and Cache](done/007-lazy-render-queue-and-cache.md) | M4 / M9 (alpha.2 / beta.2) |
| 008 | [Viewer Controls and Persistent Settings](done/008-viewer-controls-and-persistent-settings.md) | M5 (alpha.2/3) |
| 009 | [Dashboard and Session History UX](done/009-dashboard-and-session-history-ux.md) | M2/M5 (alpha.1/3) |
| 010 | [Search Result Model and Page Markers](done/010-search-result-model-and-page-markers.md) | M6 (alpha.4) |
| 011 | [Search Highlight Coordinates and Rendering](done/011-search-highlight-coordinates-and-rendering.md) | M6 (alpha.4) |
| 012 | [Zoom Overlay](done/012-zoom-overlay.md) | M7 (beta.1) |
| 013 | [Zen Mode and Keyboard Accessibility](done/013-zen-mode-and-keyboard-accessibility.md) | M5/M7 (alpha.3/beta.1) |
| 014 | [Cross-Platform Packaging and Release Artifacts](done/014-cross-platform-packaging-and-release-artifacts.md) | M8 (beta.1) |
| 015 | [CI Smoke Tests for Rendering and Search](done/015-ci-smoke-tests-for-rendering-and-search.md) | M0/M9 (alpha.1/beta.2) |
| 016 | [Security and Privacy Hardening](done/016-security-and-privacy-hardening.md) | M5/M9 (alpha.3/beta.2) |
| 017 | [Internationalization and Language Resources](done/017-internationalization-and-language-resources.md) | M2 (alpha.1) |
| 018 | [Package Artifact Smoke Test](done/018-package-artifact-smoke-test.md) | RC gate |
| 019 | [FAQ Correction for Full-Path Window Title Setting](done/019-faq-title-path-setting-doc-correction.md) | RC docs |
| 020 | [Release PDFium Bundling Documentation Correction](done/020-release-pdfium-bundling-doc-correction.md) | RC docs |
| 022 | [Contributor Test Layout Documentation Correction](done/022-contributor-test-layout-doc-correction.md) | RC docs |
| 023 | [Single-Page PDF Text Selection Layer](done/023-pdf-text-selection-layer.md) | 2.0.0-beta.8 |

---

## Future themes

See `docs/design/RFC-FUTURE-notes.md` for post-2.0 ideas (static PDFium
linking, password-protected PDFs, link navigation, native renderer
investigation, and offline WASM target).
