# PDF Tile Viewer Dioxus + PDFium RFC Roadmap

**Date:** 2026-06-07  
**Document type:** RFC theme breakdown  
**Status:** Ready to use after external design approval

---

## 1. RFC Sequencing Policy

RFCs should be written in small groups. The first group should unblock the architectural unknowns. Later RFCs should refine product behavior after the rendering path is proven.

Recommended groups:

```text
Group A: Foundation and feasibility
Group B: Core viewer restoration
Group C: Search, zoom, and UX parity
Group D: Packaging and release hardening
Group E: Future enhancements
```

---

## 2. Group A — Foundation and Feasibility

### RFC-001 — Dioxus Application Shell and Workspace Layout

**Goal:** Establish the new Rust workspace and Dioxus Desktop entry point.

**Scope:**

- Workspace structure.
- App crate.
- Domain/service module split.
- Dashboard screen skeleton.
- Toast/loader primitives.
- Build commands.

**Out of scope:**

- PDF rendering.
- Search.
- Production packaging.

**Key design questions:**

- Which Dioxus version is pinned?
- What crates are introduced?
- How are UI state providers structured?

---

### RFC-002 — Platform File Picker, Drag-Drop, and File Manager Integration

**Goal:** Replace Tauri dialog, drag/drop, and file-manager utilities.

**Scope:**

- Choose file.
- Drop file.
- File validation before PDF engine load.
- Open containing folder.
- User-facing error messages.

**Out of scope:**

- Multi-file tabs.
- Persistent recent files.

---

### RFC-003 — PDFium Loader and Bundled Dynamic Distribution

**Goal:** Make PDFium an internally managed app dependency.

**Scope:**

- PDFium loading strategy.
- Bundled dynamic layout.
- Development fallback policy.
- Platform-specific paths.
- CI smoke test.
- Error reporting.

**Out of scope:**

- Static linking.
- PDF rendering cache.

**Decision expected:**

- Whether to bundle dynamic library directly or embed/extract it.

---

### RFC-004 — Document Session Model

**Goal:** Define how PDFs are opened and represented.

**Scope:**

- `DocumentId`.
- `DocumentSource`.
- Metadata extraction.
- Page count.
- Page geometry.
- Document lifetime.
- Close/reopen behavior.

**Out of scope:**

- Page bitmap rendering.
- Search.

---

### RFC-005 — Single-Page Render Vertical Slice

**Goal:** Render one page with PDFium and display it in Dioxus.

**Scope:**

- Render API.
- Image format.
- Initial image transport.
- Render error states.
- Unit/integration fixture test.

**Out of scope:**

- Full tile grid.
- Cache eviction.

---

## 3. Group B — Core Viewer Restoration

### RFC-006 — Tile Layout Engine

**Goal:** Rebuild adaptive tile layout.

**Scope:**

- Page geometry to tile sizes.
- Auto pages-per-row.
- Fixed pages-per-row.
- Mixed page sizes.
- Row model.
- Jump-to-page scroll target model.

---

### RFC-007 — Lazy Render Queue and Cache

**Goal:** Render visible/near-visible pages efficiently.

**Scope:**

- Render priorities.
- Render generation.
- Cache key.
- Memory budget.
- Eviction policy.
- Placeholder states.

**Important:** This RFC is mandatory before large-PDF support is considered acceptable.

---

### RFC-008 — Viewer Controls and Persistent Settings

**Goal:** Restore scale, page numbers, pages-per-row, and settings persistence.

**Scope:**

- Control rail UI.
- Scale bucket model.
- Ctrl + wheel behavior.
- Settings schema.
- Legacy settings migration.
- Privacy defaults.

---

### RFC-009 — Dashboard and Session History UX

**Goal:** Finalize dashboard workflow.

**Scope:**

- Drop zone.
- Choose file.
- Runtime history.
- Empty/error states.
- Optional persisted recent files policy.

---

## 4. Group C — Search, Zoom, and UX Parity

### RFC-010 — Search Result Model and Page Markers

**Goal:** Implement non-mutating PDFium search.

**Scope:**

- Search query model.
- Minimum length.
- Page result list.
- Compact page-range formatting.
- Matched page tile marker.
- Clear search.

**Out of scope:**

- Exact rectangle highlight.

---

### RFC-011 — Search Highlight Coordinates and Rendering

**Goal:** Render exact search highlights.

**Scope:**

- PDFium rectangle extraction.
- Coordinate transformation.
- Overlay rectangles or baked highlighted render.
- Rotation handling.
- Fixture tests.

---

### RFC-012 — Zoom Overlay

**Goal:** Restore focused page inspection.

**Scope:**

- Open/close overlay.
- Previous/next page.
- Independent scale.
- Background lock.
- Optional transparency.
- Keyboard support.

---

### RFC-013 — Zen Mode and Keyboard Accessibility

**Goal:** Restore distraction-free viewing and keyboard paths.

**Scope:**

- Zen mode UI state.
- Escape/reveal behavior.
- Keyboard shortcuts.
- Focus indicators.
- Accessible labels.

---

## 5. Group D — Packaging and Release Hardening

### RFC-014 — Cross-Platform Packaging and Release Artifacts

**Goal:** Make the app distributable.

**Scope:**

- Windows artifact.
- Linux artifact.
- macOS artifact.
- PDFium bundling verification.
- Unsigned-app documentation.
- Release notes.

---

### RFC-015 — CI Smoke Tests for Rendering and Search

**Goal:** Prevent release regressions.

**Scope:**

- Fixture PDFs.
- PDFium bind test.
- Page render test.
- Search result test.
- Packaged artifact structure test.

---

### RFC-016 — Security and Privacy Hardening

**Goal:** Make local-file handling and native library loading safe by default.

**Scope:**

- PDFium load path policy.
- No arbitrary current-directory loading in production.
- File path privacy.
- Recent files privacy.
- PDF external action policy.

---

## 6. Group E — Future Enhancements

These RFCs should be postponed until the migrated app is stable.

### Future RFC-F01 — Static PDFium Linking

**Goal:** Evaluate and implement fully static PDFium linking.

**Reason to defer:** Requires target-specific static archives and platform-specific linker work.

---

### Future RFC-F02 — Password-Protected PDF Support

**Goal:** Add password prompt and encrypted PDF opening.

**Reason to defer:** Not part of current v1.1.2 behavior and complicates open-document lifecycle.

---

### Future RFC-F03 — Text Selection Layer

**Goal:** Provide selectable text over page images.

**Reason to defer:** Requires text layout extraction, coordinate mapping, selection UX, and accessibility decisions.

---

### Future RFC-F04 — Link and Outline Navigation

**Goal:** Support PDF links, bookmarks, and table-of-contents navigation.

**Reason to defer:** Useful but not central to tile overview workflow.

---

### Future RFC-F05 — Native Renderer / Non-WebView UI Investigation

**Goal:** Re-evaluate Iced, Slint, egui, or Dioxus native renderer paths.

**Reason to defer:** The approved migration is Dioxus Desktop; no-WebView is a separate strategic decision.

---

## 7. Recommended Next RFC Batch

Start with only these three RFCs:

1. **RFC-001 — Dioxus Application Shell and Workspace Layout**
2. **RFC-003 — PDFium Loader and Bundled Dynamic Distribution**
3. **RFC-005 — Single-Page Render Vertical Slice**

These three RFCs prove whether the target architecture is viable before the team spends effort on full UI parity.
