# Roadmap

Tracks the milestone plan for the Dioxus + PDFium migration.
RFC designs live in `rfcs/proposed/`; the ADR and detailed roadmap are in
`docs/design/`.

---

## Milestone overview

| Milestone | Theme | Key RFCs | Status |
|-----------|-------|----------|--------|
| M0 | Technical spikes — go/no-go gate | RFC 003, 005 | ✅ Done (this release) |
| M1 | PDF core extraction | RFC 002, 004 | ✅ Done (this release) |
| M2 | Dioxus shell + settings | RFC 001, 008, 009 | ✅ Done (this release) |
| M3 | RFC 005 vertical slice (page-1 preview) | RFC 005 | ✅ Done (this release) |
| M4 | Tile grid + lazy render queue | RFC 006, 007 | ✅ Done (alpha.2) |
| M5 | Viewer controls, navigation, drag-and-drop | RFC 002, 008, 013 | ✅ Done (alpha.3) |
| M6 | Search and page markers | RFC 010, 011 | ✅ Done (alpha.4) |
| M7 | Zoom overlay, zen mode, accessibility | RFC 012, 013 | Planned |
| M8 | Bundled PDFium packaging + CI release artifacts | RFC 014, 003 | Planned |
| M9 | Performance and large-document hardening | RFC 015 (extended), 016 | Planned |
| M10 | Decommission old Tauri/Svelte/PDF.js stack | RFC 010 (old) | Planned |

---

## Release plan

| Version | Gate |
|---------|------|
| `2.0.0-alpha.1` | Vertical slice: open PDF, render page 1 |
| `2.0.0-alpha.2` | Tile rendering, scale controls (done) |
| `2.0.0-beta.1` | Search, highlights, zoomed page viewer, file history |
| `2.0.0-rc.1` | Full v1.x feature parity, bundled PDFium, CI artifacts |
| `2.0.0` | Production release, Tauri/Svelte removed |
| `2.1.0` | Large-PDF cache policy, optional static PDFium linking |

---

## Future themes (post-2.0)

Tracked in `docs/design/RFC-FUTURE-notes.md`:

- **F01** Static PDFium linking (single binary, no external .so)
- **F02** Page text layer / text selection
- **F03** Annotation and link support
- **F04** Multi-document tabs / workspace
- **F05** Persistent recent-files with cross-session history
- **F06** Offline web app target (Dioxus Web / WASM)

---

## RFC status

See `rfcs/README.md` for the full RFC index.
All active RFCs are in `rfcs/proposed/`. Completed policy: `rfcs/done/`.
