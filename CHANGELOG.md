# Changelog

All notable changes to PDF Tile Viewer are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Fixed

- Hardened encrypted/password-protected PDF detection by mapping PDFium's
  typed password error directly, with fixture-backed smoke coverage.

---

## [2.0.0-beta.9]

### Added

- Added an RFC 018 packaged-artifact smoke gate. Release archives are extracted
  after compression and checked with production bundled-resource PDFium
  resolution before upload.
- Added smoke self-tests for malformed package archive layouts.

### Fixed

- Moved remaining obvious hardcoded user-visible app strings into the i18n
  catalog and wired the app catalog completeness tests.
- Corrected bundled-PDFium user documentation so official release archives are
  described as self-contained, with source-build PDFium setup kept separate.
- Corrected the FAQ entry for `privacy.show_full_path_in_title`; the setting
  is wired to the window title and remains opt-in for privacy.
- Corrected contributor test-layout documentation to prefer co-located module
  tests and reserve crate-level `tests/` for integration boundaries.

---

## [2.0.0-beta.8]

### Added

- Restored best-effort PDF text selection and copy in the single-page zoom
  overlay (RFC 023). The tile grid remains bitmap/search-overlay only.
- Added a PDFium-backed text-layer extraction path, serialized through the PDF
  engine worker, with domain-owned text segments and page-space bounds.
- Added a memory-only text-layer service cache keyed by document, generation,
  and page, with stale-result suppression for closed document sessions.

### Fixed

- Added request guards for zoom overlay bitmap and text-layer results so rapid
  page navigation or zoom changes do not apply stale asynchronous results.
- Kept zoom text spans, search highlights, and the rendered page bitmap aligned
  by using one natural-size rendered page coordinate system.

---

## [2.0.0-beta.7]

### Fixed

- Aligned all PDFium pins to the current `bblanchon/pdfium-binaries` release:
  `chromium/7920` is now used consistently by CI, release, MSIX, and local
  fetch/package helpers.
- Cleared the current `cargo clippy --workspace --all-targets -- -D warnings`
  findings across library crates and GUI glue.
- Improved the missing-PDFium boot error so packaged users get install/archive
  recovery guidance before low-level diagnostic details.
- Aligned resource-root resolution with release archives that place the binary
  under `bin/` and bundled PDFium under top-level `resources/`.

---

## [2.0.0-beta.6]

### Added

- **NOTICE**: PDFium is licensed under the BSD 3-Clause License
  ("Copyright 2014 The PDFium Authors"), not Apache 2.0.
  `pdfium-render` is MIT OR Apache-2.0 (Alastair Carey). Dioxus is MIT OR
  Apache-2.0. NOTICE now lists all three with accurate license statements.
- **GitHub Actions CI** (`.github/workflows/ci.yml`): rustfmt check,
  `cargo check --workspace`, library/service unit tests, and engine smoke
  tests against the pinned PDFium — on every push to `main` and every PR.
- **GitHub Actions release** (`.github/workflows/release.yml`): triggered by
  a semver tag (no `v` prefix). Builds the RFC 014 artifact matrix (Linux x64, Windows x64,
  macOS arm64, macOS x64), each with its bundled PDFium, runs the package
  verification smoke test per platform, builds a source archive, and
  publishes a GitHub Release. Pre-release tags (`-alpha`/`-beta`/`-rc`) are
  marked as GitHub pre-releases.
- **Microsoft Store MSIX workflow** (`.github/workflows/msix-store.yml`):
  manual (`workflow_dispatch`) pathway that builds the Windows binary with
  bundled PDFium, stages the package via `ci/stage-msix.sh`, runs `makeappx`,
  and produces a `.msix` artifact for Partner Center. Optional `signtool`
  signing when cert secrets are present. Store submission stays manual.
- **`ci/stage-msix.sh`**: assembles the MSIX package root and substitutes the
  `@VERSION@` placeholder in `AppxManifest.xml` with the semver core + revision.
- `AppxManifest.xml`: replaced the stale `1.1.2.0` version with a `@VERSION@`
  placeholder resolved at build time.
- Documented both release pathways (executable build vs Store MSIX) in
  `docs/src/contributors/dev.md`.

### Fixed

- **`open_action` warning** (`app.rs:113`): the `settings: Signal<AppSettingsV1>`
  parameter was unused after the beta.5 dead-code cleanup. Removed from the
  function signature and both call sites.

### Changed — test structure

All three library crates that had a monolithic `src/tests.rs` + `src/tests/`
pattern now use per-module co-location as required by the project rules
(`src/some_mod.rs` → `src/some_mod/tests.rs`):

- **`domain`**: `document`, `layout`, `search`, `settings` tests each live
  beside their module. Centralised `src/tests.rs` + `src/tests/` removed.
- **`app_services`**: `document_service`, `engine_boot`, `history_service`,
  `render_service`, `settings_service` tests co-located. Centralised files
  removed. `src/` is now as clean as `domain/src/`.
- **`packaging`**: `pdfium_bundle` tests moved to `src/pdfium_bundle/tests.rs`.
  `src/tests.rs` removed.

`pdf_engine/tests/smoke.rs` is a crate-level integration test (standard Rust
`tests/` directory) — that structure is correct and was not changed.

---

## [2.0.0-beta.5]

* Codebase housekeeping.
* `packaging/windows`: App manifest and assets for Microsoft Store publish.

---

## [2.0.0-beta.4]

Codebase housekeeping.

---

## [2.0.0-beta.3]

Source archive format.

### Changed

- **Source archive format** (takes effect from this release onward):
  - Archive name: `pdf-tile-viewer-vX.X.X.tar.gz` (added `v` prefix).
  - Archive root: `pdf-tile-viewer-vX.X.X/` containing files directly —
    no more nested `pdf-tile-viewer/` subdirectory.
  - `ci/archive-source.sh` added as the canonical source-archive builder;
    uses GNU tar `--transform` to rename the top-level directory.
  - `ci/package-linux.sh` updated with the same naming convention for
    binary release artifacts.

---

## [2.0.0-beta.2]

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

## [2.0.0-beta.1]

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

## [2.0.0-alpha.4]

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

## [2.0.0-alpha.3]

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

## [2.0.0-alpha.2]

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

## [2.0.0-alpha.1]

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
  search / close against the then-pinned real PDFium binary; gracefully skipped
  when PDFium is not available (`PDF_TILE_VIEWER_PDFIUM_DIR` unset).
- `ci/fetch-pdfium.sh` fetches the pinned PDFium prebuilt binary.
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
