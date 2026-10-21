use super::*;
use domain::search::{PageCoordinateSpace, PageRect, TextHighlight};

fn descriptor() -> PageDescriptor {
    PageDescriptor {
        page_index: PageIndex(0),
        width_points: 612.0,
        height_points: 792.0,
        rotation_degrees: 0,
    }
}

fn highlights() -> PageHighlightSet {
    PageHighlightSet {
        page_index: PageIndex(0),
        highlights: vec![TextHighlight {
            match_index: 0,
            page_rects: vec![PageRect {
                x: 0.0,
                y: 712.8,
                width: 612.0,
                height: 79.2,
                space: PageCoordinateSpace::PdfPointsBottomLeft,
            }],
        }],
    }
}

#[test]
fn tile_highlights_use_page_descriptor_transform() {
    let rects = tile_highlight_rects(
        PageIndex(0),
        &[descriptor()],
        Some(&highlights()),
        &TileImageState::Ready("data:image/png;base64,".to_string()),
        1224,
        1584,
    );

    assert_eq!(rects.len(), 1);
    assert!((rects[0].width - 1224.0).abs() < 1.0);
    assert!((rects[0].height - 158.4).abs() < 1.0);
    assert!(rects[0].y < 2.0);
}

#[test]
fn tile_highlights_do_not_fallback_to_raw_pdf_rects() {
    let rects = tile_highlight_rects(
        PageIndex(0),
        &[],
        Some(&highlights()),
        &TileImageState::Ready("data:image/png;base64,".to_string()),
        1224,
        1584,
    );

    assert!(rects.is_empty());
}
