---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Draft for implementation planning
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# Appendix B — Implementation Sequencing Checklist

## M0 — Branch and Acceptance

- [ ] Accept ADR-000 migration strategy.
- [ ] Create migration branch or new workspace.
- [ ] Add fixture PDFs or fixture generation plan.
- [ ] Decide first supported OS target for smoke testing.

## M1 — Dioxus Shell and File Intake

- [ ] Implement RFC-001 workspace and Dioxus shell.
- [ ] Implement RFC-002 choose-file flow.
- [ ] Add basic dashboard and toast UI.
- [ ] Validate PDF candidate paths.

## M2 — PDFium Loader

- [ ] Implement RFC-003 loader.
- [ ] Stage PDFium binary for development.
- [ ] Add bind smoke test.
- [ ] Document load diagnostics.

## M3 — Single-Page Render

- [ ] Implement RFC-004 document session model.
- [ ] Implement RFC-005 render page 1 flow.
- [ ] Display rendered page in Dioxus.
- [ ] Decide whether image transport is temporary or release-ready.

## M4 — Tile Viewer Core

- [ ] Implement RFC-006 layout engine.
- [ ] Implement RFC-007 lazy render queue/cache.
- [ ] Test 100-page fixture behavior.
- [ ] Enforce cache memory budget.

## M5 — Controls and Dashboard

- [ ] Implement RFC-008 controls and settings.
- [ ] Implement RFC-009 dashboard history behavior.
- [ ] Add settings migration tests.

## M6–M8 — UX Parity

- [ ] Implement RFC-010 search page markers.
- [ ] Implement RFC-011 exact highlights if reliable.
- [ ] Implement RFC-012 zoom overlay.
- [ ] Implement RFC-013 Zen mode and keyboard access.

## M9 — Release Hardening

- [ ] Implement RFC-014 packaging.
- [ ] Implement RFC-015 CI smoke tests.
- [ ] Implement RFC-016 security/privacy hardening.
- [ ] Verify clean machine launch.

## M10 — Cutover

- [ ] Remove runtime dependency on Svelte/Tauri/PDF.js from final app path.
- [ ] Remove external manual PDFium setup requirement.
- [ ] Update README and release notes.
- [ ] Tag migrated release.
