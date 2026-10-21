---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Accepted (planned for 2.0.1)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 004, RFC 007, RFC 023, RFC 026
amends: RFC 004 (§ lifecycle usage), RFC 007 (§ cache policy, § freshness)
origin: architect onboarding review 2026-10-21 (internal)
---

# RFC 035 — Document Session Lifecycle and Render Freshness

## 1. Summary

Give a document session an explicit owner and a single close operation, make
all caches respect it with one shared mechanism, document the cache policy
truthfully, and fix tiles that keep a stale-resolution image after a scale
change.

## 2. Motivation (evidence from code)

- **Sessions never close.** No caller of `EngineHandle::close_document`,
  `RenderService::evict_generation`, `TextLayerService::evict_session` or
  `NavigationService::evict_session` exists in `app`. Returning to the
  dashboard or opening another file leaves the native PDFium document, its
  session, and cached renders alive for the process lifetime.
- **Three caches, three behaviors.** Text and navigation caches suppress
  results for closed sessions via `suppressed_sessions` sets that grow without
  bound (and are currently never exercised); the render cache has no
  suppression, so a late render can repopulate it. The render cache is
  documented as LRU with generation-first eviction but is oldest-by-insertion
  with no generation priority.
- **Stale-scale tiles.** `TileImageState::Ready(String)` has no scale. After a
  scale change, `invalidate()` keeps `Ready` tiles and the scheduler skips them,
  so already-rendered tiles stay at the old resolution (CSS-scaled) until the
  document is reopened. RFC 007 requires that changing scale "invalidates or
  rekeys renders correctly".

## 3. Goals

- Exactly one close path; the UI cannot forget part of it.
- Bounded, uniform "closed session" handling across all caches.
- Cache policy in code equals cache policy in documentation.
- Tiles always converge to the current scale without flicker.

## 4. Non-goals

- Multi-document support (remains a product non-goal; the invariant is
  *one open document at a time*).
- Redesigning render scheduling or image transport (separate RFC after
  measurement).
- Engine process shutdown semantics (the worker thread is intentionally never
  torn down mid-process; unchanged).

## 5. Design

### 5.1 Session owner

Add `app_services::session_service::SessionService`, constructed once next to
the existing services and holding the engine handle and the three caches'
services. It exposes:

```rust
pub async fn close(&self, id: DocumentId, generation: DocumentGeneration);
```

`close` performs, in order: mark the session closed in every cache (so late
results are refused), evict its entries, then `engine.close_document(id)`. The
viewer calls it from a single lifecycle hook when the `Viewer` component is
dropped; `open` flows already pass through the dashboard, so the invariant
"at most one open session" holds. A debug assertion in `SessionService` checks
that opening a second document while one is open is a logic error.

### 5.2 Shared closed-session guard

Replace the per-cache unbounded `suppressed_sessions` sets with one small type
`ClosedSessions` (bounded FIFO, capacity 16 `(DocumentId, DocumentGeneration)`
pairs). Because only one document is open at a time and ids are never reused,
a 16-entry window is sufficient and bounded. All three caches use it, including
the render cache.

### 5.3 Render cache policy

Implement what RFC 007 specifies, in a form that is simple to reason about:
true **LRU** (a `get` refreshes recency), wrong-generation entries evicted
first, hard byte budget (existing 64 MB minimum / 256 MB default). The doc
comments and `testing.md` are updated to match. (Scale-bucket-first eviction
from RFC 007 § 10 is deferred to the render-pipeline RFC, where the active
scale is known to the cache.)

### 5.4 Render freshness (tile scale)

`TileImageState::Ready` records the `ScaleBucket` it was rendered at. The
scheduler treats a tile as satisfied only if its bucket equals the current
bucket; otherwise it schedules a render **and keeps showing the old image**
until the new one arrives (RFC 007 § 11: retain the old image briefly). Results
that arrive for a no-longer-current bucket are dropped by the existing
generation check extended with the bucket.

This is a deliberately minimal data-model fix. The later render-pipeline RFC may
replace the tile-state representation entirely.

## 6. Alternatives considered

- **Close inside `Phase` transitions in `app.rs`.** Rejected: ties lifecycle to
  navigation code and is easy to bypass.
- **Per-cache `Drop` guards.** Rejected: ordering across engine and caches must
  be explicit and testable.
- **Reset all tiles to `Pending` on scale change.** Rejected: causes flicker
  and discards a usable low-resolution image.

## 7. Security and robustness

Prevents unbounded native memory retention across documents and removes stale
data (text, links) of a closed document from memory. Late async results for a
closed session are refused in every cache.

## 8. Tests and acceptance

- `app_services` tests: `close` evicts all caches and refuses late inserts;
  `ClosedSessions` capacity bound; render-cache LRU order and generation-first
  eviction under a small budget.
- Engine smoke: after `close`, `debug_session_count` returns 0 (already
  available) and render/search/text/links return `DocumentNotOpen`.
- Unit test for the scheduler decision function (extracted as a pure function):
  bucket change ⇒ re-render; same bucket ⇒ skip; old image retained.
- Manual: open A → dashboard → open B ×5; native session count and RSS do not
  grow with closed documents. Zoom the grid from 1.0× to 3.0× and back; visible
  tiles sharpen without a blank flash.
- Acceptance: no unreferenced `close`/`evict_*` API remains; documentation
  matches code.

## 9. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Closing while async tasks are in flight | Medium | Cache refusal after close; engine returns `DocumentNotOpen`; tests with interleavings |
| Drop-time async work in Dioxus hooks | Medium | `close` is fire-and-forget on the app runtime; engine ordering is serialized |
