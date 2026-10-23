# RFC Index — PDF Tile Viewer

Implemented RFCs live in `rfcs/done/`; accepted RFCs (design settled,
implementation may start) in `rfcs/accepted/`; proposed follow-up work in
`rfcs/proposed/`. The lifecycle policy is in
`rfcs/done/000-rfc-lifecycle-policy.md`.

---

## Policy

| ID | Title | Status |
|----|-------|--------|
| 000 | [RFC Lifecycle Policy](done/000-rfc-lifecycle-policy.md) | Implemented |

---

## Accepted — 2.0 Corrective Track (planned for 2.0.1)

Owner-accepted 2026-10-21; implementers may start. The release version,
schedule and gates are proposed by the architect and authorized by the owner.
Execution order is serial and recorded in the Developer Handoffs below.

| ID | Title | Status |
|----|-------|--------|
| 033 | [Keyboard Routing, Modal Layers, and Dismissal Model](accepted/033-keyboard-routing-and-dismissal-model.md) | Accepted — planned 2.0.1 |
| 034 | [Display-Space Page Geometry](accepted/034-display-space-page-geometry.md) | Accepted — planned 2.0.1 |
| 035 | [Document Session Lifecycle and Render Freshness](accepted/035-document-session-lifecycle-and-render-freshness.md) | Accepted — planned 2.0.1 |
| 036 | [Release Supply-Chain and Workflow Hardening](accepted/036-release-supply-chain-and-workflow-hardening.md) | Accepted — planned 2.0.1 |
| 037 | [User-Facing Error Presentation and Accessible Names](accepted/037-user-facing-error-presentation-and-accessible-names.md) | Accepted — planned 2.0.1 |
| 038 | [Corrective Documentation, Governance, and Repository Hygiene](accepted/038-corrective-documentation-governance-and-repository-hygiene.md) | Accepted — planned 2.0.1 |
| 039 | [Viewer Scale Shortcut Crash (Emergency Fix)](accepted/039-scale-shortcut-crash-hotfix.md) | Accepted — emergency, planned 2.0.1 |

## Proposed Follow-Ups

| ID | Title | Status |
|----|-------|--------|
| 027 | [Linux Dashboard Drag/Drop Reliability](proposed/027-linux-dashboard-drag-drop-reliability.md) | Proposed — post-2.0 candidate |
| 028 | [File Picker Initial Directory Memory](proposed/028-file-picker-initial-directory-memory.md) | Proposed — post-2.0 candidate |
| 029 | [Zoom Overlay Scale Preservation](proposed/029-zoom-overlay-scale-preservation.md) | Proposed — post-2.0 candidate |
| 030 | [Smooth Zoom-Scale Transition](proposed/030-smooth-zoom-scale-transition.md) | Proposed — post-2.0 candidate |
| 031 | [Settings Storage With `app-json-settings`](proposed/031-settings-storage-app-json-settings.md) | Proposed — v2.1 candidate |
| 032 | [Persistent Document History With Privacy Controls](proposed/032-persistent-document-history-privacy.md) | Proposed — v2.1 candidate |

---

## Developer Handoffs

| RFC | Handoff |
|-----|---------|
| 023 | [Single-Page PDF Text Selection Layer](handoffs/023-pdf-text-selection-layer/implementation-handoff.md) (historical) |
| 026 | [PDF Link and Outline Navigation](handoffs/026-pdf-link-outline-navigation/implementation-handoff.md) (historical) |
| 033 | [Keyboard Routing, Modal Layers, and Dismissal Model](handoffs/033-keyboard-routing-and-dismissal-model/evidence-run.md) (evidence run only; implementation handoff pending) |
| 039 | [Viewer Scale Shortcut Crash (Emergency Fix)](handoffs/039-scale-shortcut-crash-hotfix/implementation-handoff.md) · [release preparation](handoffs/039-scale-shortcut-crash-hotfix/release-preparation.md) |
| 034 | [Display-Space Page Geometry](handoffs/034-display-space-page-geometry/implementation-handoff.md) |
| 035 | [Document Session Lifecycle and Render Freshness](handoffs/035-document-session-lifecycle-and-render-freshness/implementation-handoff.md) |
| 036 | [Release Supply-Chain and Workflow Hardening](handoffs/036-release-supply-chain-and-workflow-hardening/implementation-handoff.md) |
| 038 | [Corrective Documentation, Governance, and Repository Hygiene](handoffs/038-corrective-documentation-governance-and-repository-hygiene/implementation-handoff.md) (slices A and B′) |

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
| 018 | [Package Artifact Smoke Test](done/018-package-artifact-smoke-test.md) | 2.0.0-beta.9 |
| 019 | [FAQ Correction for Full-Path Window Title Setting](done/019-faq-title-path-setting-doc-correction.md) | 2.0.0-beta.9 |
| 020 | [Release PDFium Bundling Documentation Correction](done/020-release-pdfium-bundling-doc-correction.md) | 2.0.0-beta.9 |
| 021 | [Hardcoded UI String i18n Sweep](done/021-i18n-hardcoded-ui-string-sweep.md) | 2.0.0-beta.9 |
| 022 | [Contributor Test Layout Documentation Correction](done/022-contributor-test-layout-doc-correction.md) | 2.0.0-beta.9 |
| 023 | [Single-Page PDF Text Selection Layer](done/023-pdf-text-selection-layer.md) | 2.0.0-beta.8 |
| 024 | [Encrypted PDF Detection Hardening](done/024-encrypted-pdf-detection-hardening.md) | 2.0.0-beta.10 |
| 025 | [Password-Protected PDF Opening Workflow](done/025-password-protected-pdf-opening.md) | 2.0.0-beta.11 |
| 026 | [PDF Link and Outline Navigation](done/026-pdf-link-outline-navigation.md) | 2.0.0-beta.11 follow-up |

---

## Future themes

See `docs/design/RFC-FUTURE-notes.md` for deferred candidates, including
post-2.0 ideas and possible v2.1 themes.
