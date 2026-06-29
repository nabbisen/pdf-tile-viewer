# Changelog

All notable changes to PDF Tile Viewer are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [2.0.0-alpha.2] — unreleased

Tile grid renderer, lazy page rendering, and viewer controls (M4).

### Added

- **Tile grid** (`app/components/tile_grid.rs`): all document pages rendered
  as absolute-positioned tiles in a scroll container, sized by the layout
  engine (RFC 006). Shows a loading pulse animation while rendering, an error
  state on failure, and optional per-tile page-number labels.
- **Render service** (`app_services/render_service.rs`): `RenderCache` (LRU,
  budget-aware, generation-eviction, 6 new unit tests) and `RenderService`
  wrapping `EngineHandle`. Cache is provided via Dioxus context and shared
  across all spawned render tasks (RFC 007).
- **Lazy render scheduling** (RFC 007 §8): on viewer mount and scale/mode
  change, `render_gen` is bumped; `use_effect` spawns one async task per tile.
  Tasks guard-check the generation on completion to silently discard stale
  results (Appendix A §8).
- **Viewer controls** (`app/components/viewer_controls.rs`): scale slider with
  ± buttons (RFC 008 §9.1), auto/fixed pages-per-row toggle (RFC 008 §9.2),
  page-number visibility checkbox, jump-to-page with one-based validation and
  JS `scrollIntoView` (RFC 008 §9.3).
- **Ctrl+scroll** wheel binding on the tile grid container rescales tiles.
- **`dioxus::document` feature** enabled for JS `eval` support (jump-to-page).
- `PartialEq` added to layout domain types (`TileLayout`, `TileRow`,
  `PageTileLayout`, `TileLayoutInput`, `ViewerScale`) required by Dioxus
  `#[component]` props and `use_memo`.

### Changed

- Viewer screen (`app/screens/viewer.rs`) fully rewritten to use the tile grid;
  page-1 data-URI preview retained as fallback only.

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
