# PDF Tile Viewer Dioxus + PDFium Migration Roadmap

**Date:** 2026-06-07  
**Document type:** Migration roadmap and risk gates  
**Depends on:** `01_goal_state_external_design.md`

---

## 1. Roadmap Summary

The migration should be executed as a sequence of controlled milestones. The project should not start with a full rewrite of every screen. The first priority is proving the new rendering pipeline because the target architecture replaces PDF.js with Rust/PDFium image-tile rendering.

Recommended sequence:

```text
M0  Architecture acceptance and branch setup
M1  Dioxus shell and basic dashboard
M2  PDFium binding and bundled dynamic packaging spike
M3  Single-page render vertical slice
M4  Tile grid layout and lazy render queue
M5  Viewer controls and settings
M6  Search result model and page markers
M7  Exact highlight overlay / highlighted render
M8  Zoom overlay and Zen mode
M9  Packaging, CI, release hardening
M10 Cutover and legacy cleanup
```

---

## 2. Migration Principles

1. **Renderer first, polish later**
   - The critical unknown is not Dioxus UI syntax. It is PDFium rendering, image transport, cache behavior, and cross-platform packaging.

2. **Vertical slices over large parallel rewrite**
   - Each milestone should produce a runnable app.
   - Each milestone should have a visible acceptance test.

3. **Keep old app releasable until cutover**
   - The Tauri/Svelte version should remain available until the Dioxus version satisfies core workflows.

4. **Do not preserve old technical boundaries unnecessarily**
   - Do not recreate Tauri-style command IPC inside Dioxus.
   - Do not keep PDF.js except as a short-lived spike if needed.

5. **Treat PDFium packaging as product functionality**
   - If the app cannot find/load PDFium in packaged form, the migration is not done.

---

## 3. Milestone Details

## M0 — Architecture Acceptance and Branch Setup

### Purpose

Prepare a migration branch and lock the target direction.

### Tasks

- Accept ADR-000.
- Create migration branch or new workspace.
- Establish Rust workspace layout.
- Decide temporary feature flags:
  - `legacy-tauri` if keeping old code in tree.
  - `dioxus-next` for target app.
- Add fixture PDFs for tests.

### Deliverables

- Accepted external design.
- Migration branch.
- Empty but compilable workspace structure.

### Exit Criteria

- `cargo check` passes for the new workspace skeleton.
- Project owner approves migration scope.

---

## M1 — Dioxus Shell and Basic Dashboard

### Purpose

Replace the shell entry point and prove Dioxus Desktop app startup.

### Tasks

- Create Dioxus Desktop app.
- Implement dashboard layout.
- Implement file picker abstraction.
- Implement drag/drop abstraction if Dioxus platform support is ready; otherwise temporarily implement choose-file first.
- Implement toast and loader primitives.
- Implement session-only history model.

### Deliverables

- Runnable Dioxus desktop app.
- Dashboard screen.
- Choose-file flow that captures path but may not render yet.

### Exit Criteria

- User can start app and choose a PDF path.
- UI can show selected file name and path.
- No Tauri/Svelte code is required for this milestone.

---

## M2 — PDFium Binding and Bundled Dynamic Packaging Spike

### Purpose

Prove that PDFium can be loaded from an app-controlled bundled location on target platforms.

### Tasks

- Implement `PdfiumLoader` abstraction.
- Support development fallback to system/current directory only under explicit dev mode.
- Implement packaged dynamic path lookup.
- Add CI script to fetch or stage PDFium binary.
- Add runtime diagnostic for PDFium version/path.

### Deliverables

- `pdf_engine::pdfium_loader`.
- Small command/test that loads PDFium and reports success.
- Packaging note for each platform.

### Exit Criteria

- PDFium binds successfully on the primary development OS.
- CI or local package layout can run a PDFium smoke test.
- No user-visible `lib/pdfium/lib` requirement remains in the target path.

### Risk Gate

If bundled dynamic loading is unstable, do not proceed to full rendering. Fix packaging first.

---

## M3 — Single-Page Render Vertical Slice

### Purpose

Prove PDFium-rendered bitmap can be displayed by Dioxus.

### Tasks

- Open selected PDF through PDFium.
- Extract page count and first page geometry.
- Render page 1 at default scale.
- Display page 1 in Dioxus.
- Implement first image transport mechanism.

### Deliverables

- `open_document(path)` service.
- `render_page(document_id, page_index, scale)` service.
- Single-page preview screen.

### Exit Criteria

- User can choose PDF and see page 1 rendered by PDFium.
- App remains responsive during render.
- Render failure displays a recoverable error.

### Risk Gate

Image transport must be judged acceptable enough to proceed. If data URI is used, document it as a spike-only implementation.

---

## M4 — Tile Grid Layout and Lazy Render Queue

### Purpose

Rebuild the core tile-viewer experience.

### Tasks

- Implement layout engine.
- Implement auto pages-per-row.
- Implement fixed pages-per-row.
- Implement render queue with priority.
- Implement cache key and memory cache.
- Implement visible-window/prefetch policy.
- Implement stale generation suppression.

### Deliverables

- Multi-page tile grid.
- Progressive rendering.
- Bounded render cache.

### Exit Criteria

- 100-page fixture PDF can be opened without rendering all pages immediately.
- Scrolling progressively renders pages.
- Scale changes do not freeze UI.
- Memory usage remains bounded under default settings.

### Risk Gate

If performance is poor, optimize render queue and image transport before adding search/zoom features.

---

## M5 — Viewer Controls and Settings

### Purpose

Restore the key controls from the current app.

### Tasks

- Scale slider.
- Ctrl + mouse wheel scale if reliably available.
- Page number visibility toggle.
- Fixed/auto pages-per-row control.
- Jump-to-page control.
- Settings schema and persistence.
- Legacy settings migration.

### Deliverables

- Viewer control rail.
- Versioned settings file.
- Settings migration tests.

### Exit Criteria

- User can adjust and persist viewer controls.
- Restart app preserves relevant settings.
- Jump-to-page scrolls target page and marks it briefly.

---

## M6 — Search Result Model and Page Markers

### Purpose

Restore search at the page-marker level without exact rectangle highlight first.

### Tasks

- Implement PDFium text search service.
- Return matched pages and counts.
- Format compact matched page string.
- Mark matched tiles.
- Clear search without reopening document.

### Deliverables

- Search panel.
- Search result model.
- Matched-page tile marker.

### Exit Criteria

- Search known fixture term returns expected matched pages.
- No-match search gives clear feedback.
- Clear search removes markers.
- Search does not mutate PDF bytes.

---

## M7 — Exact Highlight Overlay or Highlighted Render

### Purpose

Restore visual search highlights at text location level.

### Tasks

- Capture match rectangles from PDFium.
- Implement coordinate transform tests.
- Choose overlay rectangles or baked highlights.
- Render exact highlights in tile and/or zoom overlay.

### Deliverables

- Highlight model.
- Coordinate transform module.
- Highlight rendering implementation.

### Exit Criteria

- Search hits visually align with text on fixture PDFs.
- Rotation and common page sizes are handled.
- Highlights scale correctly with tile scale.

### Risk Gate

If exact rectangle alignment is unreliable, keep page-level markers for first release and mark exact highlights as beta.

---

## M8 — Zoom Overlay and Zen Mode

### Purpose

Restore secondary viewer workflows.

### Tasks

- Implement zoom overlay.
- Implement overlay page navigation.
- Implement independent zoom overlay scale.
- Implement background lock.
- Implement Zen mode.
- Add keyboard support.

### Deliverables

- Zoom overlay screen component.
- Zen mode.
- Keyboard flow.

### Exit Criteria

- User can open a tile in zoom view.
- Previous/next and Escape work.
- Zen mode is reversible.
- Accessibility labels exist for major controls.

---

## M9 — Packaging, CI, and Release Hardening

### Purpose

Make the migrated app distributable.

### Tasks

- Release artifacts for primary targets.
- Bundle PDFium dynamic library.
- Verify packaged PDFium binding.
- Add fixture render/search smoke tests.
- Update README and user documentation.
- Document unsigned-app warnings if still applicable.

### Deliverables

- CI release workflow.
- Cross-platform package smoke tests.
- User-facing release documentation.

### Exit Criteria

- A clean machine/user can launch app without manually installing PDFium.
- Fixture PDF opens and renders in packaged app.
- Release artifact structure is documented.

---

## M10 — Cutover and Legacy Cleanup

### Purpose

Finalize the migration.

### Tasks

- Decide version number for Dioxus release.
- Archive or remove Tauri/Svelte implementation.
- Remove PDF.js dependency from final branch.
- Remove legacy external PDFium path from production code.
- Keep migration notes for users.

### Deliverables

- Dioxus-based release branch.
- Legacy cleanup commit.
- Migration changelog.

### Exit Criteria

- Final source tree does not require Node/Svelte/Vite/PDF.js for app runtime.
- Production app uses internally managed PDFium.
- Core v1.1.2 workflows are restored or explicitly documented as deferred.

---

## 4. Risk Register

| Risk | Severity | Mitigation |
|---|---:|---|
| Dioxus Desktop WebView behavior differs across OS | Medium | Keep UI simple; test Windows/Linux/macOS early. |
| Image transport causes high memory use | High | Start with data URI spike only; move to cache/custom protocol before release. |
| PDFium dynamic packaging is brittle | High | Make M2 a required gate before heavy UI work. |
| Static PDFium linking takes too long | Medium | Defer static linking. Use bundled dynamic first. |
| Search highlight coordinates misalign | Medium | Unit-test coordinate transforms; ship page-level markers first if needed. |
| Large PDFs freeze UI | High | Lazy render queue, generation suppression, cache budget. |
| Removing PDF.js loses text selection/link behavior | Low for current scope | Declare non-goal; consider future RFC. |
| Dioxus version churn | Medium | Pin version and avoid advanced/unstable APIs in core design. |

---

## 5. Rollback Points

| Point | Rollback strategy |
|---|---|
| After M1 | Abandon Dioxus shell if startup/platform basics fail. |
| After M2 | Keep Tauri/Svelte and only improve PDFium packaging if PDFium bundling fails. |
| After M3 | Reconsider keeping PDF.js temporarily if image display path is unacceptable. |
| After M4 | Optimize image transport/cache or reconsider native GUI toolkit if WebView image path is too slow. |
| After M6 | Release with page-level search markers if exact highlights are not reliable. |

---

## 6. Recommended Immediate Next Step

Create RFCs for M1–M3 only first:

1. Dioxus app shell and dashboard.
2. PDFium loader and bundled dynamic packaging spike.
3. Single-page PDFium render vertical slice.

Do not create detailed RFCs for search, zoom, or static linking until M3 proves the core rendering path.
