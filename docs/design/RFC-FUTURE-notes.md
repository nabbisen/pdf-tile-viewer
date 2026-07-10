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

> Implemented as RFC 025 after the RFC 024 encrypted-PDF detection hardening
> shipped in `2.0.0-beta.10`.

### Goal

Add password prompt and encrypted PDF opening workflow.

### Resolution

RFC 025 added a password prompt and encrypted PDF opening workflow. Future work
may still revisit permission-aware copy/search behavior, certificate-encrypted
PDFs, or optional credential storage.

### Resolved Design Questions

- How are passwords entered and cleared from memory?
- Should passwords ever be remembered? Recommended answer: no.
- How does failed password retry behave?

> Future RFC-F03, text selection, has been implemented as RFC 023 in
> `2.0.0-beta.8`.

## Future RFC-F04 — Link and Outline Navigation

> Implemented as RFC 026 after the `2.0.0-beta.11` release point.

### Goal

Support PDF links, bookmarks, table-of-contents navigation, and internal
destinations.

### Resolution

RFC 026 added outline/bookmark navigation, same-document zoom-overlay link
navigation, and conservative copy-only handling for external URI actions.

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

## Future RFC-F07 — Linux Desktop Drag/Drop Reliability

> Proposed as [RFC 027](../../rfcs/proposed/027-linux-dashboard-drag-drop-reliability.md).

### Goal

Make dashboard file drag/drop reliable on Linux desktop sessions, or document
the exact toolkit/session combinations where it is unsupported.

### Reason to Defer

During RFC 026 manual QA on Linux WebKitGTK, opening through the file picker
worked, but dashboard file drop did not work under the observed Wayland session
or an `GDK_BACKEND=x11` comparison run. This is not specific to PDF link or
outline navigation, and the picker remains a reliable intake path.

### Future Design Questions

- Is the failure in Dioxus Desktop event routing, Wry/WebKitGTK drag data,
  compositor/file-manager behavior, or app event wiring?
- Can a stable `tao::WindowEvent::DroppedFile` path be wired through
  `use_wry_event_handler` or a custom desktop event handler without weakening
  the existing picker path?
- Should Linux docs describe drag/drop as best-effort until the upstream
  toolkit behavior is verified?

## Future RFC-F08 — File Picker Initial Directory Memory

> Proposed as [RFC 028](../../rfcs/proposed/028-file-picker-initial-directory-memory.md).

Planning status: deferred candidate, not part of RFC 026 or the 2.0.0 final
release gate.

### Goal

Remember the parent directory of the last successfully picked PDF in memory
during the current app session, and use it as the starting directory for the
next **Open PDF...** picker.

### Reason to Defer

This is a convenience improvement, not an RFC 026 blocker. It should be
designed with privacy in mind: session-memory-only behavior is likely safe by
default, while persistence would overlap with history/settings policy.

### Future Design Questions

- Should the remembered directory update only after a successful open?
- Should drag/drop, history reopen, and picker opens all update the directory,
  or picker opens only?
- Should this remain memory-only, or become a privacy-controlled setting?

## Future RFC-F09 — Toolbar Icon Alignment Polish

> Resolved as a small pre-2.0 polish fix after RFC 026.

Planning status: resolved; no separate RFC draft is needed unless future icon
system changes are proposed.

### Goal

Fix button icon alignment in the main viewer toolbar so icon-only controls are
visually centered and consistent across themes/platforms.

### Resolution

Icon-only toolbar buttons now use fixed square inline-flex centering.

### Deferred Design Questions

- Should icon buttons have a single fixed square size and flex centering?
- Are emoji glyphs causing platform-dependent alignment, and should these
  controls switch to a more stable icon strategy?

## Future RFC-F10 — Zoom Scale Persistence

> Proposed as [RFC 029](../../rfcs/proposed/029-zoom-overlay-scale-preservation.md).

Planning status: deferred candidate, not part of RFC 026 or the 2.0.0 final
release gate.

### Goal

Preserve the zoom overlay scale so users do not need to reset it every time
they open a page in the zoom view.

### Reason to Defer

This is useful, but it changes view-state behavior and should be decided
separately from PDF link/outline support.

### Future Design Questions

- Should zoom scale persist only during the current document session, across
  app sessions, or as a global default?
- Should tile-grid scale and zoom-overlay scale remain independent?
- Should page changes, document changes, or app restarts reset zoom scale?

## Future RFC-F11 — Smooth Zoom-Scale Transition

> Proposed as [RFC 030](../../rfcs/proposed/030-smooth-zoom-scale-transition.md).

Planning status: deferred candidate, not part of RFC 026 or the 2.0.0 final
release gate.

### Goal

Reduce the dizzying visual transition when changing zoom scale in the zoom
overlay.

### Reason to Defer

Smooth transitions may require renderer-level tradeoffs: immediate bitmap
rerendering is accurate but can feel jumpy, while animated interpolation may
temporarily show scaled raster content before the final render arrives.

### Future Design Questions

- Is CSS transform interpolation acceptable while waiting for a freshly
  rendered bitmap?
- Should the app crossfade between old and new renders?
- Can smooth transition coexist with accurate text/link/search overlay
  geometry without temporary mismatch?

## Future RFC-F12 — Settings Storage With `app-json-settings`

> Proposed as [RFC 031](../../rfcs/proposed/031-settings-storage-app-json-settings.md).

Planning status: deferred new-function candidate for v2.1 or later. It is not
part of the 2.0.0 final release gate.

### Goal

Evaluate replacing or wrapping the current settings storage with the
`app-json-settings` crate.

### Reason to Defer

Settings storage affects migration, compatibility, error recovery, and privacy.
It needs explicit design before implementation.

### Future Design Questions

- What migration path preserves existing user settings?
- Does the crate handle corrupt files, schema evolution, and platform-specific
  config directories in a way that matches current policy?
- Which settings are safe to persist by default?

## Future RFC-F13 — Persistent History With Privacy Controls

> Proposed as [RFC 032](../../rfcs/proposed/032-persistent-document-history-privacy.md).

Planning status: deferred new-function candidate for v2.1 or later. It is not
part of the 2.0.0 final release gate.

### Goal

Evaluate persistent document history, with explicit privacy controls and a
clear default policy.

### Reason to Defer

Current session-only history is privacy-safe. Persisting document paths can
reveal sensitive file names and locations, so this needs owner-approved UX and
storage policy before implementation.

### Future Design Questions

- Should persistent history be opt-in only?
- What clear/reset controls are required?
- Should entries store full paths, display names only, or redacted paths?
- Should private/incognito mode disable history writes?
