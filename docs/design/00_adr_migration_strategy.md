# ADR-000: Migration Strategy to Dioxus + Internally Managed PDFium

**Status:** Proposed for acceptance  
**Date:** 2026-06-07  
**Decision owner:** Project owner / product architect  
**Scope:** PDF Tile Viewer next major architecture

---

## 1. Context

PDF Tile Viewer v1.1.2 is a desktop PDF viewer focused on displaying PDF pages in a tile layout. The current application is implemented as:

- Tauri v2 shell.
- SvelteKit + TypeScript frontend.
- PDF.js frontend rendering.
- Rust backend commands for file reading, settings, window title management, file-manager integration, and PDFium-based text search.
- PDFium loaded from a visible external dynamic library location near the executable, with system-library fallback.

The desired next direction is:

- Dioxus Desktop for the app shell and UI components.
- Rust-first application state and domain logic.
- PDFium as the main PDF rendering/search engine.
- No PDF.js dependency in the final target.
- No user-visible external `lib/pdfium/lib` setup requirement.

The migration should preserve the current product value: a lightweight, focused PDF tile viewer that lets users quickly see many pages at once.

---

## 2. Decision

We will migrate toward a **Dioxus Desktop + Rust/PDFium image-tile rendering architecture**.

The final target is:

```text
User
  ↓
Dioxus Desktop UI
  ↓
Rust app state and services
  ↓
Serialized PDF engine worker
  ↓
PDFium
  ↓
Rendered page image cache + search/highlight model
  ↓
Dioxus tile grid
```

The first production-ready target will use **bundled dynamic PDFium managed by the application**, not mandatory static PDFium linking. Static linking remains a future hardening option after cross-platform packaging proves stable.

---

## 3. Decision Details

### 3.1 Dioxus replaces Tauri + Svelte, not WebView itself

Dioxus Desktop should be adopted to remove the TypeScript/Svelte layer and Tauri command IPC boundary. However, Dioxus Desktop still renders through the system WebView. Therefore, this migration should not be described as a no-WebView migration.

### 3.2 PDFium becomes the source of rendering truth

The current app uses two PDF engines:

- PDF.js for rendering.
- PDFium for search and highlight mutation.

The target architecture should use PDFium for:

- Document opening and validation.
- Page count and page geometry.
- Page rendering to bitmap/image.
- Text search.
- Search-match coordinates.
- Optional future metadata, outlines, links, and annotations.

This avoids PDF.js/PDFium divergence.

### 3.3 Search highlight should become an overlay model

The current search flow mutates a PDF buffer by adding highlight rectangles, then reloads the PDF in PDF.js. In the target design, search highlights should be represented as app-side overlay data, not by modifying the PDF document.

Rationale:

- Avoids accidental document mutation semantics.
- Avoids re-saving the full PDF for every search.
- Enables quick clear/search changes.
- Keeps render cache keys explicit through `highlight_revision`.

### 3.4 PDFium calls must be serialized

PDFium should be treated as a native engine boundary that does not support uncontrolled concurrent use. The target design will put PDFium behind a service/worker abstraction. Multiple UI events may request work, but engine calls are serialized or carefully controlled by the service.

### 3.5 Rendering must be lazy and cache-aware

The product’s tile view may show many pages. Rendering every page at every scale eagerly is unacceptable for large PDFs. The target design will render pages lazily based on viewport proximity, with prefetch margin, cancellation by generation ID, and bounded memory/disk cache.

---

## 4. Consequences

### Positive

- Removes Svelte, TypeScript, Vite, PDF.js, and Tauri command glue from the final target.
- Consolidates PDF behavior around one engine.
- Makes search, highlighting, page geometry, and rendering internally consistent.
- Simplifies long-term maintenance for a Rust-centered project.
- Enables a clean Rust module boundary suitable for future Iced/Slint/native renderer experiments if Dioxus Desktop becomes insufficient.

### Negative / Cost

- The display renderer must be rebuilt.
- Text selection, link handling, and PDF.js viewer conveniences are lost unless explicitly reimplemented.
- Image transport into the WebView must be designed carefully.
- Large-document performance depends on caching, cancellation, and memory control.
- Bundled/static PDFium packaging becomes a first-class release concern.

---

## 5. Alternatives Considered

### Alternative A: Keep Tauri + Svelte and only embed PDFium

This reduces packaging friction but leaves the current split architecture unchanged. It does not meet the strategic goal of Rust-first UI and domain logic.

### Alternative B: Dioxus + PDF.js bridge

This reduces UI migration risk but keeps JavaScript PDF rendering and two PDF engines. It may be useful only as a temporary spike, not as a final architecture.

### Alternative C: Iced/Slint/egui instead of Dioxus

A native Rust GUI toolkit would remove WebView dependency, but would require a larger UI rewrite and different text/image/layout tradeoffs. This is a valid future direction if WebView becomes unacceptable, but it is not the approved plan for this phase.

### Alternative D: Static PDFium linking from the start

Static linking is attractive but should not be the first production target because it increases CI/release complexity and may require platform-specific C++ standard library and framework linkage. The safer path is bundled dynamic PDFium first, static optional later.

---

## 6. Acceptance Criteria for This ADR

This ADR is accepted when the project agrees that:

1. The next target architecture is Dioxus Desktop + Rust/PDFium image-tile rendering.
2. PDF.js will not remain in the final target architecture.
3. PDFium will be internally managed by the app.
4. Bundled dynamic PDFium is acceptable as the first embedded-style production target.
5. Static PDFium is deferred until after the bundled dynamic path is stable.
6. Detailed RFCs will be created after the goal-state external design is approved.

---

## 7. Reference Notes

- Dioxus Desktop currently uses the system WebView, while Rust code runs natively.
- `pdfium-render` supports runtime dynamic binding and static linking paths, but does not include PDFium itself.
- PDFium should be treated as a serialized/native dependency boundary for safety and stability.
