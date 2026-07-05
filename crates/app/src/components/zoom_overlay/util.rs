//! Utility helpers for the zoom overlay.

use domain::document::{DocumentGeneration, DocumentId, PageDescriptor, PageIndex};
use domain::layout::{RectPx, page_rect_to_image_rect};
use domain::render::ScaleBucket;
use domain::search::PageHighlightSet;
use domain::text::{PageTextLayer, TextLayerSegment};

/// Extract PNG image dimensions from the IHDR chunk without full decode.
pub fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes(data[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(data[20..24].try_into().ok()?);
    Some((w, h))
}

/// Compute RFC 011 highlight overlay rects for the zoom view.
///
/// Returns image-space `RectPx` values ready to position as overlay `<div>`s.
pub fn zoom_highlight_rects(
    page_index: domain::document::PageIndex,
    page_descriptors: &[PageDescriptor],
    search_highlights: &[PageHighlightSet],
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> Vec<RectPx> {
    let ph = match search_highlights
        .iter()
        .find(|p| p.page_index == page_index)
    {
        Some(ph) => ph,
        None => return Vec::new(),
    };
    let desc = match page_descriptors.iter().find(|d| d.page_index == page_index) {
        Some(d) => d,
        None => return Vec::new(),
    };
    ph.highlights
        .iter()
        .flat_map(|h| &h.page_rects)
        .map(|pr| page_rect_to_image_rect(desc, *pr, rendered_width_px, rendered_height_px))
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZoomTextSegmentRect {
    pub segment: TextLayerSegment,
    pub rect: RectPx,
}

/// Compute selectable text segment rects for the zoom view using the same
/// page-space transform as search highlights.
pub fn zoom_text_segment_rects(
    page_index: domain::document::PageIndex,
    page_descriptors: &[PageDescriptor],
    text_layer: &PageTextLayer,
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> Vec<ZoomTextSegmentRect> {
    if text_layer.page_index != page_index {
        return Vec::new();
    }
    let Some(desc) = page_descriptors.iter().find(|d| d.page_index == page_index) else {
        return Vec::new();
    };

    text_layer
        .segments
        .iter()
        .filter(|segment| segment.rect.width > 0.0 && segment.rect.height > 0.0)
        .map(|segment| ZoomTextSegmentRect {
            segment: segment.clone(),
            rect: page_rect_to_image_rect(
                desc,
                segment.rect,
                rendered_width_px,
                rendered_height_px,
            ),
        })
        .collect()
}

pub fn zoom_image_result_is_current(
    current_request_id: u64,
    request_id: u64,
    active_page: Option<PageIndex>,
    requested_page: PageIndex,
    active_scale: f32,
    requested_scale_bucket: ScaleBucket,
) -> bool {
    current_request_id == request_id
        && active_page == Some(requested_page)
        && ScaleBucket::from_scale(active_scale) == requested_scale_bucket
}

pub fn zoom_text_layer_matches_overlay(
    layer: &PageTextLayer,
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: PageIndex,
) -> bool {
    layer.document_id == document_id
        && layer.generation == generation
        && layer.page_index == page_index
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::search::{PageCoordinateSpace, PageRect, TextHighlight};
    use domain::text::TextLayerSegment;

    fn descriptor() -> PageDescriptor {
        PageDescriptor {
            page_index: PageIndex(0),
            width_points: 612.0,
            height_points: 792.0,
            rotation_degrees: 0,
        }
    }

    fn page_rect() -> PageRect {
        PageRect {
            x: 100.0,
            y: 700.0,
            width: 120.0,
            height: 20.0,
            space: PageCoordinateSpace::PdfPointsBottomLeft,
        }
    }

    #[test]
    fn text_segments_use_same_transform_as_search_highlights() {
        let text_layer = PageTextLayer {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(0),
            segments: vec![TextLayerSegment {
                segment_index: 0,
                text: "Hello".to_string(),
                rect: page_rect(),
            }],
        };
        let highlights = vec![PageHighlightSet {
            page_index: PageIndex(0),
            highlights: vec![TextHighlight {
                match_index: 0,
                page_rects: vec![page_rect()],
            }],
        }];

        let text_rects =
            zoom_text_segment_rects(PageIndex(0), &[descriptor()], &text_layer, 1224, 1584);
        let highlight_rects =
            zoom_highlight_rects(PageIndex(0), &[descriptor()], &highlights, 1224, 1584);

        assert_eq!(text_rects.len(), 1);
        assert_eq!(highlight_rects.len(), 1);
        assert_eq!(text_rects[0].rect, highlight_rects[0]);
    }

    #[test]
    fn text_segments_without_page_descriptor_are_not_mounted() {
        let text_layer = PageTextLayer {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(0),
            segments: vec![TextLayerSegment {
                segment_index: 0,
                text: "Hello".to_string(),
                rect: page_rect(),
            }],
        };

        assert!(zoom_text_segment_rects(PageIndex(0), &[], &text_layer, 1224, 1584).is_empty());
    }

    #[test]
    fn image_result_guard_rejects_stale_page_or_scale() {
        assert!(zoom_image_result_is_current(
            4,
            4,
            Some(PageIndex(1)),
            PageIndex(1),
            2.7,
            ScaleBucket::from_scale(2.7),
        ));
        assert!(!zoom_image_result_is_current(
            5,
            4,
            Some(PageIndex(1)),
            PageIndex(1),
            2.7,
            ScaleBucket::from_scale(2.7),
        ));
        assert!(!zoom_image_result_is_current(
            4,
            4,
            Some(PageIndex(2)),
            PageIndex(1),
            2.7,
            ScaleBucket::from_scale(2.7),
        ));
        assert!(!zoom_image_result_is_current(
            4,
            4,
            Some(PageIndex(1)),
            PageIndex(1),
            3.1,
            ScaleBucket::from_scale(2.7),
        ));
    }

    #[test]
    fn text_layer_overlay_guard_checks_document_generation_and_page() {
        let layer = PageTextLayer {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(3),
            segments: Vec::new(),
        };

        assert!(zoom_text_layer_matches_overlay(
            &layer,
            DocumentId(1),
            DocumentGeneration(2),
            PageIndex(3),
        ));
        assert!(!zoom_text_layer_matches_overlay(
            &layer,
            DocumentId(9),
            DocumentGeneration(2),
            PageIndex(3),
        ));
        assert!(!zoom_text_layer_matches_overlay(
            &layer,
            DocumentId(1),
            DocumentGeneration(8),
            PageIndex(3),
        ));
        assert!(!zoom_text_layer_matches_overlay(
            &layer,
            DocumentId(1),
            DocumentGeneration(2),
            PageIndex(4),
        ));
    }
}
