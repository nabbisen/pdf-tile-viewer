# Architecture Overview

See `docs/design/00_adr_migration_strategy.md` for the full ADR.

## Crate graph

```
app  (Dioxus Desktop binary)
  screens/viewer.rs           tile grid + keyboard + zen + search + zoom
  screens/viewer/render.rs    viewport-aware render scheduler
  components/zoom_overlay.rs  RFC 012 zoom overlay
  components/search_panel.rs  RFC 010/011 search UI
  components/tile_grid.rs     page tiles with search markers
  components/viewer_controls.rs scale, pages-per-row, jump-to-page
  components/drop_zone.rs     RFC 002 drag-and-drop
 └── app_services  (orchestration layer)
      ├── domain  (pure types — no I/O, no Dioxus, no PDFium)
      ├── pdf_engine  (PDFium worker, render, search + highlights)
      └── packaging  (app-dir policy, PDFium resolution, BuildInfo)
```

## Key constraints

- **PDFium is serialized**: all calls run on one OS thread owned by
  `pdf_engine::worker::EngineHandle`. Native handles never cross into UI code.
- **Search is non-mutating**: results are coordinate models, never a
  modified PDF buffer.
- **Generation-tagged results**: every async result carries `document_id +
  generation` so stale renders/searches can be discarded without races.

## Known technical debt

- `app/components/zoom_overlay.rs` was 323 ELOC; split into `zoom_overlay/util.rs`
  (PNG dimensions + highlight-rect helpers), now 295 ELOC — within the 300 soft limit.
