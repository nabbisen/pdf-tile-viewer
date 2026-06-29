---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Implemented
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# RFC-011 — Search Highlight Coordinates and Rendering

## 1. Summary

This RFC defines exact search highlight rendering for matched text locations. It builds on RFC-010 page-level search by adding match rectangles, coordinate transformations, and visual highlight rendering in tile and zoom contexts.

## 2. Motivation

Page-level search markers help users find relevant pages, but users often need to see where the match occurs on the page. The old architecture could produce highlighted PDF bytes. The target architecture should instead keep search highlights as application overlay data or optional baked render flags.

## 3. Goals

- Extract match rectangles or bounding boxes from PDFium search/text APIs.
- Define canonical coordinate systems.
- Transform PDF coordinates to rendered tile/zoom coordinates.
- Render highlights accurately over page images.
- Test common page rotations and page sizes.

## 4. Non-Goals

- Persisting highlights into PDF files.
- Annotation editing.
- General text selection layer.
- Regex capture highlighting.

## 5. Highlight Data Model

```rust
pub struct SearchHighlightSet {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub query: SearchQuery,
    pub pages: Vec<PageHighlightSet>,
}

pub struct PageHighlightSet {
    pub page_index: PageIndex,
    pub highlights: Vec<TextHighlight>,
}

pub struct TextHighlight {
    pub match_index: usize,
    pub page_rects: Vec<PageRect>,
}

pub struct PageRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub coordinate_space: PageCoordinateSpace,
}

pub enum PageCoordinateSpace {
    PdfPointsBottomLeft,
    NormalizedTopLeft,
}
```

The implementation must choose and document one canonical coordinate space for app-owned data. The recommendation is to normalize to top-left page coordinates after extraction.

## 6. Coordinate Transformation

```text
PDFium text rectangle
    ↓ normalize rotation / coordinate origin
Canonical PageRect
    ↓ scale to rendered page image dimensions
Image-space RectPx
    ↓ translate by tile image_rect position
Viewport-space RectPx
```

## 7. Transform Function Contract

```rust
pub fn page_rect_to_image_rect(
    page: &PageDescriptor,
    rect: PageRect,
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> RectPx;

pub fn image_rect_to_tile_rect(
    image_rect: RectPx,
    tile: &PageTileLayout,
) -> RectPx;
```

## 8. Overlay vs Baked Highlight Decision

Recommended first release: overlay rectangles.

| Option | Pros | Cons | Policy |
|---|---|---|---|
| Overlay rectangles | Fast to update; search does not force rerender | Requires precise coordinate transform | Preferred |
| Baked highlights in rendered image | Visually stable if PDFium draws highlight | Requires rerender when search changes | Optional later |

## 9. Visual Design

- Highlights should be translucent enough to keep text readable.
- Matched page marker from RFC-010 should remain visible.
- Current/selected search match may use a stronger outline.
- Highlight color should meet contrast expectations but not obscure content.

## 10. Rotation and Crop Handling

The implementation must test at least:

- Portrait page, 0° rotation.
- Landscape page, 0° rotation.
- Page with 90° rotation.
- Page with multiple text matches on one line.
- Page with matches spanning multiple rectangles.

If PDF crop boxes are used, the canonical page geometry must account for them consistently.

## 11. Acceptance Criteria

- Search highlights align with text on fixture PDFs at default scale.
- Highlights remain aligned after scale changes.
- Highlights remain aligned in zoom overlay.
- Rotation fixtures are covered or explicitly documented as unsupported.
- Clearing search removes overlay highlights without rerendering the PDF image.
- Search highlight state does not mutate PDF bytes.

## 12. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| Coordinate systems are misunderstood | Define canonical coordinates and test transforms. |
| Highlights misalign after scaling | Use rendered image dimensions and tile image rects, not CSS guesses. |
| Rotated/cropped pages are difficult | Support common cases first; gate edge cases with fixtures. |

## 13. Release Policy

If exact highlight alignment is not reliable by the release deadline, ship RFC-010 page-level markers and mark RFC-011 as beta/deferred. Do not ship misleading highlights.
