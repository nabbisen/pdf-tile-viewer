---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-24
status: Accepted (planned for 2.0.2)
accepted: 2026-10-24
baseline: PDF Tile Viewer 2.0.1
priority: Corrective
depends_on: RFC 007, RFC 035
amends: RFC 007 (§ 10 memory budget: the budget now also covers UI-side state)
origin: WP0 evidence run 2026-10-22 (Part B), architect review
amended: 2026-10-24 — § 4.1 cap clarified; § 9 result of the measured acceptance
---

# RFC 040 — Tile Retention Window (bounded UI memory)

## 1. Summary

Bound the memory used by rendered page tiles in the **UI layer** by keeping a
rendered tile only while it is near the viewport, and returning tiles that are
far away to the "pending" state. Memory then depends on the window size, not on
how many pages the user has scrolled past. No change to image transport, to the
render cache, or to what the user sees when staying in one area of the document.

## 2. Motivation (measured)

WP0 Part B (release build, synthetic corpus, `.git-exclude/review-request/wp0-evidence-run/`):

| Document | RSS after open | RSS after a full scroll down and back | Growth |
|---|---:|---:|---:|
| text, 100 pages | 220 MB + 244 MB (app + WebKit processes) | 303 MB + 354 MB | ≈ +200 MB |
| text, 1000 pages | 231 MB + 275 MB | 596 MB + 459 MB | ≈ +550 MB |
| image-heavy, 100 pages | 338 MB + 254 MB | 1 016 MB + 497 MB | ≈ **+920 MB** |

Memory was never released while scrolling away. The Rust-side render cache is
bounded (256 MB), but the UI additionally keeps **every** rendered tile as a
base64 `data:` URI in a signal (and WebKit keeps the decoded bitmap alive for the
`<img>` elements). The number of tiles ever shown, not the viewport, drives
memory. RFC 007 § 10 and the goal-state design (§ 19) require a bounded budget.

## 3. Goals / Non-goals

Goals: memory bounded independently of document length; no visible change while
the user stays within a region; correct interaction with scale changes, search
overlays and the zoom view; the bound is measurable.

Non-goals: replacing data URIs with a custom image protocol, virtualizing the tile
DOM nodes, coalescing or prioritizing render requests, engine changes (all
RFC-FUTURE F14, 2.1.0); changing the Rust render cache.

## 4. Design

### 4.1 Retention plan (pure function in `domain::layout`)

```rust
pub struct RetentionLimits {
    pub window_px: f32,        // kept distance above and below the viewport
    pub max_ready_tiles: usize,// hard cap on tiles held in the Ready state
}
pub fn retention_victims(
    layout: &TileLayout,
    viewport: ViewportRect,
    ready: &[PageIndex],
    limits: RetentionLimits,
) -> Vec<PageIndex>;
```

A tile that is `Ready` is **kept** if it intersects
`[viewport.top - window_px, viewport.bottom + window_px]`; tiles outside are
victims. If more than `max_ready_tiles` remain, the kept tiles farthest from the
viewport centre are added to the victims until the cap holds. Visible tiles are
never victims. Default limits: `window_px = max(2 × viewport_height, 1500)` and
`max_ready_tiles = 80`. Both are constants in one place with tests; they are not
user settings in this release. **Clarification (accepted 2026-10-24):** the scheduler
raises the cap to at least the number of tiles it wants rendered in the same pass
(visible + prefetch): at small scales that number can exceed 80, and evicting tiles
that are requested again in the same pass would only make them churn.

### 4.2 Application

`schedule_visible_renders` (viewer `render.rs`) computes the victims on every
scheduling pass and resets them in `tile_images` to `Pending` (dropping the URI
string). Tiles in `Rendering` state are untouched. A victim that scrolls back
into the prefetch margin is re-requested by the existing scheduler; the Rust
render cache normally serves it without a new PDFium render (only the base64
encoding is repeated). Retention applies to the tile grid only; the zoom
overlay's single image is already released when the overlay closes.

### 4.3 Interaction with RFC 035

Tile state records its `ScaleBucket` (RFC 035 § 5.4). Victim selection ignores
the bucket (distance only). A scale change keeps the existing "old image stays
until the new one arrives" behavior for tiles inside the window.

### 4.4 User-visible behavior

Scrolling back to a region after moving far away shows the loading placeholder
briefly (tiles within 2 viewport heights are never affected). This is the only
visible difference; it is documented in the FAQ ("Why do pages far from the
view show a loading placeholder?").

### 4.5 Design continuity (architect self-review, 2026-10-24)

This is not a throw-away stopgap. What RFC 040 adds is the **policy**: which
rendered tiles may stay, decided by one pure, tested function
(`retention_victims`). The **mechanism** it uses today (reset a tile to `Pending`,
which removes its `<img>`) is the same mechanism a later DOM-virtualization and
custom-image-protocol design (RFC-FUTURE F14) needs: even with short URLs, the
decoded bitmaps that WebKit holds for live `<img>` elements dominate memory, so
far tiles must still be released. F14 will replace the transport and may replace
the placeholder rendering, but it keeps this policy function. Window size and cap
are provisional values chosen from the WP0 measurements; the acceptance test
(§ 7) is what fixes them, and they may be tuned in a patch release without any
contract change.

### 4.6 What the user sees

A tile far from the viewport shows the existing loading placeholder (the same one
used while a page first renders) when the user returns to it, then fills in. No
new control, message or setting is introduced; the explanation is one FAQ entry.
Tiles within two viewport heights never change.

## 5. Alternatives considered

- **Time-based eviction.** Rejected: non-deterministic, harder to test.
- **Byte budget on the URI strings only.** Rejected as primary: WebKit's decoded
  bitmaps (not visible to the app) dominate; a count and distance bound is
  directly controllable and testable.
- **Do nothing until the image protocol (F14).** Rejected: 2.0.x users can reach
  ≈ 1 GB on a 100-page scan today; F14 is a 2.1.0 redesign.

## 6. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Visible flicker when scrolling back and forth over the window edge | Medium | Window ≥ 2 viewport heights (> prefetch margin 600 px); hysteresis is not needed because eviction is distance-based; manual check with fast scroll |
| Re-encoding CPU cost | Low | Render cache hit avoids PDFium; measured in acceptance |
| Interaction with search highlights | Low | Highlights are drawn only over `Ready` tiles already |

## 7. Tests and acceptance

- Unit tests for `retention_victims`: window boundary inclusive/exclusive, visible
  tiles never evicted, cap enforced farthest-first, empty and single-row layouts,
  mixed page sizes, deterministic ordering.
- Scheduler test with a fake state: victims reset to `Pending`; `Rendering` untouched.
- **Measured acceptance (same method and corpus as WP0 Part B, release build):**
  after a full top-to-bottom-and-back scroll, total RSS (app + WebKit) is at most
  **55 %** of the 2.0.1 value for image-heavy 100 pages and text 1000 pages, and
  **plateaus** (a second full scroll adds < 10 %). The review request records the
  before/after table.
- Manual: open a 100-page document, scroll to the end and back; tiles within the
  window never blink; far tiles show the placeholder and fill in; the search
  highlights and the zoom view still work.

## 8. Release implications

Patch-compatible (a defect fix against the documented memory budget); no setting,
file or API change. CHANGELOG "Fixed".

## 9. Result and follow-up (2026-10-24)

The policy and its scheduler integration are implemented and tested (commits `3197274`,
`07c71fa`). The **measured acceptance of § 7 is not met**, and the cause is an
assumption of this RFC that the measurements do not support.

| text-1000, release build | app process | WebKit processes | total |
|---|---:|---:|---:|
| before (`a8ca205`), after one full scroll down | 678 MB | 484 MB | 1 162 MB |
| after (`07c71fa`), after one full scroll down | 334 MB | 522 MB | 856 MB |
| after, one down-and-back (2.0.1 reference: 1 055 MB) | 390 MB | 573 MB | **963 MB (91 %)** |
| after, second down-and-back (plateau) | 400 MB | 626 MB | 1 027 MB (+6.6 %) |

- **Met:** plateau (< 10 % on the second pass, measured on the total).
- **Not met:** ≤ 55 % of the 2.0.1 value (91 %). The **app process** improved by about
  half; the **WebKit processes did not shrink and keep growing**, although at most about
  a dozen tiles are `Ready` at any time.
- **What this means:** § 2 assumed that releasing tiles from UI state releases the memory
  WebKit holds for them. It does not. The WebKit side is probably dominated by something
  other than live `<img>` elements (candidates, **unverified**: WebKit's cache of `data:`
  URL resources, JavaScript-heap strings not yet collected, allocator behaviour). The
  assumption was not tested before the acceptance criterion was written; that was a
  review gap on the architect's side.
- **Status:** the acceptance criterion is **not changed**. RFC 040 stays on `main` as a
  correct and useful policy (it is the retention policy any later transport redesign
  keeps, § 4.5), but its user-facing claim is limited to "memory growth is reduced",
  not "bounded". Whether 2.0.2 ships it as is, or together with a transport change
  (RFC-FUTURE F14), is decided after the investigation in
  `rfcs/handoffs/040-tile-retention-window/webkit-memory-investigation.md`
  (idle decay of the WebKit memory; a custom-protocol prototype; the image-heavy case).

### 9.1 Investigation results (2026-10-24, text-1000, 1042×1134 window, one tile per row)

| Step (total MB; app / WebKit) | `data:` URLs (`07c71fa`) | prototype: custom protocol (never merged) |
|---|---:|---:|
| after open | 494 (233 / 274) | 496 (232 / 276) |
| after scroll down (pass 1) | 936 (381 / 577) | 794 (353 / 461) |
| back to top (pass 1) | 1 029 (383 / 671) | 871 (364 / 528) |
| idle +30 s / +120 s | 1 029 / 1 029 | 871 / 869 |
| second down and back | 978 → 1 016 | 871 → 879 |

- WebKit's memory is **retained, not lazily reclaimed** (no decay while idle) and **plateaus**
  (no growth on the second pass): the earlier statement that UI memory "grows without bound"
  is corrected to "grows to a plateau set by WebKit's own caching plus the app's bounded
  render cache" (≈ 670 MB of WebKit processes here; unverified on smaller machines).
- A custom image protocol lowers the total by about 15 % and does not reach the 55 % target.
- Behavior of the policy at user level is as specified (far tiles released and refilled;
  tiles stay `Ready` on small scrolls; search and zoom unaffected).
- New finding (not fixable here): after a fast scroll over large images the scheduler keeps
  rendering pages already scrolled past for tens of seconds (no cancellation, repeated
  requests); it belongs to the render-pipeline RFC (F14).
- The image-heavy document is reported in § 9.2.

### 9.2 Image-heavy result (2026-10-25, image-100, 1042×1134 window, one tile per row)

One run per build, recorded-state wait (poll the accessibility tree until the destination
tile has its image), then the RSS reading; no anomaly.

| Step (total MB) | before (`a8ca205`) | after (`07c71fa`) |
|---|---:|---:|
| after open | 584 | 590 |
| scroll down (pass 1) | 894 | 749 |
| back to top (pass 1) | 1 566 | 805 (**51 %** of before) |
| second down and back | 1 742 | 836 (48 %) |

- **Met on this document:** ≤ 55 % after one down-and-back (51 % of the base commit; 53 % of
  the 2.0.1 reference of § 2, which was measured at another window geometry) and plateau
  (second pass adds 3.9 %; the base commit keeps growing, +11 %).
- **Overall acceptance of § 7 remains partly met:** image-100 meets the 55 % criterion,
  text-1000 does not (91 %, § 9). The criterion is not changed. The saving is largest where
  the application process dominates (image-heavy), and small where the WebView's own memory
  dominates (long text documents).
- The base commit's figure approaches "every page resident" (1.7 GB for 100 large pages), so
  the ratio is document-size dependent: a longer image-heavy document would show a larger
  saving, a shorter one a smaller one.
- Single run per build, Linux/WebKitGTK only; WebView2 is unmeasured (release-QA Windows
  memory check). The time for the first scroll down to finish rendering (31 s before, 41 s
  after; 0–2 s afterwards) is one sample each; no conclusion is drawn from the difference. It
  is the render backlog of F14.
- **Decision for 2.0.2 (owner, approved 2026-10-25):** ship as is. User-facing wording stays "reduced memory
  growth", with the measured image-heavy figure stated as a measurement, not a guarantee.
