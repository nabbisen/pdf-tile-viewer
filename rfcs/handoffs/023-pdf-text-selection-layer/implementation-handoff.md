# Handoff — RFC 023 Single-Page PDF Text Selection Layer

## 1. Summary

RFC 023 has been revised as a design-first RC issue. The target is selectable
and copyable PDF text in the single-page zoom overlay only. Tile-grid view
remains image-based and continues to render search results as visual overlays.

## 2. Scope followed

- Paused creating additional RFCs.
- Analyzed the existing PDFium search/highlight/render architecture.
- Updated `rfcs/proposed/023-pdf-text-selection-layer.md`.
- Updated `rfcs/README.md` to match the revised RFC title.
- Started implementation in small slices:
  - added `domain::text` request/result/cache-key/error types;
  - added PDFium text-layer extraction through the serialized worker;
  - added `app_services::text_service` with bounded memory-only caching and
    generation-close stale-result suppression;
  - wired `TextLayerService` into the app context;
  - added zoom-overlay text-layer request state with request-id/current-page
    stale-result guarding;
  - rendered transparent selectable text spans above the zoom page bitmap and
    search highlights;
  - patched UI review findings before manual smoke: natural-size zoom page
    geometry, stale bitmap render guarding, non-selectable/non-draggable page
    bitmap, and explicit document/generation/page validation before applying
    text-layer results.

## 3. Files changed

- `rfcs/proposed/023-pdf-text-selection-layer.md`
- `rfcs/README.md`
- `rfcs/handoffs/023-pdf-text-selection-layer/implementation-handoff.md`
- `crates/domain/src/text.rs`
- `crates/domain/src/text/tests.rs`
- `crates/pdf_engine/src/text_layer.rs`
- `crates/pdf_engine/src/worker.rs`
- `crates/pdf_engine/tests/smoke.rs`
- `crates/app_services/src/text_service.rs`
- `crates/app_services/src/text_service/tests.rs`
- `crates/app/src/app.rs`
- `crates/app/src/components/zoom_overlay.rs`
- `crates/app/src/components/zoom_overlay/util.rs`
- `crates/app/assets/main.css`
- `crates/app/src/i18n.rs`
- `crates/app/src/i18n/en.rs`
- `crates/app/src/i18n/ja.rs`

## 4. Design decisions and assumptions

- Architect review result:
  `.git-exclude/reviewed/pdf_tile_viewer_rfc023_text_selection_design_review.md`
  returned **Approve with mandatory changes before implementation**.
- Architect rereview result:
  `.git-exclude/reviewed/pdf_tile_viewer_rfc023_text_selection_rereview.md`
  returned **Approved to proceed to implementation prototype**, after a small
  mandatory documentation patch. That patch has been applied to RFC 023.
- Text selection is scoped to the zoom overlay / single-page view.
- Tile-grid text selection is a non-goal for this RFC.
- Tile-grid search rendering remains unchanged: page bitmap, match-count badge,
  and search highlight rectangles.
- Zoom overlay should render page bitmap, selectable text layer, and search
  highlight rectangles.
- Text extraction should be requested only for the active zoom page.
- Text-layer data belongs in a dedicated `domain::text` module, not
  `domain::search`.
- Segment-level extraction via PDFium text segments is the preferred first
  implementation. Character-level extraction is a fallback if segment selection
  quality is unacceptable.
- Text-layer geometry should reuse `PageDescriptor`, `PageRect`, and
  `page_rect_to_image_rect()` rather than introducing a second coordinate path.
- Initial copy behavior relies on native WebView selection/copy; the app does
  not intercept copy for RFC 023.
- Native copied text follows DOM segment order in the first implementation.
  This is best effort and may differ from visual reading order in complex PDFs.
- A bounded memory-only `app_services` cache is recommended, keyed by
  `(document_id, generation, page_index)`. For RC, eviction is
  oldest-by-insertion under a memory estimate cap, matching the implemented
  service and existing render cache style. True recency-updating LRU remains a
  possible follow-up if reading workflows need it.
- Closed-session stale-result suppression is keyed by `(document_id,
  generation)`, not generation alone.
- PDFium extraction skips empty text segments and segments with non-positive
  bounds before returning domain text-layer data.
- Manual Windows/macOS/Linux WebView selection smoke is a blocking RC gate.
- Zoom-only changes should preserve stable segment keys and DOM nodes where
  practical; document/page changes may replace the layer.
- Zoom page layer order is: bitmap, search highlights with
  `pointer-events:none`, transparent selectable text spans, native selection
  highlight.

## 5. Tests and gates run

Observed after the backend extraction slice:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p domain`
- `cargo check -p pdf_engine`
- `cargo test -p pdf_engine --test smoke`
- `cargo check --workspace`
- `cargo test -p app`
- `cargo test --workspace --exclude app`

Observed after the app-services cache slice:

- `cargo fmt`
- `cargo test -p domain`
- `cargo test -p app_services`

Observed after backend review follow-up patches:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p domain`
- `cargo test -p app_services`
- `cargo test -p pdf_engine --test smoke`
- `cargo check --workspace`
- `cargo test -p app`
- `cargo test --workspace --exclude app`

Observed after the ZoomOverlay UI slice:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p app`
- `cargo test -p app_services`
- `cargo check --workspace`
- `cargo test --workspace --exclude app`

Observed after UI implementation review follow-up patches:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p app`
- `cargo test -p app_services`
- `cargo check --workspace`
- `cargo test --workspace --exclude app`

Broader gates should be rerun after each follow-up slice. Manual WebView
selection smoke was reported successful by the maintainer before the
`2.0.0-beta.8` release-prep update.

## 6. Generated artifacts

None.

## 7. Known limitations

- Rotated/cropped page behavior is identified as a risk and needs fixture or
  manual coverage.
- The implementation approach assumes native selection can work over
  absolutely positioned text spans in Dioxus Desktop/WebView.
- The first architect review noted that the review tarball did not include the
  RFC file itself. Future review packages should include the primary RFC in the
  archive, not only review context.

## 8. Recommended next step

Prepare and publish `2.0.0-beta.8`:

1. Commit the RFC 023 UI and release-prep changes.
2. Run the release workflow from tag `2.0.0-beta.8`.
3. After release, verify the published archive layout still keeps `bin/` and
   `resources/` together and includes bundled PDFium.
4. Keep real-world PDF fidelity checks open for rotated/cropped pages and
   complex text ordering.
5. Consider adding a PDFium-backed `TextLayerService::get_or_extract()` smoke
   test before RC.
