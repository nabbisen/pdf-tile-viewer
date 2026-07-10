---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-05
status: Implemented (2.0.0-beta.8)
baseline: PDF Tile Viewer 2.0.0-beta.8 + PDF Tile Viewer v1 text-selection parity
review: .git-exclude/reviewed/pdf_tile_viewer_rfc023_text_selection_design_review.md
rereview: .git-exclude/reviewed/pdf_tile_viewer_rfc023_text_selection_rereview.md
---

# RFC 023 — Single-Page PDF Text Selection Layer

## 1. Summary

Restore best-effort selectable and copyable PDF text in the single-page zoom
overlay by adding a DOM text layer above the PDFium-rendered page bitmap.

The tile-grid view remains image-based. It continues to render search match
badges and visual search highlight rectangles, but it does not mount a
selectable text layer.

PDF Tile Viewer v1 allowed users to select text through the PDF.js viewer. The
Dioxus/PDFium migration renders pages as bitmaps, so the current beta loses
that workflow. This RFC scopes parity recovery to the single-page/zoom view,
where selection is useful, bounded, and closest to v1's single-page reading
behavior.

## 2. Feature Contract

RFC 023 restores best-effort text selection in the single-page zoom overlay. It
allows users to drag-select visible text and copy it using the
platform/WebView's native selection behavior.

It does not guarantee:

- PDF.js-equivalent text fidelity;
- character-perfect geometry;
- semantic reading order for complex layouts;
- selectable text in tile view;
- selectable image regions or screenshot-style rectangular selection.

PDF text is not always semantic document text. Fonts, ligatures, synthetic
spaces, hidden OCR layers, page rotations, clipping, form XObjects, unusual
glyphs, and complex writing modes can all affect extraction and selection.

## 3. Motivation

Text selection is a visible user workflow regression from v1. Users select text
to copy passages, quote or cite content, use system lookup tools, or refine
manual searches. The migrated app should not require users to leave PDF Tile
Viewer for this basic PDF-viewer task.

Full tile-grid text selection is not required to restore the practical
workflow. The tile grid is optimized for overview, search, navigation, and page
scanning. Users can click a page to open the single-page zoom view, then select
text there.

## 4. Goals

- Add selectable and copyable text to the single-page zoom overlay.
- Keep tile-grid search rendering unchanged: match badges plus visual
  highlight rectangles over page bitmaps.
- Extract text through PDFium without mutating PDF bytes.
- Keep PDFium native handles inside `pdf_engine`.
- Reuse the same page geometry path used by search highlights.
- Request/extract text only for the active single-page view, not every tile.
- Keep search highlights visible in both tile view and zoom view.
- Keep extracted text local, memory-only, and out of logs/settings/history.

## 5. Non-Goals

- Selectable text directly in the tile grid.
- Rendering a document-wide text layer.
- PDF.js-level text layout fidelity.
- App-managed rectangular selection or app-managed copy for the RC.
- PDF annotation editing.
- Selecting images or arbitrary rectangular screenshots.
- Link, outline, or bookmark navigation.
- Changing the search result model or replacing search overlays.

## 6. View Responsibilities

### 6.1 Tile Grid

The tile grid remains a bitmap overview. Search result rendering is independent
of the selectable text layer:

```text
page tile
  image bitmap
  search highlight rectangles          yes, visible overlays
  match-count badge                    yes
  page number label                    optional
  selectable text layer                no
```

Hard invariant:

> Tile-grid search highlighting must not depend on text-layer extraction. The
> tile grid never requires a `PageTextLayer`; only the zoom overlay requests
> one.

Search and text selection may both use PDFium text APIs and shared page
geometry, but they must not share lifecycle state.

### 6.2 Single-Page / Zoom Overlay

The zoom overlay becomes the text-selection surface:

```text
zoom page
  image bitmap
  selectable text layer                yes, for select/copy
  search highlight rectangles          yes, visible overlays
```

Only the active zoom page needs a text layer. Closing the overlay may drop the
text layer from UI state. A small memory-only per-session cache is recommended
for repeated page inspection.

## 7. Proposed Architecture

```text
app / Dioxus
  ZoomOverlay
    - displays page bitmap
    - requests text layer on open/page change
    - renders TextSelectionLayer over bitmap
    - renders SearchHighlightLayer with pointer-events:none
    - applies result only if overlay page still matches

app_services
  TextLayerService
    - request orchestration
    - bounded session cache
    - stale-result filtering
    - memory-only lifecycle

pdf_engine
  Worker command: ExtractPageTextLayer
    - owns PDFium handles
    - extracts page text segments
    - converts PDFium bounds into domain PageRect
    - returns domain-owned PageTextLayer

domain::text
  TextLayerRequest
  PageTextLayer
  TextLayerSegment
```

## 8. Text Extraction

Add a page text-layer request to the PDF engine worker. The worker must:

1. Validate `document_id`, `generation`, and `page_index`.
2. Load `page.text()` for the requested page.
3. Extract text segments and segment bounds.
4. Convert each bound into the existing `PageRect` coordinate model.
5. Return domain-owned data only; PDFium handles must not cross the worker
   boundary.

The first implementation should prefer segment-level extraction via
`PdfPageText::segments()`. If segment text order or selection behavior fails
basic smoke testing, the implementation may fall back to finer granularity, but
per-character DOM is not the default RC design.

## 9. Domain Model

Add a dedicated text-layer module. Recommended path:

```text
crates/domain/src/text.rs
```

Do not add text-layer data to `domain::search`. Search asks "where are
matches?" Text selection asks "what page text fragments can the user select?"

Initial model:

```rust
pub struct TextLayerRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
}

pub struct PageTextLayer {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub segments: Vec<TextLayerSegment>,
}

pub struct TextLayerSegment {
    pub segment_index: u32,
    pub text: String,
    pub rect: PageRect,
}
```

`segment_index` is required for deterministic tests, stable Dioxus keys, and a
future app-managed copy fallback.

If the PDFium segment index type is wider than `u32`, conversion to
`segment_index: u32` must be checked. Overflow should fail extraction for that
page with a non-text-leaking error. If that constraint becomes awkward during
implementation, the model may use `usize` instead.

`PageRect` currently lives under `domain::search`. For RC, RFC 023 may reuse it
directly to avoid churn. Longer term, page geometry should move to a neutral
module such as `domain::geometry` if more non-search features consume it.

## 10. Geometry Invariants

Text segment rectangles and search rectangles must use the same page coordinate
space and the same page descriptor/render geometry when mapped to the displayed
bitmap.

The implementation must not fork page rotation, crop, or CSS scaling behavior
for text. One geometry path must serve:

- search highlights in tile grid;
- search highlights in zoom overlay;
- text selection layer in zoom overlay.

Mapping must target the displayed image coordinate system. If a bitmap's
natural pixel dimensions differ from its CSS display size, the text layer must
align to the displayed image rectangle, not only the raw bitmap dimensions.

## 11. DOM Selection and Copy Semantics

The RC implementation uses native WebView selection and copy behavior.

Rules:

- The app does not intercept copy in RFC 023.
- Users copy selected text with normal platform/browser commands such as
  `Ctrl+C` or `Cmd+C`.
- A future RFC may add app-managed copy if cross-platform native selection is
  insufficient.
- Extracted text must not be logged.
- Error messages must not include PDF text snippets.
- The first implementation renders text segments in deterministic extraction
  order. Native copied text therefore follows DOM segment order, not visual
  geometry. For RC, this order is best effort and may differ from visual
  reading order on complex PDFs.

Simple one-column fixture tests must verify acceptable word and line extraction
order. A future app-managed copy RFC may introduce visual-order reconstruction.

Zoom page stacking order, bottom to top:

```text
1. page bitmap image
2. search highlight overlay, pointer-events:none
3. transparent selectable text spans
4. native selection highlight, produced by WebView
```

The text-selection layer must be constrained to the rendered page image
rectangle. It must not cover close buttons, page navigation controls, zoom
controls, search controls, or other overlay chrome.

Expected CSS behavior:

```css
.zoom-page-image {
  user-select: none;
}

.search-highlight-layer {
  pointer-events: none;
}

.text-selection-layer {
  position: absolute;
  inset: 0;
  user-select: text;
}

.text-selection-segment {
  position: absolute;
  color: transparent;
  white-space: pre;
  user-select: text;
}

.text-selection-segment::selection {
  background: rgba(80, 130, 255, 0.35);
  color: transparent;
}
```

The exact CSS can change, but these behavioral constraints must hold:

- the page bitmap is not selectable;
- visible search highlights do not intercept pointer events;
- text spans participate in hit testing and selection;
- `display: none`, `visibility: hidden`, and `pointer-events: none` are invalid
  for selectable text spans;
- text spans use stable keys such as `(page_index, segment_index)`;
- changing zoom scale should update layout/style while preserving stable text
  segment keys and DOM nodes where practical;
- replacing the active text layer is allowed for document or page changes;
- zoom-only changes should not intentionally re-key all segments.

## 12. Cache Policy

Use a small bounded memory-only cache owned by `app_services`.

```text
Cache key:   (document_id, generation, page_index)
Owner:       app_services
Contents:    domain-owned PageTextLayer only
Lifetime:    document session lifetime
Eviction:    oldest-by-insertion under a small memory estimate cap
Invalidation: document close, generation mismatch, explicit session reset
Persistence: none
```

This avoids repeated PDFium extraction when users reopen the same zoom page,
while preventing document-wide or persistent text storage.

For the RC implementation, oldest-by-insertion eviction is intentionally
accepted to match the existing render cache style and keep the text-selection
slice small. A true recency-updating LRU cache remains a possible follow-up if
manual reading workflows show repeated hot-page eviction.

## 13. Async Stale-Result Handling

Generation tagging is required but not sufficient. The UI must also check the
current zoom page before applying an async result.

Required rule:

> A returned `PageTextLayer` may be applied only if `(document_id, generation,
> page_index)` still matches the currently open zoom overlay page at the moment
> of application. Otherwise it is dropped silently.

An app-local request id may also be used to guard against rapid page changes:

```rust
pub struct TextLayerRequestId(u64);
```

The request id does not need to be part of the domain model.

## 14. Failure-State UX

Text-layer loading must not block the zoom overlay.

Recommended RC behavior:

- While text layer loads: keep the page usable as an image; no blocking spinner
  is required.
- On extraction failure: show a small, non-modal message such as
  "Text selection is unavailable for this page."
- For image-only or scanned PDFs with no text: show the same unavailable state
  or a no-op state.
- Search highlights remain available even if the selection layer fails.
- The zoom overlay remains closable and navigable.

## 15. Security and Privacy

RFC 023 creates a new local path that exposes PDF text into the WebView DOM and
to the system clipboard through explicit user copy behavior.

Rules:

- Extracted text remains local.
- Extracted text is not logged.
- Extracted text is not written to settings, history, diagnostics, or disk
  cache.
- Text-layer cache is memory-only and cleared with the document session.
- Clipboard changes occur only through explicit user selection/copy behavior.
- Error messages must not include PDF text snippets.
- Diagnostics may record page index, segment count, extraction duration, and
  error category, but must not record segment text or copied content.

## 16. Fidelity Limitations

Segment-level extraction is acceptable for RC, but selection is best effort.

Unsupported or best-effort cases include:

- complex multi-column reading order;
- vertical writing systems;
- rotated text inside an otherwise normal page;
- ligatures and synthetic spaces;
- hidden OCR text behind scanned images;
- clipped text;
- unusual fonts or Type3 glyphs;
- forms/annotation text if not part of the page text extraction path.

These limitations do not block the feature unless the app claims stronger text
fidelity.

## 17. Implementation Plan

1. Add `domain::text` request/result types.
2. Add `pdf_engine::text_layer` extraction code.
3. Add `EngineHandle::extract_page_text_layer()` and a worker command.
4. Add an `app_services` text-layer service with a bounded memory-only cache.
5. Add zoom-overlay state for the active page's `PageTextLayer` and request id.
6. Render the text layer in `ZoomOverlay` using the shared page/image geometry
   path.
7. Keep `TileGrid` unchanged.
8. Document the feature as single-page/zoom text selection.

## 18. Acceptance Criteria

### 18.1 Design Acceptance

- RFC scopes selection to the zoom overlay.
- RFC excludes tile-grid text layers.
- RFC uses a dedicated text/selection domain model.
- RFC states native DOM selection is the initial copy mechanism.
- RFC states PDF.js-equivalent fidelity is not guaranteed.
- RFC defines cache ownership, lifetime, invalidation, and persistence policy.
- RFC defines stale-result rejection.
- RFC defines platform manual smoke gates.

### 18.2 Implementation Acceptance

- Users can open a page in the zoom overlay and drag-select visible PDF text.
- Users can copy selected text with the platform/browser copy command.
- Tile-grid view does not mount selectable text layers.
- Tile-grid search results continue to show match badges and highlight
  rectangles.
- Zoom-overlay search highlights continue to render when search results exist.
- Text layer requests are limited to the active zoom page.
- Stale text-layer results are discarded by document, generation, page, and
  request identity as appropriate.
- Extracted text does not appear in logs, settings, history, diagnostics, or
  persistent cache.

## 19. Test and Gate Requirements

### 19.1 Automated Gates

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace --exclude app`
- `cargo test -p app`
- PDFium smoke tests with `PDF_TILE_VIEWER_PDFIUM_DIR`

### 19.2 Feature Tests

- PDF engine test extracts non-empty text-layer segments from a fixture PDF.
- Generation mismatch test rejects stale text-layer requests.
- App/service test verifies stale results are not applied to the wrong active
  zoom page.
- Cache test verifies bounded memory-only eviction/invalidation behavior.
- Coordinate transform test verifies text segment rectangles map through the
  same path as search highlight rectangles.
- Existing search highlight tests remain independent of text-layer tests.

### 19.3 Manual RC Smoke Matrix

Native WebView selection is a blocking RC smoke gate.

Platforms:

- Windows WebView2
- macOS WKWebView
- Linux WebKitGTK

Minimum cases:

1. Simple one-column text page: drag-select one word, one line, and multiple
   lines; copy/paste to a text editor.
2. Zoomed page: selection alignment remains acceptable after zoom overlay
   scaling.
3. Search + selection together: search highlights remain visible and do not
   intercept selection.
4. Page navigation while text layer is loading: stale layer is not applied to
   the wrong page.
5. Rotated page: selection layer does not grossly misalign, or the limitation
   is documented.
6. Scanned/image-only page: app remains stable and shows unavailable/no-op
   behavior.

If native selection/copy fails on any required platform, RFC 023 cannot be
marked RC-ready. The project must either revise RFC 023 to include app-managed
copy for that platform or defer the feature from the RC.

## 20. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Transparent absolutely positioned spans do not select/copy consistently across WebViews | High | Blocking manual smoke on Windows/macOS/Linux; keep app-managed copy as future fallback. |
| Segment geometry does not align with rendered text | Medium | Reuse the existing page geometry path; add fixture and manual smoke coverage. |
| Dioxus rerender destroys active selection | High | Stable segment keys; avoid state churn during active selection; smoke copy after search/zoom state changes. |
| Search becomes coupled to text layer | High | Dedicated `domain::text`; hard invariant that tile grid never requests `PageTextLayer`. |
| Stale async layer appears on wrong page | High | Match document, generation, page, and optional request id before apply. |
| Extracted text leaks to logs/history | High | Security rules plus review of logging and persistence paths. |
| Complex PDFs copy in surprising order | Medium | Document best-effort fidelity; future RFC for app-managed copy/text ordering if needed. |
| Character-level fallback expands scope too much | Medium | Segment-first; fallback only after RC smoke failure. |
