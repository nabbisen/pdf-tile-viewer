---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Implemented in the 2.0.2 train — awaiting release (the architect moves it to done/ after the release)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 004, RFC 006, RFC 011, RFC 023, RFC 026
amends: RFC 011 (§ 6–§ 10 coordinate transforms, rotation), RFC 004 (PageDescriptor), RFC 006 (rotation)
origin: architect onboarding review 2026-10-21 (internal)
---

# RFC 034 — Display-Space Page Geometry

## 1. Summary

Make the PDF engine the **single place** that converts PDF user-space geometry
into **display-space** geometry (rotation and crop applied). Everything above
the engine — layout, tiles, search highlights, text layer, link hit areas — then
consumes one coordinate system and contains no rotation logic.

## 2. Motivation (verified)

Experiment (`pdfium-render 0.9.1`, MediaBox 612×792, text at user space
(73.8, 699.7)):

| `/Rotate` | `page.width() × height()` | text segment bounds (left, bottom, w, h) |
|---:|---|---|
| 0 | 612 × 792 | 73.8, 699.7, 166.2, 17.6 |
| 90 | **792 × 612** | 73.8, 699.7, 166.2, 17.6 |
| 180 | 612 × 792 | 73.8, 699.7, 166.2, 17.6 |
| 270 | **792 × 612** | 73.8, 699.7, 166.2, 17.6 |

Consequences in 2.0.0:

1. Page size is already **rotation-applied**, but `domain::layout::tile_size`
   and `page_rect_to_image_rect` swap width/height again for 90°/270°. Rotated
   tiles get the wrong aspect ratio (image letterboxed in a wrong-shaped box)
   and overlay scale factors are swapped.
2. Text, search and link rectangles are **always unrotated user space**; the
   code only flips Y. On any rotated page every overlay is misplaced (180° is
   mirrored; 90°/270° are transposed).
3. RFC 011 requires rotation fixtures to be "covered or explicitly documented
   as unsupported". Neither is true: the fixture generator has no `/Rotate`.

## 3. Goals

- One coordinate contract, one conversion site, no rotation math outside
  `pdf_engine`.
- Correct tiles and overlays for `/Rotate` 0/90/180/270 and for pages whose
  crop box origin is not (0, 0).
- Fixtures and tests that make the contract executable.

## 4. Non-goals

- `UserUnit` scaling (treated as 1.0 and documented as unsupported).
- Arbitrary non-orthogonal rotation (not representable in PDF page `/Rotate`).
- Changing rendering output (PDFium already renders rotated pages correctly).

## 5. Design

### 5.1 Coordinate spaces

| Space | Definition | Where it exists |
|---|---|---|
| User space | PDF native; unrotated; origin bottom-left of the page box | inside `pdf_engine` only |
| Display space | After `/Rotate` and crop; origin **top-left**; y down; unit = PDF points | all of `domain`, `app_services`, `app` |
| Image space | Display space scaled to the rendered bitmap | UI only |

### 5.2 Domain changes

- `PageDescriptor` carries **display** size only:
  `display_width_points`, `display_height_points`. The `rotation_degrees` field
  is removed from the descriptor (nothing above the engine may use it for
  geometry).
- `PageRect` has a single meaning: display space, top-left origin, points. The
  `PageCoordinateSpace` enum and `PageRect::to_top_left` are removed.
- `PageRect` (and its text/navigation consumers) moves from `domain::search` to
  a neutral `domain::geometry` module (resolves the RFC 023 note).
- `layout::tile_size` and `page_rect_to_image_rect` use display size only:
  `scale = rendered_px / display_points`; no flip, no swap.
- `DestinationView` coordinates (`x_points`, `y_points`, `FitRectangle`) are
  converted to display space by the same function (they were user space).

### 5.3 Engine conversion

A pure module `pdf_engine::user_space` (no PDFium handles) implements:

```text
let (x0, y0, x1, y1) = page box in user space (PDFium's page bounding box)
let W = x1 - x0, H = y1 - y0
display size: r = 0/180 → (W, H);   r = 90/270 → (H, W)

point (ux, uy) → display (dx, dy):
  r =   0:  dx = ux - x0   dy = y1 - uy
  r =  90:  dx = uy - y0   dy = ux - x0
  r = 180:  dx = x1 - ux   dy = uy - y0
  r = 270:  dx = y1 - uy   dy = x1 - ux
```

A rectangle is converted by transforming two opposite corners and normalizing
to `(min, abs-size)`. The formulas are derived analytically from the corner
mapping of a clockwise page rotation; the implementation **must** cross-check
them against PDFium (`FPDF_PageToDevice` at a high pixel size to keep integer
rounding below 0.01 pt) in a smoke test.

`search`, `text_layer` and `navigation` extraction call this module before
returning domain values. No other crate sees user-space coordinates.

### 5.4 Fixtures

Extend `fixtures/generate_fixtures.py` (deterministic, no third-party
libraries) with: one page each at `/Rotate` 0, 90, 180, 270 with text at a known
position; a link annotation; a page with a non-zero crop-box origin; and a
mixed-rotation multi-page file. Expected display-space rectangles are asserted
within ±0.5 pt.

## 6. Compatibility

Internal API change across crates (types listed in § 5.2). No persisted data,
settings, or user-visible API is affected. RFC 004, 006, 011 receive an
"amended by RFC 034" note (front matter) in the same change.

## 7. Alternatives considered

- **Keep user-space rects and add rotation to `page_rect_to_image_rect`.**
  Rejected: every consumer (tile highlights, zoom text, links, future features)
  would need to pass the right rotation and crop; the error already happened.
- **Convert in the UI via JavaScript/CSS transforms.** Rejected: logic outside
  Rust and outside tests.
- **Use `FPDF_PageToDevice` for every rectangle.** Rejected as the primary
  mechanism: integer device coordinates lose precision; kept as a test oracle.

## 8. Security

No new data flows. Conversion is pure arithmetic on engine-owned values; inputs
are already validated by PDFium. Degenerate/NaN rectangles are dropped at the
conversion site.

## 9. Tests and acceptance

- Pure unit tests for all four rotations, non-zero crop origin, rectangle
  normalization, and round-trip invariants.
- Smoke tests: search, text-layer and link rects on every rotated fixture match
  expected display rects; cross-check against `FPDF_PageToDevice`.
- Layout tests: tile size for 90°/270° equals the rendered bitmap aspect ratio.
- Manual QA: highlights, text selection and link activation visually aligned on
  rotated fixtures in tile and zoom views.
- Acceptance: no `rotation` or `to_top_left` remains outside `pdf_engine`;
  RFC 011 § 10 is satisfied by committed fixtures.

## 10. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Page box semantics differ for odd crop/media combinations | Medium | Use PDFium's own page bounding box; fixture with offset crop box; cross-check oracle |
| Large mechanical type rename | Low | Compiler-guided; single handoff slice |
