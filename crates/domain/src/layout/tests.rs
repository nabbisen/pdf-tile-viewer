use crate::document::{PageDescriptor, PageIndex};
use crate::layout::*;

fn a4_pages(n: usize) -> Vec<PageDescriptor> {
    (0..n)
        .map(|i| PageDescriptor {
            page_index: PageIndex(i),
            width_points: 595.0,
            height_points: 842.0,
            rotation_degrees: 0,
        })
        .collect()
}

fn input(pages: Vec<PageDescriptor>, viewport: f32, mode: PagesPerRowMode) -> TileLayoutInput {
    TileLayoutInput {
        pages,
        viewport_width_px: viewport,
        scale: ViewerScale(0.5), // A4 tile: 297.5 px wide
        mode,
        gap_px: 10.0,
        padding_px: 10.0,
        label_height_px: 0.0,
    }
}

#[test]
fn auto_mode_computes_stable_rows_per_rfc_006_formula() {
    // available = 1000 - 20 = 980; floor((980+10)/(297.5+10)) = 3
    let layout = compute_layout(
        &input(a4_pages(7), 1000.0, PagesPerRowMode::Auto),
        LayoutGeneration(1),
    );
    assert_eq!(layout.rows.len(), 3);
    assert_eq!(layout.rows[0].tile_indices.len(), 3);
    assert_eq!(layout.rows[2].tile_indices.len(), 1);
}

#[test]
fn auto_mode_clamps_to_at_least_one_page_per_row() {
    // Viewport narrower than one tile must still place one page per row.
    let layout = compute_layout(
        &input(a4_pages(2), 100.0, PagesPerRowMode::Auto),
        LayoutGeneration(1),
    );
    assert_eq!(layout.rows.len(), 2);
}

#[test]
fn fixed_mode_produces_requested_pages_per_row_clamped() {
    let layout = compute_layout(
        &input(a4_pages(10), 800.0, PagesPerRowMode::Fixed(5)),
        LayoutGeneration(1),
    );
    assert_eq!(layout.rows[0].tile_indices.len(), 5);
    let clamped = compute_layout(
        &input(a4_pages(10), 800.0, PagesPerRowMode::Fixed(0)),
        LayoutGeneration(1),
    );
    assert_eq!(clamped.rows[0].tile_indices.len(), 1);
}

#[test]
fn mixed_page_sizes_do_not_overlap_and_rows_take_max_height() {
    let pages = vec![
        PageDescriptor {
            page_index: PageIndex(0),
            width_points: 595.0,
            height_points: 842.0,
            rotation_degrees: 0,
        },
        PageDescriptor {
            page_index: PageIndex(1),
            width_points: 842.0,
            height_points: 595.0, // landscape, shorter
            rotation_degrees: 0,
        },
        PageDescriptor {
            page_index: PageIndex(2),
            width_points: 595.0,
            height_points: 842.0,
            rotation_degrees: 0,
        },
    ];
    let layout = compute_layout(
        &TileLayoutInput {
            pages,
            viewport_width_px: 5000.0,
            scale: ViewerScale(1.0),
            mode: PagesPerRowMode::Fixed(3),
            gap_px: 10.0,
            padding_px: 0.0,
            label_height_px: 0.0,
        },
        LayoutGeneration(1),
    );
    // Single row whose height is the tallest tile (RFC 006 §9).
    assert_eq!(layout.rows.len(), 1);
    assert_eq!(layout.rows[0].height_px, 842.0);
    // Tiles are laid out left-to-right without overlap.
    let t = &layout.tiles;
    assert!(t[0].x_px + t[0].width_px <= t[1].x_px);
    assert!(t[1].x_px + t[1].width_px <= t[2].x_px);
}

#[test]
fn rotation_90_swaps_effective_tile_dimensions() {
    let pages = vec![PageDescriptor {
        page_index: PageIndex(0),
        width_points: 595.0,
        height_points: 842.0,
        rotation_degrees: 90,
    }];
    let layout = compute_layout(
        &TileLayoutInput {
            pages,
            viewport_width_px: 2000.0,
            scale: ViewerScale(1.0),
            mode: PagesPerRowMode::Fixed(1),
            gap_px: 0.0,
            padding_px: 0.0,
            label_height_px: 0.0,
        },
        LayoutGeneration(1),
    );
    assert_eq!(layout.tiles[0].width_px, 842.0);
    assert_eq!(layout.tiles[0].image_rect_px.height, 595.0);
}

#[test]
fn jump_to_page_target_exists_for_every_page() {
    let layout = compute_layout(
        &input(a4_pages(9), 1000.0, PagesPerRowMode::Auto),
        LayoutGeneration(1),
    );
    for i in 0..9 {
        assert!(scroll_target_for_page(&layout, PageIndex(i)).is_some());
    }
    assert!(scroll_target_for_page(&layout, PageIndex(9)).is_none());
}

#[test]
fn visibility_query_returns_visible_and_prefetch_pages() {
    // 3 per row, row height 421 + 10 gap. Viewport over rows 0–1,
    // prefetch margin reaches row 2.
    let layout = compute_layout(
        &input(a4_pages(12), 1000.0, PagesPerRowMode::Auto),
        LayoutGeneration(1),
    );
    let result = query_visibility(
        &layout,
        ViewportRect {
            y_px: 0.0,
            height_px: 600.0,
        },
        400.0,
    );
    assert!(result.visible_pages.contains(&PageIndex(0)));
    assert!(result.visible_pages.contains(&PageIndex(5)));
    assert!(result.prefetch_pages.contains(&PageIndex(6)));
    assert!(!result.visible_pages.contains(&PageIndex(11)));
}

#[test]
fn scale_bucketing_prevents_cache_fragmentation() {
    use crate::render::ScaleBucket;
    // RFC 007 §5: tiny float differences map to the same bucket.
    assert_eq!(ScaleBucket::from_scale(1.0), ScaleBucket::from_scale(1.001));
    assert_eq!(ScaleBucket::from_scale(1.0).0, 100);
    assert_eq!(ScaleBucket::from_scale(0.2).0, 20);
    assert_ne!(ScaleBucket::from_scale(1.0), ScaleBucket::from_scale(1.2));
}
