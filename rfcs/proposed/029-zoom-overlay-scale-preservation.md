---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: Post-2.0 candidate
depends_on: RFC 012, RFC 023, RFC 026
---

# RFC 029 — Zoom Overlay Scale Preservation

## 1. Summary

Preserve the zoom overlay scale while the app is running so users do not need
to reset it each time they open a page in the zoom view.

This is a small user-facing convenience improvement. It should be designed
separately from persistent settings because persistence changes privacy and
schema behavior.

## 2. Motivation

Users often inspect several pages at the same zoom level. Resetting the zoom
overlay scale on every open interrupts that workflow, especially when checking
details across multiple pages.

## 3. Goals

- Preserve the last zoom overlay scale during the current app session.
- Keep tile-grid scale and zoom-overlay scale independent.
- Preserve text-selection, search-highlight, and link-overlay geometry.
- Avoid adding persistent settings in the first slice.

## 4. Non-Goals

- Persisting zoom scale across app launches.
- Changing tile-grid scale behavior.
- Adding per-document zoom memory.
- Adding animated zoom transitions; see RFC 030.
- Entering the 2.0.0 final release gate.

## 5. Proposed Behavior

The app should lift zoom overlay scale state above the overlay component so the
value survives closing and reopening the overlay. The state remains
process-memory-only. Opening a different document may either keep the global
session zoom scale or reset it; the recommended first design is global
session-only because it is simple and matches the user's immediate workflow.

## 6. Acceptance / QA Checklist

- [ ] Change zoom overlay scale on one page.
- [ ] Close and reopen zoom on another page; scale is preserved.
- [ ] Tile-grid scale is unchanged.
- [ ] Search highlights stay aligned after preserved zoom scale is used.
- [ ] Text selection stays aligned after preserved zoom scale is used.
- [ ] PDF link overlays stay aligned after preserved zoom scale is used.
- [ ] App restart resets the scale unless persistence is explicitly promoted.
