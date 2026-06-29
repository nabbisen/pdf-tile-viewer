# Changelog

All notable changes to PDF Tile Viewer are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [2.0.0-alpha.1] — unreleased

Complete architectural migration from the Tauri + SvelteKit + PDF.js stack
to a Rust-first Dioxus Desktop application backed by a serialized PDFium
engine worker. This release implements the **RFC 005 vertical slice**:
open a local PDF and display the rendered first page.

### Architecture

- New Cargo workspace: `domain`, `pdf_engine`, `app_services`, `packaging`,
  `app` (RFC 001).
- `domain` crate: pure Rust types — document sessions, render requests,
  search results, settings schema, tile-layout model (RFCs 002, 005–010).
- `pdf_engine` crate: PDFium binding, serialized engine worker (`EngineHandle`
  + dedicated OS thread), page rendering to PNG, non-mutating text search
  (RFCs 003, 004, 005, 010).
- `packaging` crate: app-directory policy, PDFium resolution rules (production
  loads only from `resources/pdfium/<platform>/`; RFC 016 §6).
- `app_services` crate: file intake validation (RFC 002 §7), engine bootstrap,
  settings persistence with backup-on-corruption (RFC 008 §11), session
  history with dedupe-to-top (RFC 009), platform file picker and
  reveal-in-file-manager (RFC 002).
- `app` crate: Dioxus 0.7 desktop shell — dashboard and viewer screens,
  provisional page-1 PNG preview (RFC 005).

### Internationalization

- RFC 017 i18n layer: closed `MessageKey` enum, complete English reference
  catalog, Japanese translation with automatic fallback to English.

### Testing

- 51 unit tests covering RFC specifications across all library crates.
- 5 engine smoke tests (RFC 015) exercising open / geometry / render /
  search / close against a real PDFium 7763 binary; gracefully skipped
  when PDFium is not available (`PDF_TILE_VIEWER_PDFIUM_DIR` unset).
- `ci/fetch-pdfium.sh` fetches the pinned PDFium 7763 prebuilt binary.
- `fixtures/generate_fixtures.py` produces deterministic hand-built PDF
  test fixtures without third-party dependencies.

### Known gaps (planned in subsequent milestones)

- Tile grid renderer, lazy page cache, and scale controls (RFC 006/007, M4).
- Full search UI with coordinate-based highlights (RFC 010/011, M6).
- Drag-and-drop, multi-page navigation, jump-to-page (RFC 002/013, M5).
- Zoom overlay and zen mode (RFC 012/013, M7).
- Bundled/embedded PDFium packaging (RFC 014/006, M8).
- Password-protected PDF support (deferred, noted in RFC 002).

---

## [1.x] (previous Tauri + SvelteKit releases)

See the v1 branch for the changelog of the Tauri + SvelteKit + PDF.js
application. The v2 rewrite preserves feature parity as its target but
does not share source history with v1.
