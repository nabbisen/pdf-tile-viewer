//! Tile layout engine (RFC 006).
//!
//! Pure function from page descriptors + viewport + settings to tile
//! positions. Depends on neither PDFium nor Dioxus. Rows handle mixed page
//! sizes; tiles are top-aligned for predictable reading order (RFC 006 §9).

use crate::document::{PageDescriptor, PageIndex};

/// Pixels-per-point multiplier applied to page geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewerScale(pub f32);

impl ViewerScale {
    pub const MIN: f32 = 0.2;
    pub const MAX: f32 = 5.0;
    pub const DEFAULT: f32 = 1.0;

    pub fn clamped(value: f32) -> Self {
        ViewerScale(value.clamp(Self::MIN, Self::MAX))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagesPerRowMode {
    Auto,
    Fixed(u16),
}

pub const MAX_AUTO_PAGES_PER_ROW: usize = 16;
pub const MAX_FIXED_PAGES_PER_ROW: u16 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct LayoutGeneration(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct RectPx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileLayoutInput {
    pub pages: Vec<PageDescriptor>,
    pub viewport_width_px: f32,
    pub scale: ViewerScale,
    pub mode: PagesPerRowMode,
    pub gap_px: f32,
    pub padding_px: f32,
    pub label_height_px: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileRow {
    pub row_index: usize,
    pub y_px: f32,
    pub height_px: f32,
    pub tile_indices: Vec<PageIndex>,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct TileLayout {
    pub generation: LayoutGeneration,
    pub content_width_px: f32,
    pub content_height_px: f32,
    pub rows: Vec<TileRow>,
    pub tiles: Vec<PageTileLayout>,
}

/// Effective rendered size of a page at a scale, honoring 90°/270° rotation.
fn tile_size(page: &PageDescriptor, scale: ViewerScale) -> (f32, f32) {
    let rot = page.rotation_degrees.rem_euclid(360);
    let (w, h) = if rot == 90 || rot == 270 {
        (page.height_points, page.width_points)
    } else {
        (page.width_points, page.height_points)
    };
    (w * scale.0, h * scale.0)
}

/// Auto pages-per-row (RFC 006 §7): representative width is the first page.
fn auto_pages_per_row(input: &TileLayoutInput) -> usize {
    let Some(first) = input.pages.first() else {
        return 1;
    };
    let (tile_w, _) = tile_size(first, input.scale);
    let available = (input.viewport_width_px - 2.0 * input.padding_px).max(0.0);
    if tile_w <= 0.0 {
        return 1;
    }
    let candidate = ((available + input.gap_px) / (tile_w + input.gap_px)).floor() as usize;
    candidate.clamp(1, MAX_AUTO_PAGES_PER_ROW)
}

fn pages_per_row(input: &TileLayoutInput) -> usize {
    match input.mode {
        PagesPerRowMode::Auto => auto_pages_per_row(input),
        PagesPerRowMode::Fixed(n) => n.clamp(1, MAX_FIXED_PAGES_PER_ROW) as usize,
    }
}

/// Compute the full tile layout. O(page_count), deterministic.
pub fn compute_layout(input: &TileLayoutInput, generation: LayoutGeneration) -> TileLayout {
    let per_row = pages_per_row(input);
    let mut rows: Vec<TileRow> = Vec::new();
    let mut tiles: Vec<PageTileLayout> = Vec::new();
    let mut content_width: f32 = 0.0;
    let mut cursor_y = input.padding_px;

    for (row_index, chunk) in input.pages.chunks(per_row).enumerate() {
        let sizes: Vec<(f32, f32)> = chunk.iter().map(|p| tile_size(p, input.scale)).collect();
        let image_row_height = sizes.iter().map(|s| s.1).fold(0.0_f32, f32::max);
        let row_height = image_row_height + input.label_height_px;

        let mut cursor_x = input.padding_px;
        let mut tile_indices = Vec::with_capacity(chunk.len());
        for (page, &(w, h)) in chunk.iter().zip(sizes.iter()) {
            let image_rect = RectPx {
                x: cursor_x,
                y: cursor_y,
                width: w,
                height: h,
            };
            let label_rect = (input.label_height_px > 0.0).then_some(RectPx {
                x: cursor_x,
                y: cursor_y + image_row_height,
                width: w,
                height: input.label_height_px,
            });
            tiles.push(PageTileLayout {
                page_index: page.page_index,
                row_index,
                x_px: cursor_x,
                y_px: cursor_y,
                width_px: w,
                height_px: row_height,
                image_rect_px: image_rect,
                label_rect_px: label_rect,
            });
            tile_indices.push(page.page_index);
            cursor_x += w + input.gap_px;
        }
        let row_right = cursor_x - input.gap_px + input.padding_px;
        content_width = content_width.max(row_right);

        rows.push(TileRow {
            row_index,
            y_px: cursor_y,
            height_px: row_height,
            tile_indices,
        });
        cursor_y += row_height + input.gap_px;
    }

    let content_height = if rows.is_empty() {
        2.0 * input.padding_px
    } else {
        cursor_y - input.gap_px + input.padding_px
    };

    TileLayout {
        generation,
        content_width_px: content_width.max(input.viewport_width_px.min(content_width + 1.0)),
        content_height_px: content_height,
        rows,
        tiles,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportRect {
    pub y_px: f32,
    pub height_px: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VisibilityQueryResult {
    pub visible_pages: Vec<PageIndex>,
    pub prefetch_pages: Vec<PageIndex>,
}

/// Visibility query powering the lazy render queue (RFC 006 §10, RFC 007).
/// `prefetch_margin_px` extends the viewport above and below; pages in the
/// margin but not visible are returned as prefetch candidates.
pub fn query_visibility(
    layout: &TileLayout,
    viewport: ViewportRect,
    prefetch_margin_px: f32,
) -> VisibilityQueryResult {
    let vis_top = viewport.y_px;
    let vis_bottom = viewport.y_px + viewport.height_px;
    let pre_top = vis_top - prefetch_margin_px;
    let pre_bottom = vis_bottom + prefetch_margin_px;

    let mut result = VisibilityQueryResult::default();
    for row in &layout.rows {
        let top = row.y_px;
        let bottom = row.y_px + row.height_px;
        if bottom < pre_top || top > pre_bottom {
            continue;
        }
        let visible = bottom >= vis_top && top <= vis_bottom;
        for &page in &row.tile_indices {
            if visible {
                result.visible_pages.push(page);
            } else {
                result.prefetch_pages.push(page);
            }
        }
    }
    result
}

/// Scroll target for jump-to-page (RFC 006 §11).
pub fn scroll_target_for_page(layout: &TileLayout, page: PageIndex) -> Option<f32> {
    layout
        .tiles
        .iter()
        .find(|t| t.page_index == page)
        .map(|t| t.y_px)
}
