# Handoff — RFC 035 Document Session Lifecycle and Render Freshness

Governing RFC: [`../../accepted/035-document-session-lifecycle-and-render-freshness.md`](../../accepted/035-document-session-lifecycle-and-render-freshness.md)
Executes the RFC; must not redefine it. Conflicts or gaps → design request to the architect.

Schedule: Tue 11-17 → Thu 11-19 (**Slice D**), after RFC 034 (Slice C) is committed and `main` is green. Working model: directly on `main`, no branches, one actor at a time, hand over with the `main` commit hash. No tags, no releases (DEC-001).

## Purpose
One close path that the UI cannot forget; uniform, bounded handling of closed sessions across all caches; a truthful LRU render cache; tiles that converge to the current scale.

## Background (verified in code, 2026-10-21)
No caller of `EngineHandle::close_document`, `RenderService::evict_generation`, `TextLayerService::evict_session`, or `NavigationService::evict_session` exists in `app`. Text/navigation caches keep unbounded `suppressed_sessions` sets; the render cache has none, documents LRU but evicts oldest-by-insertion. `TileImageState::Ready(String)` has no scale; `invalidate()` keeps `Ready` tiles and the scheduler skips them.

## Change scope

### 1. `ClosedSessions` (app_services)
`crates/app_services/src/closed_sessions.rs` (+ `closed_sessions/tests.rs`): bounded FIFO set of `(DocumentId, DocumentGeneration)`, capacity 16 (const), API `mark_closed`, `is_closed`. Replace `suppressed_sessions: HashSet` in `TextLayerCache` and `NavigationCache` (rename their `evict_session` internals to use it) and add it to `RenderCache`. Behavior of existing suppression tests must be preserved; add a capacity test (the 17th insert drops the oldest).

### 2. Render cache LRU (app_services/render_service)
- `RenderCache::get` becomes `&mut self` and refreshes recency (`last_used_seq`). Callers hold the existing `Mutex`.
- Eviction victim order: (1) entries whose generation differs from the **active generation** (`set_active_generation(Option<DocumentGeneration>)`, set by `SessionService::opened`), then (2) least-recently-used. Byte budget semantics unchanged (default 256 MB, minimum 64 MB).
- `insert` refuses keys whose session is in `ClosedSessions`.
- Fix the doc comments and `docs/src/contributors/testing.md` wording so they describe exactly this policy (no "FIFO" mention remains; if `testing.md` does not describe the cache, add nothing).

### 3. `SessionService` (app_services)
`crates/app_services/src/session_service.rs` (+ tests). Constructed in `app.rs` where the services are created, provided via `use_context_provider`. Per RFC 035 § 5.1 (read it, including the synchronous signature):
```rust
pub fn opened(&self, session: &DocumentSession);
pub fn close(&self, id: DocumentId, generation: DocumentGeneration);
```
`opened` records the active session, sets the active generation on the render cache, and `debug_assert!`s that no other session is active. `close`: mark closed in all three caches → evict entries → `drop(engine.close_document(id))` (the command is sent when the method is called). Idempotent: closing an unknown/already closed session is a no-op. `app.rs` calls `opened` at both success sites (picker/path open and password open).

### 4. Wire the lifecycle (app)
In `Viewer` (`crates/app/src/screens/viewer.rs`) add `use_drop(move || session_service.close(id, generation))`. Do not use `spawn`. Verify manually that returning to the dashboard and opening another document closes the first (engine `debug_session_count` via a debug-only log or the smoke-style check described below).

### 5. Tile freshness (app)
- `TileImageState::Ready { uri: String, bucket: ScaleBucket }` (state.rs). `Failed` keeps its current payload (RFC 037 changes it later).
- Extract a pure function in `screens/viewer/render.rs`: `tile_needs_render(state: Option<&TileImageState>, wanted: ScaleBucket) -> bool` — `true` for missing/`Pending`/`Rendering`/`Failed`/`Ready` with a different bucket; `false` only for `Ready` with `bucket == wanted`. Tests in `screens/viewer/render/tests.rs`.
- `schedule_visible_renders` uses it. For a stale `Ready` tile: **do not** overwrite the state (the old image keeps showing); simply request the new render. **Do not add in-flight markers or coalescing** (they interact badly with the existing generation drops; coalescing belongs to the later render-pipeline RFC).
- On completion, store `Ready { uri, bucket: <bucket requested> }` only if the render generation still matches **and** `ScaleBucket::from_scale(*scale.peek()) == requested bucket`.
- `invalidate()` keeps its role (reset non-`Ready` tiles to `Pending`, bump `render_gen`); update its comment.

## Non-change scope
Scheduling policy, prefetch margin, image transport (data URIs stay), search, zoom overlay, keyboard handling, error messages, settings, any engine protocol change.

## Required tests
1. `closed_sessions`: capacity and ordering; `is_closed` semantics.
2. Caches: after `mark_closed`, inserts are refused in render, text and navigation caches; entries are evicted; `evict_session_does_not_suppress_same_generation_for_another_document` equivalents still pass.
3. Render cache: LRU order (a `get` protects an entry), generation-first eviction when `active_generation` is set, budget respected, `evict_generation` still works.
4. `SessionService`: `opened` + `close` evicts everything and refuses late inserts; double `close` is harmless; opening a second session without closing trips the debug assertion (`#[should_panic]` under `debug_assertions`).
5. Smoke test (PDFium, in `tests/smoke/lifecycle.rs`): open → `SessionService::close` equivalent through `EngineHandle` shows `debug_session_count() == 0` and render/search/text/links return `DocumentNotOpen`.
6. `tile_needs_render` truth table (pure).
7. Gate green: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace`; smoke.

## Acceptance criteria (RFC 035 § 8)
- No unreferenced `close`/`evict_*` API remains (grep proof in the review request).
- Manual: open A → dashboard → open B, ×5: native session count stays ≤ 1 (use a temporary debug log, removed before commit) and RSS does not grow by whole-document amounts. Zoom the grid 1.0× → 3.0× → 1.0×: visible tiles sharpen without a blank flash. Record steps and observations.
- Documentation matches code.

## Prohibited shortcuts
Re-introducing per-cache suppression sets; `spawn` in drop; resetting stale tiles to `Pending` (flicker); adding in-flight markers; changing byte budgets; leaving `close_document` callable only from tests.

## Security and robustness
Late async results for a closed session are refused in every cache; no text, link or image data of a closed document stays in memory beyond the cache's own bounds; the closed-session record is bounded.

## Known risks
`use_drop` ordering relative to in-flight tasks: tasks owned by the dropped scope are cancelled, results from tasks that outlive it are refused by the caches. Dioxus `Signal` captures in `use_drop` must be plain values (clone `SessionService`, copy ids).

## Commit plan (each green)
1. `ClosedSessions` + cache refactors; 2. render-cache LRU + tests; 3. `SessionService` + wiring + smoke test; 4. tile freshness + tests; 5. docs/comments.

## Review request
`.git-exclude/review-request/035-lifecycle/README.md` (single entry point): summary, commits, changed files, deviations, commands and results, manual-check records, grep proof, unresolved issues, requested review focus.
