# Architecture Overview

See `docs/design/00_adr_migration_strategy.md` for the full ADR.

## Crate graph

```
app  (Dioxus Desktop binary)
  app.rs                       root: services, phase (dashboard/viewer), window title, <html lang>
  screens/viewer.rs            the viewer component: signals, layout, dispatcher of shortcut actions
    viewer/toolbar.rs            back / outline / search / reveal / zen buttons
    viewer/shortcuts.rs          which layer is on top; key -> resolver action
    viewer/persistence.rs        outline loader, viewport measurement, settings writers
    viewer/render.rs             viewport-aware render scheduler, tile freshness, tile retention
  components/zoom_overlay.rs   RFC 012 zoom overlay (+ zoom_overlay/: loaders, state, text/link layers,
                               external-link dialog)
  components/modal.rs          shared modal contract: focus, Tab trap, inert background (RFC 033)
  components/search_panel.rs   RFC 010/011 search UI (status / alert live regions)
  components/tile_grid.rs      page tiles with search markers
  components/viewer_controls.rs scale, Auto|Fixed columns, page numbers, jump-to-page
  components/outline_panel.rs  RFC 026 outline
  components/password_prompt.rs, drop_zone.rs, error_panel.rs
  keys.rs                      key event -> KeyDescriptor; the editable-field key helper
  i18n.rs, i18n/failure.rs     catalogs, system-locale choice, failure -> message key mappers
  page_meta.rs                 document language and title for the web view
 └── app_services  (orchestration layer)
      session_service  one owner of the open session (opened / close)
      render_service, text_service, navigation_service  caches with a bounded ClosedSessions record
      document_service  intake validation, open, password outcome (OpenFailure)
      ├── domain  (pure types — no I/O, no Dioxus, no PDFium)
      │     shortcuts (resolver), layout (tile layout, tile retention plan), geometry, ...
      ├── pdf_engine  (PDFium worker, render, search + highlights)
      │     user_space (the only place with rotation / crop arithmetic), navigation (+ destination)
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
- **One place for page geometry** (RFC 034): `pdf_engine::user_space` converts PDF user-space geometry to
  display space (rotation and crop applied, origin top-left). Everything above the engine receives display
  geometry and has no rotation logic.
- **One resolver for keys** (RFC 033): `domain::shortcuts::resolve(context, key)` is the only mapping from keys
  to actions; modal layers own the keyboard, editable fields use `keys::editable_field_keys`.
- **Failures are keys, not text** (RFC 037): UI state stores a `MessageKey`; one exhaustive mapper per failure
  family in `i18n::failure`; `{:?}` never reaches the UI.
- **Bounded UI memory** (RFC 040): `domain::layout::retention_victims` decides which rendered tiles may stay
  (near the viewport, at most 80); the viewer returns the others to `Pending`. The WebView's own memory is not
  reduced by this (see RFC 040 § 9).
- **Interface language** (RFC 043): `i18n::choose_locale` — `ui.locale`, else `LANG`, else the OS language.
