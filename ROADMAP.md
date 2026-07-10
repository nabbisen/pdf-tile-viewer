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
| M7 | Zoom overlay, zen mode, accessibility | RFC 012, 013 | ✅ Done (beta.1) |
| M8 | Bundled PDFium packaging + CI release artifacts | RFC 014, 003 | ✅ Done (beta.1) |
| M9 | Performance and large-document hardening | RFC 015 (extended), 016 | ✅ Done (beta.2) |
| M10 | Decommission old Tauri/Svelte/PDF.js stack (RFC lifecycle) | — | ✅ Done (beta.2) |

---

## Release plan

| Version | Gate |
|---------|------|
| `2.0.0-alpha.1` | Vertical slice: open PDF, render page 1 |
| `2.0.0-alpha.2` | Tile rendering, scale controls (done) |
| `2.0.0-beta.1` | Search, highlights, zoomed page viewer, file history |
| `2.0.0-beta.11` | Final beta before 2.0.0; password-protected PDF opening |
| `2.0.0 RC` | Release-candidate validation after RFC 026 follow-up acceptance |
| `2.0.0` | Final release prep in progress |
| `2.1.0` | Post-2.0 candidates: settings/history policy, picker/zoom polish, optional platform investigations |

---

## Future themes and v2.1 candidates

These items are **not 2.0.0 blockers**. They belong to the post-2.0 planning
queue. Proposed RFCs live in `rfcs/proposed/`.

Tracked in `docs/design/RFC-FUTURE-notes.md`:

- **F01** Static PDFium linking (single binary, no external .so)
- **F05** Native renderer / non-WebView UI investigation
- **F06** Offline web app target (Dioxus Web / WASM)
- **RFC 027 / F07** Linux dashboard drag/drop reliability
- **RFC 028 / F08** File picker initial directory memory
- **F09** Toolbar icon alignment polish — resolved as a small pre-2.0 fix
- **RFC 029 / F10** Zoom overlay scale preservation
- **RFC 030 / F11** Smooth zoom-scale transition
- **RFC 031 / F12** Settings storage with `app-json-settings`
- **RFC 032 / F13** Persistent document history with privacy controls

Implemented former future themes:

- **F02** Password-protected PDF open flow — RFC 025 (`2.0.0-beta.11`)
- **F03** Text selection — RFC 023 (`2.0.0-beta.8`)
- **F04** Link and outline navigation — RFC 026

---

## RFC status

See `rfcs/README.md` for the full RFC index.
All active RFCs are in `rfcs/proposed/`. Completed policy: `rfcs/done/`.
