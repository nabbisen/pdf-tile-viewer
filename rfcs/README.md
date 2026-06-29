# PDF Tile Viewer RFCs

This directory follows the lifecycle policy defined in
[RFC 000](./done/000-rfc-lifecycle-policy.md): the folder an RFC lives in is
the source of truth for its state (`proposed/` → open for review and
implementation, `done/` → implemented, `archive/` → withdrawn or superseded).
States are exactly those defined by RFC 000: Draft, Proposed, Implemented,
Withdrawn, Superseded. Numbers are stable and never reused.

RFCs 001–017 cover the v2 migration from Tauri + Svelte + PDF.js to
Dioxus Desktop + Rust/PDFium. They were derived from the approved goal-state
design package and the v1.1.2 reverse-engineered design (see
[`docs/design/`](../docs/design/)). The RFC theme list in the earlier
migration feasibility study (RFC-001…RFC-010 themes) is superseded by this
set; the goal-state package's `03_rfc_roadmap.md` records the mapping.

## Proposed

| ID | Title | Priority | Milestone |
|----|-------|----------|-----------|
| 001 | [Dioxus Application Shell and Workspace Layout](./proposed/001-dioxus-application-shell-and-workspace-layout.md) | P0 | M1 |
| 002 | [Platform File Picker, Drag-Drop, and File Manager Integration](./proposed/002-platform-file-picker-drag-drop-and-file-manager-integration.md) | P0 | M1 |
| 003 | [PDFium Loader and Bundled Dynamic Distribution](./proposed/003-pdfium-loader-and-bundled-dynamic-distribution.md) | P0 | M2 |
| 004 | [Document Session Model](./proposed/004-document-session-model.md) | P0 | M3 |
| 005 | [Single-Page Render Vertical Slice](./proposed/005-single-page-render-vertical-slice.md) | P0 | M3 |
| 006 | [Tile Layout Engine](./proposed/006-tile-layout-engine.md) | P0 | M4 |
| 007 | [Lazy Render Queue and Cache](./proposed/007-lazy-render-queue-and-cache.md) | P0 | M4 |
| 008 | [Viewer Controls and Persistent Settings](./proposed/008-viewer-controls-and-persistent-settings.md) | P1 | M5 |
| 009 | [Dashboard and Session History UX](./proposed/009-dashboard-and-session-history-ux.md) | P1 | M5 |
| 010 | [Search Result Model and Page Markers](./proposed/010-search-result-model-and-page-markers.md) | P1 | M6 |
| 011 | [Search Highlight Coordinates and Rendering](./proposed/011-search-highlight-coordinates-and-rendering.md) | P1 | M7 |
| 012 | [Zoom Overlay](./proposed/012-zoom-overlay.md) | P1 | M8 |
| 013 | [Zen Mode and Keyboard Accessibility](./proposed/013-zen-mode-and-keyboard-accessibility.md) | P1 | M8 |
| 014 | [Cross-Platform Packaging and Release Artifacts](./proposed/014-cross-platform-packaging-and-release-artifacts.md) | P0 | M9 |
| 015 | [CI Smoke Tests for Rendering and Search](./proposed/015-ci-smoke-tests-for-rendering-and-search.md) | P0 | M9 |
| 016 | [Security and Privacy Hardening](./proposed/016-security-and-privacy-hardening.md) | P0 | M9 |
| 017 | [Internationalization and Language Resources](./proposed/017-internationalization-and-language-resources.md) | P0 | M1+ |

## Implemented

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | repository adoption |

## Archive

| ID | Title | Reason |
|----|-------|--------|
| — | (empty) | — |

## Future RFC themes (not yet numbered)

Deferred topics that must not block the migration are recorded in
[`docs/design/RFC-FUTURE-notes.md`](../docs/design/RFC-FUTURE-notes.md):
static PDFium linking (F01), password-protected PDFs (F02), text selection
layer (F03), link/outline navigation (F04), native non-WebView renderer
investigation (F05), and the offline web app target via Dioxus web/WASM (F06).
They receive numbers when they enter `proposed/`.

## Conventions

- Filenames: `NNN-slug.md`, numbered from 001, numbers assigned at creation
  and never reused.
- The `status:` field in each RFC's front matter mirrors the folder and is
  updated in the same commit that moves the file.
- When an RFC moves, run `grep -rl "NNN-slug.md" rfcs/ docs/ README.md` and
  fix inbound references in the same commit.
- Implementation order follows `ROADMAP.md` (milestones M0–M10).
