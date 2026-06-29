---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Proposed
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# RFC-006 — Tile Layout Engine

## 1. Summary

This RFC defines the layout engine for displaying PDF pages as a tiled multi-page overview. It restores the core product identity of PDF Tile Viewer: users can view many pages at once, with adaptive or fixed pages-per-row behavior.

The layout engine is pure domain/application logic. It computes tile positions and sizes from page descriptors, viewport width, scale, gap, and pages-per-row settings. It must not depend on PDFium or Dioxus.

## 2. Motivation

The tile layout is the key differentiator of the app. Moving rendering to PDFium does not change the product goal: users need fast spatial overview of many pages. A pure layout engine enables predictable rendering, scroll targeting, visibility calculations, lazy rendering, and highlight coordinate mapping.

## 3. Goals

- Compute adaptive tile layout for all pages.
- Support auto pages-per-row and fixed pages-per-row.
- Support mixed page sizes and rotations.
- Provide row/visibility data for the lazy render queue.
- Provide scroll target data for jump-to-page.

## 4. Non-Goals

- Actually rendering images.
- Cache eviction.
- Search highlighting.
- Virtual DOM optimization details.

## 5. Layout Inputs

```rust
pub struct TileLayoutInput {
    pub pages: Vec<PageDescriptor>,
    pub viewport_width_px: f32,
    pub scale: ViewerScale,
    pub mode: PagesPerRowMode,
    pub gap_px: f32,
    pub padding_px: f32,
    pub show_page_numbers: bool,
}

pub enum PagesPerRowMode {
    Auto,
    Fixed(u16),
}
```

## 6. Layout Output

```rust
pub struct TileLayout {
    pub generation: LayoutGeneration,
    pub content_width_px: f32,
    pub content_height_px: f32,
    pub rows: Vec<TileRow>,
    pub tiles: Vec<PageTileLayout>,
}

pub struct TileRow {
    pub row_index: usize,
    pub y_px: f32,
    pub height_px: f32,
    pub tile_indices: Vec<PageIndex>,
}

pub struct PageTileLayout {
    pub page_index: PageIndex,
    pub row_index: usize,
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub image_rect_px: RectPx,
    pub label_rect_px: Option<RectPx>,
}
```

## 7. Auto Pages-Per-Row Rule

Auto mode should compute how many pages fit within the available width at the current scale.

Recommended behavior:

1. Choose a representative page width from the median or first page width.
2. Convert page width to tile width using current scale.
3. Compute how many tiles fit with gaps and padding.
4. Clamp to a safe range.

```text
available = viewport_width - padding_left - padding_right
candidate = floor((available + gap) / (tile_width + gap))
pages_per_row = clamp(candidate, 1, max_auto_pages_per_row)
```

The exact formula may be refined, but it must be deterministic and tested.

## 8. Fixed Pages-Per-Row Rule

Fixed mode respects the user-selected number unless it is outside allowed bounds.

```text
fixed = clamp(user_value, 1, max_fixed_pages_per_row)
```

If the viewport becomes too narrow, horizontal overflow is allowed only if the UI intentionally supports it. Otherwise, tile scale or row fitting must adapt. The first release should prefer vertical scroll and avoid horizontal scroll where possible.

## 9. Mixed Page Sizes

Rows must handle pages with different aspect ratios. The row height is the maximum tile height in that row.

```text
row_height = max(tile.height for tile in row) + optional_label_height
```

Tiles in a row may be top-aligned or centered vertically. The design recommends top alignment for predictable reading order.

## 10. Visibility Query

The layout engine should support visibility calculation:

```rust
pub struct ViewportRect {
    pub y_px: f32,
    pub height_px: f32,
}

pub struct VisibilityQueryResult {
    pub visible_pages: Vec<PageIndex>,
    pub prefetch_pages: Vec<PageIndex>,
}
```

This powers RFC-007.

## 11. Jump-to-Page

Jump-to-page should use the tile’s `y_px` as a scroll target. The UI may briefly mark the target page.

```rust
pub fn scroll_target_for_page(layout: &TileLayout, page: PageIndex) -> Option<f32>
```

## 12. Acceptance Criteria

- Layout engine is testable without Dioxus or PDFium.
- Auto mode computes stable rows across common viewport widths.
- Fixed mode produces the expected number of pages per row.
- Mixed page sizes do not overlap.
- Jump-to-page target exists for every page.
- Visibility query returns correct pages for sample viewport ranges.

## 13. Risks

| Risk | Mitigation |
|---|---|
| Layout tied to rendered image dimensions | Use `PageDescriptor` and scale, not bitmap result, as source. |
| Large document layout is expensive | Layout computation should be O(page_count) and cacheable. |
| Mixed page sizes produce visual disorder | Use consistent row alignment and gap policy. |
