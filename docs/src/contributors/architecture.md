# Architecture Overview

See `docs/design/00_adr_migration_strategy.md` for the full ADR.

## Crate graph

```
app  (Dioxus Desktop binary)
 └── app_services  (orchestration layer)
      ├── domain  (pure types — no I/O, no Dioxus, no PDFium)
      ├── pdf_engine  (PDFium worker, render, search)
      └── packaging  (app-dir policy, PDFium resolution)
```

## Key constraints

- **PDFium is serialized**: all calls run on one OS thread owned by
  `pdf_engine::worker::EngineHandle`. Native handles never cross into UI code.
- **Search is non-mutating**: results are coordinate models, never a
  modified PDF buffer.
- **Generation-tagged results**: every async result carries `document_id +
  generation` so stale renders/searches can be discarded without races.
