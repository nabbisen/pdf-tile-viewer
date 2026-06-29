# Changelog

All notable changes to PDF Tile Viewer are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [2.0.0-beta.2] — unreleased

Performance hardening (M9), RFC lifecycle completion (M10),
documentation audit, window title/size persistence, and code-quality fixes.

### Performance and large-document hardening (M9)

- **Viewport-based render scheduling** (`screens/viewer/render.rs`): only
  visible and near-visible tiles (within a 600 px prefetch margin) are
  scheduled; distant tiles stay `Pending` until scrolled into view.
- **Scroll tracking**: `onscroll` on the tile-grid container updates
  `scroll_y` and `viewport_height`, re-triggering the scheduler on each
  scroll event.
- **Viewport height measurement**: `eval("return window.innerHeight")` on
  mount; combined with `window.innerWidth` for accurate layout.
- **Settings-wired cache budget**: `advanced.render_cache_budget_mb` now
  propagates to `RenderService::new(engine, budget)`.
- **50-page benchmark fixture** and 2 new engine smoke tests (7 total).
- **`viewer.rs` module split**: render scheduling extracted to
  `screens/viewer/render.rs` (both files under 200 ELOC).

### RFC lifecycle completion (M10)

- All 17 migration RFCs moved `rfcs/proposed/` → `rfcs/done/` with
  `status: Implemented`.
- `rfcs/README.md` rewritten with all-implemented index and milestone
  cross-references.

### Window title and size (RFC 008/016)

- **Window title** respects `privacy.show_full_path_in_title`: default
  shows `<filename> — PDF Tile Viewer`; `true` shows the full path.
  Implemented via `document::Title` (dioxus-document).
- **Window size save**: viewport dimensions written to
  `settings.window.width/height` when measured on document open.
- **Window size restore**: `main.rs` uses `LaunchBuilder::with_cfg` +
  `Config::new().with_window(WindowBuilder::new().with_inner_size(...))`
  to restore the last-used size at launch.

### Code-quality

- `zoom_overlay/util.rs` sub-module extracted (`png_dimensions`,
  `zoom_highlight_rects`); `zoom_overlay.rs` now 295 ELOC (under the
  300 soft limit).

### Documentation audit

Comprehensive review of all docs against the codebase. Corrections in:
`README.md`, `features.md`, `settings.md`, `shortcuts.md`, `faq.md`,
`testing.md`, `installation.md`, `tile-grid.md`, `search.md`,
`opening.md`, `architecture.md`. Key fixes: stale version references,
removed unimplemented shortcuts (`Ctrl+O/F`, `G`), marked schema-only
settings clearly, updated test counts (58).

---

## [2.0.0-rc.2] — unreleased

Complete v2 Dioxus + PDFium candidate release.

### Summary

PDF Tile Viewer 2.0.0 is the complete Rust-first rewrite of the original
Tauri + SvelteKit + PDF.js + external PDFium application. The product
delivers feature parity with v1.x while replacing the entire frontend stack
with a single Rust codebase.

### New in 2.0.0 (over rc.1)

- **Window title privacy**: the title bar shows `<filename> — PDF Tile Viewer`
  by default. Set `privacy.show_full_path_in_title: true` in settings to
  show the full file path (RFC 016 §7).
- **Window size persistence**: the viewer measures `window.innerWidth/Height`
  via JS eval on document open and saves to `window.width/height` in settings.
  `main.rs` restores the last-used size at launch via `LaunchBuilder` +
  `Config::new().with_window(WindowBuilder::new().with_inner_size(...))`.
- **`zoom_overlay/util.rs`** split out: `png_dimensions` and
  `zoom_highlight_rects` moved to a sub-module, bringing `zoom_overlay.rs`
  from 323 to 310 ELOC (toward the 300 soft limit).
- Documentation audit pass: all stale version references, "coming soon"
  items that are done, unimplemented settings, and missing shortcuts
  corrected across `README.md`, `features.md`, `settings.md`,
  `shortcuts.md`, `faq.md`, `testing.md`, `installation.md`,
  `tile-grid.md`, `search.md`, and `opening.md`.

### Technical summary (full v2 over v1.x)

| Dimension | v1.x | v2.0.0 |
|-----------|------|--------|
| Frontend | SvelteKit + TypeScript | Rust + Dioxus |
| PDF rendering | PDF.js (JavaScript) | PDFium (Rust/native) |
| PDF search | PDFium (external proc) | PDFium (same worker) |
| Search model | Modified PDF bytes | Non-mutating overlays |
| Shell | Tauri v2 | Dioxus Desktop |
| External lib | Visible lib/ directory | resources/ bundled |
| Test coverage | Minimal | 58 tests + 7 smoke |

---

## [2.0.0-beta.1] — unreleased

Zoom overlay (M7) and release packaging infrastructure (M8).
This milestone achieves feature parity with the original v1.1.x
Tauri + SvelteKit application (minus multi-tab and per-document state).

### Added — Zoom Overlay (RFC 012)

- **`app/components/zoom_overlay.rs`**: click any tile → full overlay with
  dimmed backdrop, high-resolution page render at `viewer.zoom_overlay_scale`
  (default 2.7×), scale ±, prev/next/first/last navigation, RFC 011 highlight
  overlays, `autofocus` on close button, stop-propagation on panel click.
- **Keyboard shortcuts** (RFC 012 §9 / RFC 013 §8): Escape close,
  ←/→/PageUp/PageDown navigate, Home/End, +/- scale.
- Escape key priority order in viewer: zoom overlay → search panel →
  zen mode → back to dashboard.
- `png_dimensions()` helper reads the IHDR chunk without full decode for
  correct coordinate transform scaling.

### Added — Release Packaging (RFC 014)

- **`packaging/src/build_info.rs`**: `BuildInfo` struct populated at
  compile-time (`env!("CARGO_PKG_VERSION")`, `option_env!("VERGEN_GIT_SHA")`,
  `env!("TARGET")`); short_version() for diagnostics display (RFC 014 §8).
- **`crates/packaging/build.rs`**: re-exports the `TARGET` triple as
  `CARGO_ENV_TARGET` for use in `BuildInfo`.
- **`ci/package-linux.sh`**: full packaging pipeline — release build,
  staging directory with binary + bundled PDFium + docs, engine smoke test,
  `.tar.gz` archive output (RFC 014 §7).

### Changed

- `zoom_page: Signal<Option<PageIndex>>` in viewer drives overlay open/close.
- Tile click callback wired: `on_tile_click: Some(Callback::new(…))`.
- Search highlights passed into zoom overlay for RFC 011 alignment in zoomed view.

---

## [2.0.0-alpha.4] — unreleased

Text search with page markers and highlight coordinate overlays (M6).

### Added

- **Search panel** (`app/components/search_panel.rs`): floating search bar
  with text input (Enter or Search button), Clear button, match summary
  ("N matches / Np"), compact matched-page range display ("1, 4–6, 10"),
  and Searching/NoResults/Failed states (RFC 010 §8). Toggled by 🔍
  button in the viewer header; Escape closes it.
- **Page-level search markers** (`tile_grid.rs`): matched tiles receive an
  accent-coloured border and a badge showing the match count (RFC 010 §7).
- **Highlight coordinate overlays** (RFC 011): semi-transparent yellow
  `<div>` overlays positioned over each match rectangle. Coordinates are
  extracted from PDFium's `PdfPageTextSegment::bounds()` and converted
  from PDF bottom-left space to image-pixel space via
  `page_rect_to_image_rect` (RFC 011 §6–§7).
- **RFC 011 domain types** (`domain/search.rs`): `PageCoordinateSpace`,
  `PageRect`, `TextHighlight`, `PageHighlightSet`, `SearchHighlightSet`.
- **RFC 011 transform functions** (`domain/layout.rs`):
  `page_rect_to_image_rect` and `image_rect_to_tile_rect` with full test
  coverage (4 new tests: y-axis flip, identity pass-through, scale, offset).
- **`search_document_with_highlights`** in both `pdf_engine/search.rs` and
  `pdf_engine/worker.rs`: single PDFium pass returning page summaries +
  match rects together.
- `EngineHandle` now implements `PartialEq` via an `Arc<()>` identity token
  (required by Dioxus `#[component]` props).
- 6 new i18n message keys (Search, Clear, Placeholder, Summary, NoResults,
  Searching) in English and Japanese.
- Total: **56 tests** (up from 52); all passing.

---

## [2.0.0-alpha.3] — unreleased

Drag-and-drop, settings persistence, viewport measurement, zen mode,
keyboard shortcuts, reveal-in-file-manager (M5).

### Added

- **Drag-and-drop** (`app/components/drop_zone.rs`): reusable `DropZone`
  component accepting a single dropped PDF via HTML5 ondrop with Wry
  native path injection (RFC 002 §5.2). Rejects multiple files with a
  clear error toast; shows visual `drag-over` highlight.
- **Settings persistence** (RFC 008 §8): scale, pages-per-row, and
  show-page-numbers are written back to `AppSettingsV1` and saved on
  every viewer-control change (immediate save; debounce deferred to M6).
- **Viewport width measurement** (RFC 005→M5): `eval("return window.innerWidth")`
  called on viewer mount; result feeds the layout engine so tile columns
  adapt to the actual window width instead of the 1200 px placeholder.
- **Reveal in file manager** (RFC 002 §9): "📂" button in the viewer
  header calls `app_services::platform::reveal_in_file_manager`. Shows
  inline error if the OS command fails.
- **Zen mode** (RFC 013 §5–§7): `Z` key toggles full-screen tile view;
  controls and header are hidden; a floating `×` exit button stays
  visible. `Escape` exits zen mode (or navigates to dashboard when not
  in zen).
- **Keyboard shortcuts** (RFC 013 §8): `+`/`=` scale up, `-` scale
  down, `0` reset scale to default, `Z` toggle zen, `Escape` exit zen /
  back to dashboard. Viewer container is focusable (`tabindex="0"`) with
  a visible focus ring.
- `SettingsStore` provided via Dioxus context so the viewer can save
  without prop-drilling.
- 5 new i18n message keys (`ErrMultipleFilesDropped`, `DropZoneHint`,
  `RevealInFileManager`, `ZenModeEnter`, `ZenModeExit`) in both English
  and Japanese catalogs.
- `HasFileData` trait import resolved for `DragData::files()` in Dioxus
  desktop drop events.

### Changed

- `Dashboard` component gains `on_open_path` callback and `last_error`
  signal for drop-zone error display and history-item re-open.
- `app.rs` refactored: `open_action` handles both picker (no path) and
  direct path (drag/drop, history re-open) in a single function.
- Drop zone visual polish: dashed border, `drag-over` highlight, focus
  rings on all interactive elements (RFC 013 §9).

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
