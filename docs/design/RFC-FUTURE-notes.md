---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Draft for implementation planning
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# Future RFC Notes — Deferred Topics

## 1. Purpose

This document records future RFC themes that should not block the first Dioxus + PDFium migration. They are intentionally deferred until the core migrated app is stable.

## Future RFC-F01 — Static PDFium Linking

### Goal

Evaluate and possibly implement fully static PDFium linking.

### Reason to Defer

Static linking requires target-specific PDFium static archives and platform-specific linker work. The first migrated release should prioritize bundled dynamic PDFium because it is simpler and proves the product pipeline earlier.

### Future Design Questions

- Which static PDFium build source is trusted?
- How are C++ runtime and platform libraries linked?
- Does static linking materially improve distribution enough to justify complexity?
- Can CI build or verify static artifacts for every target?

## Future RFC-F02 — Password-Protected PDF Support

### Goal

Add password prompt and encrypted PDF opening workflow.

### Reason to Defer

Password-protected PDFs complicate the document-opening lifecycle and error states. The existing migration should first restore the ordinary local PDF tile viewer workflow.

### Future Design Questions

- How are passwords entered and cleared from memory?
- Should passwords ever be remembered? Recommended answer: no.
- How does failed password retry behave?

> Future RFC-F03, text selection, has been implemented as RFC 023 in
> `2.0.0-beta.8`.

## Future RFC-F04 — Link and Outline Navigation

### Goal

Support PDF links, bookmarks, table-of-contents navigation, and internal
destinations.

### Reason to Defer

Useful, but not central to the tile overview workflow. It also introduces
external action/security policy decisions.

## Future RFC-F05 — Native Renderer / Non-WebView UI Investigation

### Goal

Re-evaluate a no-WebView UI path using Iced, Slint, egui, a Dioxus native
renderer path, or another toolkit.

### Reason to Defer

The approved migration target is Dioxus Desktop. Removing WebView is a
separate strategic decision and should not be mixed into this migration.

## Future RFC-F06 — Offline Web App Target (Dioxus Web / WASM)

### Goal

Offer PDF Tile Viewer as an offline-capable web app built from the same Dioxus component tree, in addition to the local desktop GUI.

### Reason to Defer

The desktop migration must land first. The web target requires a PDF engine backend that runs in the browser (PDFium compiled to WebAssembly, or another engine behind the existing PDF service boundary), a browser-appropriate file intake model (File System Access API / file input instead of native dialogs), and a different settings/storage backend.

### Standing Constraint on Current RFCs

To keep this target reachable, the current migration must respect the dependency rules in RFC 001: `domain` stays free of platform I/O, UI components call services (never PDFium directly), and platform integration stays in swappable service implementations. These rules are already mandated; this note records *why* they also matter strategically.

### Future Design Questions

- PDFium-to-WASM build pipeline and size budget, versus adopting a second engine behind the PDF service trait.
- Offline storage of settings (IndexedDB/localStorage) behind the settings service facade.
- Whether desktop and web share one `app` crate with target-gated services or split into thin per-target binary crates.
