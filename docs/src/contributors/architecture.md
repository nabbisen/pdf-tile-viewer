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

- **One owner for the session lifecycle** (RFC 035): `app_services::session_service::SessionService`
  registers the open document (`opened`) and closes it (`close`). `close` marks the session closed
  in the render, text-layer and navigation caches (a bounded `ClosedSessions` record in each, so late
  async results are refused), evicts its entries, and sends the engine `close_document`. The `Viewer`
  component calls it from `use_drop`; at most one document is open at a time.
- **Render cache policy** (RFC 035): true LRU (a hit refreshes recency); entries of a generation other
  than the open document's go first; hard byte budget (default 256 MB, minimum 64 MB). Tiles record the
  scale bucket they were rendered at and keep their old image until the render for the current bucket
  arrives; the zoom overlay does the same for a scale change on the same page.
