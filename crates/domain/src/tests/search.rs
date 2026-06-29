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
use crate::layout::{PageTileLayout, RectPx, image_rect_to_tile_rect, page_rect_to_image_rect};
use crate::search::{PageCoordinateSpace, PageRect};

fn portrait_page() -> PageDescriptor {
    PageDescriptor {
        page_index: crate::document::PageIndex(0),
        width_points: 612.0, // US Letter
        height_points: 792.0,
        rotation_degrees: 0,
    }
}

fn pdf_rect(x: f32, y: f32, w: f32, h: f32) -> PageRect {
    PageRect {
        x,
        y,
        width: w,
        height: h,
        space: PageCoordinateSpace::PdfPointsBottomLeft,
    }
}

#[test]
fn top_left_conversion_flips_y_axis() {
    // A rect near the top of the page in PDF space (high y value, bottom-left origin)
    // should map to low y in top-left space.
    let page_h = 792.0_f32;
    let r = pdf_rect(72.0, 700.0, 100.0, 20.0);
    let tl = r.to_top_left(page_h);

    assert_eq!(tl.space, PageCoordinateSpace::NormalizedTopLeft);
    assert_eq!(tl.x, 72.0);
    assert!(
        (tl.y - (page_h - 700.0 - 20.0)).abs() < 0.001,
        "expected y≈{}, got {}",
        page_h - 700.0 - 20.0,
        tl.y
    );
    assert_eq!(tl.width, 100.0);
    assert_eq!(tl.height, 20.0);
}

#[test]
fn already_top_left_is_unchanged() {
    let r = PageRect {
        x: 10.0,
        y: 20.0,
        width: 50.0,
        height: 10.0,
        space: PageCoordinateSpace::NormalizedTopLeft,
    };
    let out = r.to_top_left(792.0);
    assert_eq!(out, r);
}

#[test]
fn page_rect_to_image_rect_scales_proportionally() {
    let page = portrait_page();
    // A rect covering the full page width and top 10% of the page.
    let r = pdf_rect(0.0, 712.8, 612.0, 79.2); // top 10%, PDF coords
    // Render at 2× scale: 1224×1584px
    let ir = page_rect_to_image_rect(&page, r, 1224, 1584);

    // Full width → 1224 px
    assert!((ir.width - 1224.0).abs() < 1.0, "width {}", ir.width);
    // Height covers 10% of image height = 158.4 px
    assert!((ir.height - 158.4).abs() < 1.0, "height {}", ir.height);
    // y origin: top 10%, so should be near 0 in image space
    assert!(ir.y < 2.0, "y origin near top, got {}", ir.y);
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
