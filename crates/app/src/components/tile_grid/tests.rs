use super::*;
use domain::geometry::PageRect;
use domain::render::ScaleBucket;
use domain::search::TextHighlight;

fn descriptor() -> PageDescriptor {
    PageDescriptor {
        page_index: PageIndex(0),
        display_width_points: 612.0,
        display_height_points: 792.0,
    }
}

fn highlights() -> PageHighlightSet {
    PageHighlightSet {
        page_index: PageIndex(0),
        highlights: vec![TextHighlight {
            match_index: 0,
            page_rects: vec![PageRect {
                x: 0.0,
                y: 0.0, // display space: the top 10% of the page
                width: 612.0,
                height: 79.2,
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
        &TileImageState::Ready {
            uri: "data:image/png;base64,".to_string(),
            bucket: ScaleBucket(200),
        },
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
        &TileImageState::Ready {
            uri: "data:image/png;base64,".to_string(),
            bucket: ScaleBucket(200),
        },
        1224,
        1584,
    );

    assert!(rects.is_empty());
}
