use crate::document::PageIndex;
use crate::search::*;

#[test]
fn compact_page_formatting_matches_rfc_010_example() {
    // RFC 010 §10: [0, 3, 4, 5, 9] → "1, 4–6, 10"
    let pages: Vec<PageIndex> = [0, 3, 4, 5, 9].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 4\u{2013}6, 10");
}

#[test]
fn two_adjacent_pages_are_listed_not_ranged() {
    let pages: Vec<PageIndex> = [0, 1].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 2");
}

#[test]
fn unsorted_and_duplicate_input_is_normalized() {
    let pages: Vec<PageIndex> = [4, 0, 4, 2, 3].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 3\u{2013}5");
}

#[test]
fn empty_input_formats_to_empty_string() {
    assert_eq!(format_matched_pages(&[]), "");
}

#[test]
fn minimum_query_policy_requires_two_visible_chars() {
    // RFC 010 §9.
    assert!(!SearchQuery::plain("").is_runnable());
    assert!(!SearchQuery::plain(" a ").is_runnable());
    assert!(SearchQuery::plain("ab").is_runnable());
    assert!(SearchQuery::plain("  ab  ").is_runnable());
}

// ── RFC 011: Coordinate transform tests ──────────────────────────────────────

use crate::document::PageDescriptor;
use crate::geometry::PageRect;
use crate::layout::{PageTileLayout, RectPx, image_rect_to_tile_rect, page_rect_to_image_rect};

fn portrait_page() -> PageDescriptor {
    PageDescriptor {
        page_index: crate::document::PageIndex(0),
        display_width_points: 612.0, // US Letter
        display_height_points: 792.0,
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> PageRect {
    PageRect {
        x,
        y,
        width: w,
        height: h,
    }
}

#[test]
fn page_rect_to_image_rect_scales_proportionally() {
    let page = portrait_page();
    // A rect covering the full page width and the top 10% of the page (display space).
    let r = rect(0.0, 0.0, 612.0, 79.2);
    // Render at 2× scale: 1224×1584px
    let ir = page_rect_to_image_rect(&page, r, 1224, 1584);

    assert!((ir.width - 1224.0).abs() < 1.0, "width {}", ir.width);
    assert!((ir.height - 158.4).abs() < 1.0, "height {}", ir.height);
    assert!(ir.y < 2.0, "y origin near top, got {}", ir.y);
}

#[test]
fn page_rect_to_image_rect_uses_separate_scale_per_axis_without_flip_or_swap() {
    // Display size 792×612 (a landscape page, e.g. /Rotate 90 already applied).
    let page = PageDescriptor {
        page_index: crate::document::PageIndex(0),
        display_width_points: 792.0,
        display_height_points: 612.0,
    };
    // Bitmap 1584×612: scale_x = 2.0, scale_y = 1.0.
    let ir = page_rect_to_image_rect(&page, rect(100.0, 50.0, 40.0, 10.0), 1584, 612);
    assert_eq!(ir.x, 200.0);
    assert_eq!(ir.y, 50.0);
    assert_eq!(ir.width, 80.0);
    assert_eq!(ir.height, 10.0);
}

#[test]
fn image_rect_to_tile_rect_adds_tile_offset() {
    let tile = PageTileLayout {
        page_index: crate::document::PageIndex(0),
        row_index: 0,
        x_px: 200.0,
        y_px: 300.0,
        width_px: 306.0,
        height_px: 396.0,
        image_rect_px: RectPx {
            x: 200.0,
            y: 300.0,
            width: 306.0,
            height: 396.0,
        },
        label_rect_px: None,
    };
    let image_r = RectPx {
        x: 10.0,
        y: 5.0,
        width: 50.0,
        height: 20.0,
    };
    let vp = image_rect_to_tile_rect(image_r, &tile);

    assert_eq!(vp.x, 210.0); // 200 + 10
    assert_eq!(vp.y, 305.0); // 300 + 5
    assert_eq!(vp.width, 50.0);
    assert_eq!(vp.height, 20.0);
}
