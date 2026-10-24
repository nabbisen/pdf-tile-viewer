# Handoff — RFC 034 Display-Space Page Geometry

Governing RFC: [`../../accepted/034-display-space-page-geometry.md`](../../accepted/034-display-space-page-geometry.md)
Executes the RFC; must not redefine it. Conflicts or gaps → design request to the architect.

Order: **Slice C**, after RFC 038 Slice B′ is committed and `main` is green. Working model: directly on `main`, no branches, one actor at a time, hand over with the `main` commit hash. No tags, no releases (DEC-001).

## Purpose
Make the engine the single place that converts PDF user-space geometry into display-space geometry, so that tiles and every overlay (search highlights, text layer, link hit areas) are correct on rotated and cropped pages.

## Background (verified 2026-10-21)
`pdfium-render 0.9.1`: for a page with MediaBox 612×792 and `/Rotate 90`, `page.width()/height()` return **792×612** (rotation already applied) while text-segment bounds stay in unrotated user space (`left=73.8 bottom=699.7 w=166.2 h=17.6` at 0°, 90°, 180°, 270°). The code currently swaps width/height again for 90°/270° and only flips Y.

## Change scope

### 1. Domain (`crates/domain`)
- New `src/geometry.rs` (tests in `src/geometry/tests.rs`): `pub struct PageRect { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }` — **display space, origin top-left, y down, unit PDF points** (document this on the type). Move the type out of `search.rs`; delete `PageCoordinateSpace` and `PageRect::to_top_left`. Update every import (`search`, `text`, `navigation`, `layout`, `app`, `app_services` tests, `pdf_engine`). No re-exports from `search`.
- `document::PageDescriptor`: fields become `page_index`, `display_width_points`, `display_height_points`. Remove `rotation_degrees`. Doc comment: "display geometry; rotation and crop already applied".
- `layout::tile_size` uses display size only (no swap). `layout::page_rect_to_image_rect(page, rect, rendered_w, rendered_h)`: `scale_x = rendered_w / display_width`, `scale_y = rendered_h / display_height`; output `x*scale_x`, … (no flip, no swap).
- `navigation::DestinationView`: `x_points`/`y_points` become display-space, and `FitRectangle(PageRect)` uses the new `PageRect`. Update docs.

### 2. Engine (`crates/pdf_engine`)
- New `src/user_space.rs` (tests in `src/user_space/tests.rs`, **pure, no PDFium handles**):
  - `PageBox { x0, y0, x1, y1 }`; `Rotation { R0, R90, R180, R270 }` (`from_degrees(i32)` via `rem_euclid(360)`; any non-multiple of 90 → `R0`);
  - `display_size(box, rot)`;
  - `point_to_display(box, rot, ux, uy) -> (dx, dy)` and `rect_to_display(box, rot, UserRect) -> Option<PageRect>` using exactly the RFC § 5.3 formulas (transform two opposite corners, normalize to `(min, abs-size)`); return `None` for non-finite values or zero/negative size.
- `PageGeometry { page_box, rotation }` computed once per page at open and stored in `PdfiumBackedSession` (new `pub(crate) fn geometry(&self, id, page_index)`). Page box = `page.boundaries().bounding()` (this wraps `FPDF_GetPageBoundingBox`, which returns the **page box = CropBox ∩ MediaBox in user space**; pdfium-render's doc comment calling it a content bounding box is misleading — verify with the crop-offset fixture below). Rotation = `page.rotation()` as today.
- `collect_page_descriptors` fills `display_width/height_points` from `page.width()/height()` (as today) and compares them with `display_size(page_box, rotation)`. If they differ by more than 0.5 pt, the page is still opened and rendered (never refuse a document over overlay geometry), but its `PageGeometry` is marked inconsistent and `rect_to_display` returns `None` for that page, so no misplaced overlay can appear. Report every such page found in the fixtures or the evidence corpus in the review request; any occurrence is escalated to the architect.
- `search.rs`, `text_layer.rs`, `navigation.rs`: convert every rectangle (and destination coordinates) with `PageGeometry` before returning domain values; drop items whose conversion returns `None`. No other crate may see user-space coordinates.
- `render.rs`: unchanged behavior; it already sizes the bitmap from display dimensions.

### 3. App (`crates/app`)
Mechanical updates only: `tile_grid.rs` and `zoom_overlay/util.rs` use the renamed fields/types and the new `page_rect_to_image_rect`. No UI behavior changes.

### 4. Fixtures (`crates/pdf_engine/tests/fixtures/` — after Slice B′ moved them; if the move did not happen, stop and report)
Extend `generate_fixtures.py` (deterministic, no third-party libraries) with: `rotate-0.pdf`, `rotate-90.pdf`, `rotate-180.pdf`, `rotate-270.pdf` (each: one page 612×792 MediaBox, text `Hello tile viewer` at a known position, one internal link annotation with a known rectangle); `crop-offset.pdf` (MediaBox `[0 0 612 792]`, CropBox `[50 100 562 692]`, text and link inside the crop); `mixed-rotation.pdf` (4 pages: 0/90/180/270). Document expected display rectangles (in the generator docstring and in the tests).

## Non-change scope
Rendering output, settings, UI layout and behavior, search result model beyond coordinates, `UserUnit` (documented as unsupported: treated as 1.0), RFC 033/035/037 topics.

## Required tests
1. `user_space` unit tests: all four rotations (corner mapping, hand-computed), non-zero crop origin, rectangle normalization, degenerate/NaN rejection, `display_size` for each rotation.
2. `domain` tests: update existing layout/search/document tests; **replace** `rotation_90_swaps_effective_tile_dimensions` with a test asserting tile size uses display dimensions as given; `page_rect_to_image_rect` scale test.
3. Smoke tests (PDFium): for each rotated fixture and `crop-offset.pdf`, search for `Hello`, extract the text layer and page links, and assert display rectangles equal the documented expectations within ±0.5 pt; assert the descriptor's display size; **oracle test**: for sample rectangle corners, compare `rect_to_display` against `page.points_to_pixels(x, y, &PdfRenderConfig::new().set_target_width(10000))` scaled back to points, tolerance 0.05 pt.
4. Update `app` tests for renamed fields.
5. Gate green: `cargo fmt --check`; `cargo check --workspace` (0 warnings); `cargo test --workspace`; smoke.

## Acceptance criteria (RFC 034 § 9)
- No `rotation_degrees`, `PageCoordinateSpace`, or `to_top_left` remains outside `pdf_engine::user_space`/the engine session geometry.
- Search highlights, text-layer segments and link rects on all rotated and crop fixtures match expectations; oracle test passes.
- Tile size for 90°/270° fixtures equals the rendered bitmap aspect ratio (assert in a smoke test using the render output size).
- **Manual QA (architect + owner, Linux):** open `mixed-rotation.pdf` and `rotate-90.pdf`; verify tiles have the correct shape, highlights align in tile and zoom views, text selection aligns, link activation hits the right area. Provide screenshots (`niri msg action screenshot-window`, window 1280×720) in the review request.

## Prohibited shortcuts
Keeping user-space rects "for now" anywhere above the engine; special-casing rotations in the UI; adjusting expected values to match output without re-deriving them; loosening tolerances; skipping the oracle test.

## Security
Pure arithmetic on engine-owned values; non-finite and degenerate rectangles are dropped at the conversion site; no new inputs from the document beyond boxes/rotation PDFium already validated.

## Known risks
Unusual page boxes (inverted or zero-size): follow PDFium for rendering and layout; never fail the document because of them; overlays for such pages are dropped (see § 2). Large mechanical rename: do it in a first commit with no behavior change (compiler-guided), then the conversion in later commits.

## Commit plan (each green)
1. `domain::geometry` + renames (behavior unchanged for 0° pages); 2. `user_space` module + tests; 3. fixtures; 4. engine conversion + descriptor cross-check; 5. smoke/oracle tests; 6. app updates and manual QA evidence.

## Review request
`.git-exclude/review-request/034-geometry/README.md` (single entry point): summary, commits, changed files, deviations, commands and results, fixture expectation table, oracle numbers, screenshots, unresolved issues, requested review focus.
