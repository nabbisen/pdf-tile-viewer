use crate::document::{PageDescriptor, PageIndex};
use crate::layout::*;

fn a4_pages(n: usize) -> Vec<PageDescriptor> {
    (0..n)
        .map(|i| PageDescriptor {
            page_index: PageIndex(i),
            display_width_points: 595.0,
            display_height_points: 842.0,
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
            display_width_points: 595.0,
            display_height_points: 842.0,
        },
        PageDescriptor {
            page_index: PageIndex(1),
            display_width_points: 842.0,
            display_height_points: 595.0, // landscape, shorter
        },
        PageDescriptor {
            page_index: PageIndex(2),
            display_width_points: 595.0,
            display_height_points: 842.0,
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
fn tile_size_uses_display_dimensions_as_given() {
    // A page whose display geometry is already landscape (e.g. /Rotate 90 applied by the
    // engine) is laid out as landscape: width and height are NOT swapped again (RFC 034).
    let pages = vec![PageDescriptor {
        page_index: PageIndex(0),
        display_width_points: 842.0,
        display_height_points: 595.0,
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

// ── RFC 039: scale arithmetic ────────────────────────────────────────────────

fn assert_close(actual: Option<f32>, expected: Option<f32>) {
    match (actual, expected) {
        (Some(a), Some(e)) => assert!((a - e).abs() < 1e-4, "{a} != {e}"),
        (a, e) => assert_eq!(a, e),
    }
}

#[test]
fn scale_next_steps_up_and_down_from_default() {
    assert_close(ViewerScale::next(1.0, ScaleChange::By(0.2)), Some(1.2));
    assert_close(ViewerScale::next(1.0, ScaleChange::By(-0.2)), Some(0.8));
}

#[test]
fn scale_next_rounds_to_one_decimal() {
    // 1.0 + 0.14 = 1.14 -> 1.1
    assert_close(ViewerScale::next(1.0, ScaleChange::By(0.14)), Some(1.1));
    // repeated 0.2 steps do not drift (0.6000001 style float noise is rounded away)
    let mut current = 1.0_f32;
    for _ in 0..10 {
        current = ViewerScale::next(current, ScaleChange::By(0.2)).expect("below max");
    }
    assert_close(Some(current), Some(3.0));
    // a change smaller than the rounding step is not a change
    assert_eq!(ViewerScale::next(1.0, ScaleChange::By(0.04)), None);
}

#[test]
fn scale_next_clamps_at_both_ends() {
    assert_close(ViewerScale::next(4.9, ScaleChange::By(0.2)), Some(5.0));
    assert_eq!(ViewerScale::next(5.0, ScaleChange::By(0.2)), None);
    assert_close(ViewerScale::next(0.3, ScaleChange::By(-0.2)), Some(0.2));
    assert_eq!(ViewerScale::next(0.2, ScaleChange::By(-0.2)), None);
    assert_close(ViewerScale::next(1.0, ScaleChange::To(9.0)), Some(5.0));
    assert_close(ViewerScale::next(1.0, ScaleChange::To(-3.0)), Some(0.2));
}

#[test]
fn scale_next_to_sets_and_reports_no_change_when_equal() {
    assert_close(ViewerScale::next(1.4, ScaleChange::To(1.0)), Some(1.0));
    assert_eq!(ViewerScale::next(1.0, ScaleChange::To(1.0)), None);
    // equal after rounding counts as equal
    assert_eq!(ViewerScale::next(1.0, ScaleChange::To(1.04)), None);
}

#[test]
fn scale_next_ignores_non_finite_input() {
    assert_eq!(ViewerScale::next(1.0, ScaleChange::By(f32::NAN)), None);
    assert_eq!(ViewerScale::next(1.0, ScaleChange::To(f32::NAN)), None);
}
