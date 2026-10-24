# Roadmap

Tracks the milestone plan for the Dioxus + PDFium migration and what comes next.
The RFC index is `rfcs/README.md` (RFCs move between `proposed/`, `accepted/`,
`done/` and `archive/` as they progress); the ADR and the detailed migration roadmap
are in `docs/design/`. Version numbers are the owner's decisions; the rules for what
a patch, minor or major release means are in
[Versioning policy](docs/src/contributors/versioning.md).

---

## Milestone overview

| Milestone | Theme | Key RFCs | Status |
|-----------|-------|----------|--------|
| M0 | Technical spikes — go/no-go gate | RFC 003, 005 | ✅ Done (alpha.1) |
| M1 | PDF core extraction | RFC 002, 004 | ✅ Done (alpha.1) |
| M2 | Dioxus shell + settings | RFC 001, 008, 009 | ✅ Done (alpha.1) |
| M3 | RFC 005 vertical slice (page-1 preview) | RFC 005 | ✅ Done (alpha.1) |
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
| `2.0.0` | Released (tag `2.0.0`, 2026-07-14) |
| `2.0.1` | Hotfix, RFC 039 only: no abort on the scale shortcuts — released 2026-10-24 |
| `2.0.2` | Corrective train (RFC 033–040): fixes only |
| `2.1.0` | Post-2.0 candidates: settings/history policy, viewport tracking, grid keyboard navigation, picker/zoom polish; **Escape no longer leaves the document when idle (RFC 033, step 2)** |

---

## Approved corrective track (owner decision, 2026-10-21)

Correctness, safety, and truthfulness fixes found by the architect onboarding
review. No new features. RFC 039 (the scale-shortcut abort found by the evidence
run) was released as `2.0.1`; the remaining RFCs form the `2.0.2` train. Nothing is
tagged or published without the owner's explicit go/no-go (DEC-001).

| RFC | Theme |
|-----|-------|
| 033 | Keyboard routing, modal layers, Escape never leaves the document |
| 034 | Display-space page geometry (rotation/crop correctness) |
| 035 | Document session lifecycle and render freshness |
| 036 | Release supply-chain and workflow hardening |
| 037 | User-facing error presentation and accessible names |
| 038 | Documentation truth, RFC-policy compliance, repository hygiene |
| 040 | Tile retention window (bounded UI memory) |
| 043 | System locale detection |

The evidence run (manual repro + measured stress test) is done; its measurements
feed the render-pipeline RFC (F14).

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
- **F14** Render pipeline and image transport (measurements from the evidence run exist)
- **F15** Search progress and cancellation
- **F16** PDFium process isolation (roadmap theme; owner decision D6)

Implemented former future themes:

- **F02** Password-protected PDF open flow — RFC 025 (`2.0.0-beta.11`)
- **F03** Text selection — RFC 023 (`2.0.0-beta.8`)
- **F04** Link and outline navigation — RFC 026

---

## RFC status

See `rfcs/README.md` for the full RFC index: proposed RFCs are in `rfcs/proposed/`,
accepted ones (implementation may start) in `rfcs/accepted/`, implemented RFCs and
the lifecycle policy in `rfcs/done/`, withdrawn or superseded ones in `rfcs/archive/`.
