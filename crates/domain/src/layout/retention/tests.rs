use crate::document::{PageDescriptor, PageIndex};
use crate::layout::*;

/// One page per row, scale 1, no gap/padding/label: page `i` occupies `[100 i, 100 i + 100]`.
fn column(n: usize) -> TileLayout {
    column_with_heights(&vec![100.0; n])
}

fn column_with_heights(heights: &[f32]) -> TileLayout {
    let pages = heights
        .iter()
        .enumerate()
        .map(|(i, &h)| PageDescriptor {
            page_index: PageIndex(i),
            display_width_points: 100.0,
            display_height_points: h,
        })
        .collect();
    compute_layout(
        &TileLayoutInput {
            pages,
            viewport_width_px: 100.0,
            scale: ViewerScale(1.0),
            mode: PagesPerRowMode::Fixed(1),
            gap_px: 0.0,
            padding_px: 0.0,
            label_height_px: 0.0,
        },
        LayoutGeneration(0),
    )
}

fn pages(range: std::ops::Range<usize>) -> Vec<PageIndex> {
    range.map(PageIndex).collect()
}

fn limits(window_px: f32, max_ready_tiles: usize) -> RetentionLimits {
    RetentionLimits {
        window_px,
        max_ready_tiles,
    }
}

fn viewport(y_px: f32, height_px: f32) -> ViewportRect {
    ViewportRect { y_px, height_px }
}

fn indices(victims: &[PageIndex]) -> Vec<usize> {
    victims.iter().map(|p| p.0).collect()
}

#[test]
fn default_limits_follow_the_viewport_height() {
    let small = RetentionLimits::for_viewport(400.0);
    assert_eq!(small.window_px, 1500.0, "floor of 1500 px");
    assert_eq!(small.max_ready_tiles, 80);
    let large = RetentionLimits::for_viewport(1000.0);
    assert_eq!(large.window_px, 2000.0, "2 viewport heights");
    assert_eq!(RetentionLimits::for_viewport(750.0).window_px, 1500.0);
}

#[test]
fn the_window_is_larger_than_the_prefetch_margin() {
    // The scheduler prefetches 600 px around the viewport; retention must never be tighter.
    assert!(RetentionLimits::for_viewport(100.0).window_px > 600.0);
}

#[test]
fn with_min_cap_only_raises_the_cap() {
    let base = limits(1000.0, 80);
    assert_eq!(base.with_min_cap(200).max_ready_tiles, 200);
    assert_eq!(base.with_min_cap(10).max_ready_tiles, 80);
    assert_eq!(base.with_min_cap(200).window_px, 1000.0);
}

#[test]
fn tiles_beyond_the_window_are_victims_and_the_boundary_is_inclusive() {
    let layout = column(30);
    // viewport [1000, 1200], window 300 -> kept region [700, 1500]
    let victims = retention_victims(
        &layout,
        viewport(1000.0, 200.0),
        &pages(0..30),
        limits(300.0, 1000),
    );
    // page 6 = [600, 700] touches the edge 700 -> kept; page 5 = [500, 600] -> victim
    // page 15 = [1500, 1600] touches the edge 1500 -> kept; page 16 -> victim
    assert_eq!(indices(&victims), (0..6).chain(16..30).collect::<Vec<_>>());
}

#[test]
fn a_tile_one_pixel_outside_the_window_is_a_victim() {
    let layout = column(30);
    // kept region [700.5, 1499.5]: page 6 ([600,700]) and page 15 ([1500,1600]) fall outside
    let victims = retention_victims(
        &layout,
        viewport(1000.5, 199.0),
        &[PageIndex(6), PageIndex(7), PageIndex(14), PageIndex(15)],
        limits(300.0, 1000),
    );
    assert_eq!(indices(&victims), vec![6, 15]);
}

#[test]
fn visible_tiles_are_never_victims() {
    let layout = column(30);
    // Cap 0 would evict everything; the tiles intersecting the viewport must stay.
    let victims = retention_victims(
        &layout,
        viewport(1000.0, 200.0),
        &pages(0..30),
        limits(10_000.0, 0),
    );
    let kept: Vec<usize> = (0..30)
        .filter(|i| !victims.contains(&PageIndex(*i)))
        .collect();
    // viewport [1000, 1200]: page 9 ([900,1000]) touches the top, page 12 ([1200,1300]) the bottom
    assert_eq!(kept, vec![9, 10, 11, 12]);
}

#[test]
fn a_tile_partly_in_the_viewport_counts_as_visible() {
    let layout = column(30);
    let victims = retention_victims(
        &layout,
        viewport(1050.0, 100.0), // [1050, 1150] cuts pages 10 and 11
        &pages(0..30),
        limits(10_000.0, 0),
    );
    assert!(!victims.contains(&PageIndex(10)));
    assert!(!victims.contains(&PageIndex(11)));
}

#[test]
fn the_cap_removes_the_farthest_tiles_first() {
    let layout = column(30);
    // viewport [1000, 1100] -> centre 1050; visible: pages 9 (touches 1000), 10, 11 (touches 1100)
    // Kept distances from the centre: page 12 (1250) = 200, page 8 (850) = 200 ... use an
    // asymmetric ready set so there is no tie.
    let ready = [8, 9, 10, 11, 12, 13, 14, 20].map(PageIndex);
    let victims = retention_victims(
        &layout,
        viewport(1000.0, 100.0),
        &ready,
        limits(10_000.0, 5),
    );
    // 8 ready tiles, cap 5 -> 3 victims: the farthest are page 20 (1_950-1_050 = 900),
    // page 14 (1_450 -> 400) and page 13 (1_350 -> 300)
    assert_eq!(indices(&victims), vec![13, 14, 20]);
}

#[test]
fn equal_distances_are_resolved_deterministically_by_page_index() {
    let layout = column(30);
    // viewport centre 1050 is the middle of page 10; pages 9 and 11 are visible (touching),
    // pages 8 and 12 are equally far (200). One of them must go: the higher index first.
    let ready = [8, 10, 12].map(PageIndex);
    let victims = retention_victims(
        &layout,
        viewport(1000.0, 100.0),
        &ready,
        limits(10_000.0, 2),
    );
    assert_eq!(indices(&victims), vec![12]);
}

#[test]
fn result_does_not_depend_on_the_order_of_the_ready_list() {
    let layout = column(60);
    let ordered = pages(0..60);
    let mut shuffled = ordered.clone();
    shuffled.reverse();
    shuffled.rotate_left(17);
    let v = viewport(2000.0, 500.0);
    let l = limits(800.0, 12);
    let a = retention_victims(&layout, v, &ordered, l);
    let b = retention_victims(&layout, v, &shuffled, l);
    assert_eq!(a, b);
    assert!(a.windows(2).all(|w| w[0] < w[1]), "sorted, no duplicates");
}

#[test]
fn after_applying_the_plan_the_cap_holds() {
    let layout = column(200);
    let ready = pages(0..200);
    let limits = limits(100_000.0, 25);
    let victims = retention_victims(&layout, viewport(5000.0, 600.0), &ready, limits);
    assert_eq!(ready.len() - victims.len(), 25);
}

#[test]
fn empty_ready_list_and_empty_layout() {
    let layout = column(10);
    assert!(retention_victims(&layout, viewport(0.0, 500.0), &[], limits(500.0, 80)).is_empty());

    let empty = column(0);
    let victims = retention_victims(
        &empty,
        viewport(0.0, 500.0),
        &[PageIndex(0)],
        limits(500.0, 80),
    );
    assert_eq!(
        indices(&victims),
        vec![0],
        "a page the layout does not know is released"
    );
}

#[test]
fn a_single_row_layout_keeps_everything_near_the_viewport() {
    let layout = compute_layout(
        &TileLayoutInput {
            pages: (0..6)
                .map(|i| PageDescriptor {
                    page_index: PageIndex(i),
                    display_width_points: 50.0,
                    display_height_points: 100.0,
                })
                .collect(),
            viewport_width_px: 1000.0,
            scale: ViewerScale(1.0),
            mode: PagesPerRowMode::Fixed(6),
            gap_px: 0.0,
            padding_px: 0.0,
            label_height_px: 0.0,
        },
        LayoutGeneration(0),
    );
    assert_eq!(layout.rows.len(), 1);
    let victims = retention_victims(
        &layout,
        viewport(0.0, 100.0),
        &pages(0..6),
        limits(10.0, 80),
    );
    assert!(
        victims.is_empty(),
        "one row, all six tiles intersect the viewport"
    );
}

#[test]
fn rows_with_several_tiles_are_released_together() {
    // Two pages per row: both tiles of a row share its vertical extent.
    let layout = compute_layout(
        &TileLayoutInput {
            pages: (0..20)
                .map(|i| PageDescriptor {
                    page_index: PageIndex(i),
                    display_width_points: 50.0,
                    display_height_points: 100.0,
                })
                .collect(),
            viewport_width_px: 100.0,
            scale: ViewerScale(1.0),
            mode: PagesPerRowMode::Fixed(2),
            gap_px: 0.0,
            padding_px: 0.0,
            label_height_px: 0.0,
        },
        LayoutGeneration(0),
    );
    let victims = retention_victims(
        &layout,
        viewport(0.0, 100.0),
        &pages(0..20),
        limits(200.0, 80),
    );
    // rows 0..=3 (y 0..400) intersect [-200, 300]: row 3 = [300, 400] touches -> pages 0..8 kept
    assert_eq!(indices(&victims), (8..20).collect::<Vec<_>>());
}

#[test]
fn mixed_page_sizes_use_the_row_extent_of_each_tile() {
    // heights 100, 400, 100, 100, 100 -> tops 0, 100, 500, 600, 700
    let layout = column_with_heights(&[100.0, 400.0, 100.0, 100.0, 100.0]);
    let victims = retention_victims(
        &layout,
        viewport(900.0, 100.0), // [900, 1000]
        &pages(0..5),
        limits(300.0, 80), // keep [600, 1300]
    );
    // page 1 = [100, 500] and page 2 = [500, 600] (touches 600 -> kept? bottom 600 >= 600 yes)
    assert_eq!(indices(&victims), vec![0, 1]);
}

#[test]
fn tiles_of_a_tall_page_that_reach_into_the_window_are_kept() {
    // A very tall page is kept as long as any part of it lies inside the window.
    let layout = column_with_heights(&[100.0, 5000.0, 100.0]);
    let victims = retention_victims(
        &layout,
        viewport(4900.0, 100.0),
        &pages(0..3),
        limits(200.0, 80),
    );
    assert_eq!(indices(&victims), vec![0]);
}
