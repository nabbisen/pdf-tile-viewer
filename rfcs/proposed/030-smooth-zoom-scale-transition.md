---
project: PDF Tile Viewer
document_family: Post-2.0 / v2.1 candidate RFCs
language: English
date: 2026-07-10
status: Proposed
baseline: PDF Tile Viewer 2.0.0 release candidate line
priority: Post-2.0 candidate
depends_on: RFC 012, RFC 023, RFC 026, RFC 029
---

# RFC 030 — Smooth Zoom-Scale Transition

## 1. Summary

Reduce the dizzying visual transition when changing zoom scale in the zoom
overlay.

This RFC is a polish candidate with a real correctness tradeoff: zoom animation
must not make text, search, or PDF link overlays visibly incorrect.

## 2. Motivation

The current zoom overlay rerenders the page bitmap at the new scale. That is
accurate, but the visual transition can feel abrupt. A smoother transition may
make repeated zoom changes more comfortable.

## 3. Goals

- Improve perceived smoothness when zoom scale changes.
- Keep the final rendered bitmap and all overlays geometrically accurate.
- Avoid activating links or breaking text selection during transition.
- Preserve existing keyboard and button zoom controls.

## 4. Non-Goals

- Replacing the PDF rendering pipeline.
- Adding continuous pinch-zoom gesture support.
- Animating tile-grid scale changes.
- Shipping a transition that leaves overlays misaligned after settling.
- Entering the 2.0.0 final release gate.

## 5. Design Options

### Option A — CSS Transform While New Render Loads

Temporarily transform the old bitmap to the requested scale, then swap to the
fresh render when ready. This feels smooth but can temporarily mismatch text,
search, and link overlays unless overlays are hidden or transformed in lockstep.

### Option B — Crossfade Old and New Renders

Keep the old render visible until the new render arrives, then crossfade. This
avoids temporary geometry mismatch during rendering but may still feel less
direct than transform interpolation.

### Option C — No Animation, Reduce Layout Jumps

Keep immediate rerendering but stabilize container dimensions and loading
state. This is the safest fallback if overlay correctness cannot be guaranteed.

## 6. Acceptance / QA Checklist

- [ ] Zooming in and out feels less abrupt than the current behavior.
- [ ] Final bitmap, text selection, search highlights, and link overlays align.
- [ ] During transition, no stale link overlay can be activated at a wrong
      location.
- [ ] During transition, text selection does not select the wrong text.
- [ ] Rapid zoom changes do not show stale final renders.
- [ ] Reduced-motion users can avoid animation if CSS media support is used.
