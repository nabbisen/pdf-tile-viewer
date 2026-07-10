---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Draft for implementation planning
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# Appendix C — Source and Assumption Notes

## 1. Purpose

This appendix records external assumptions used while designing this RFC set. It is not a legal or dependency audit.

## 2. Dioxus Desktop Assumption

The target app uses Dioxus Desktop as the app shell. Dioxus Desktop renders through a system WebView. Therefore this migration removes Tauri/Svelte and JavaScript product logic, but it does not remove WebView from the desktop rendering stack.

Reference checked during design:

- Dioxus Desktop guide, `https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/`

## 3. PDFium / pdfium-render Assumption

The target app uses PDFium through Rust bindings as the authority for document opening, page geometry, rendering, and search/highlight data. `pdfium-render` supports runtime binding to PDFium and has documentation for dynamic/static library path handling.

References checked during design:

- pdfium-render crate docs, `https://docs.rs/crate/pdfium-render/latest`
- pdfium-render repository, `https://github.com/ajrcarey/pdfium-render`

## 4. PDFium Threading Assumption

PDFium should be treated as a native engine boundary that is not freely parallel by default. The RFCs therefore require a serialized PDF engine worker or equivalent protected boundary.

Reference checked during design:

- pdfium-render issue/discussion on thread-safety concerns, `https://github.com/ajrcarey/pdfium-render/issues/20`

## 5. Packaging Assumption

The first target uses bundled dynamic PDFium because it is more practical than static linking for a near-term migration. Static linking remains a future RFC.

## 6. Product Scope Assumption

The target product preserves the existing tile viewer identity: open a local PDF, view many pages in a tile overview, search text, mark matched pages, inspect a page in zoom overlay, and keep local settings. It does not become a full PDF editor or general document-management system.
