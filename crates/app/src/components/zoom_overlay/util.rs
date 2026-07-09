//! Utility helpers for the zoom overlay.

use domain::document::{DocumentGeneration, DocumentId, PageDescriptor, PageIndex};
use domain::layout::{RectPx, page_rect_to_image_rect};
use domain::navigation::{NavigationTarget, PageLinkSet};
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

#[derive(Clone, Debug, PartialEq)]
pub struct ZoomPageLinkRect {
    pub link: domain::navigation::PageLink,
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

/// Compute PDF link hit-test rects for the zoom view using the same page-space
/// transform as search highlights and selectable text.
pub fn zoom_page_link_rects(
    page_index: domain::document::PageIndex,
    page_descriptors: &[PageDescriptor],
    page_links: &PageLinkSet,
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> Vec<ZoomPageLinkRect> {
    if page_links.page_index != page_index {
        return Vec::new();
    }
    let Some(desc) = page_descriptors.iter().find(|d| d.page_index == page_index) else {
        return Vec::new();
    };

    page_links
        .links
        .iter()
        .filter(|link| link.rect.width > 0.0 && link.rect.height > 0.0)
        .map(|link| ZoomPageLinkRect {
            link: link.clone(),
            rect: page_rect_to_image_rect(desc, link.rect, rendered_width_px, rendered_height_px),
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

pub fn zoom_page_links_match_overlay(
    links: &PageLinkSet,
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: PageIndex,
) -> bool {
    links.document_id == document_id
        && links.generation == generation
        && links.page_index == page_index
}

pub fn internal_link_target_at(
    link_rects: &[ZoomPageLinkRect],
    x: f64,
    y: f64,
    page_count: usize,
) -> Option<PageIndex> {
    link_rects.iter().find_map(|positioned| {
        if !rect_contains_point(positioned.rect, x as f32, y as f32) {
            return None;
        }
        let NavigationTarget::InternalDestination(destination) = &positioned.link.target else {
            return None;
        };
        (destination.page_index.0 < page_count).then_some(destination.page_index)
    })
}

pub fn client_point_relative_to_rect(
    client_point: (f64, f64),
    rect_origin: (f64, f64),
) -> (f64, f64) {
    (
        client_point.0 - rect_origin.0,
        client_point.1 - rect_origin.1,
    )
}

pub fn movement_exceeds_click_threshold(
    start: (f64, f64),
    end: (f64, f64),
    threshold_px: f64,
) -> bool {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    (dx * dx + dy * dy) > threshold_px * threshold_px
}

fn rect_contains_point(rect: RectPx, x: f32, y: f32) -> bool {
    x >= rect.x && y >= rect.y && x <= rect.x + rect.width && y <= rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::navigation::{
        DestinationView, NavigationDestination, NavigationLimitStatus, PageLink, PageLinkId,
    };
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
    fn page_links_use_same_transform_as_search_highlights() {
        let page_links = PageLinkSet {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(0),
            links: vec![PageLink {
                id: PageLinkId(7),
                rect: page_rect(),
                target: NavigationTarget::InternalDestination(NavigationDestination {
                    page_index: PageIndex(1),
                    view: DestinationView::PageOnly,
                }),
            }],
            limit_status: NavigationLimitStatus::default(),
        };
        let highlights = vec![PageHighlightSet {
            page_index: PageIndex(0),
            highlights: vec![TextHighlight {
                match_index: 0,
                page_rects: vec![page_rect()],
            }],
        }];

        let link_rects =
            zoom_page_link_rects(PageIndex(0), &[descriptor()], &page_links, 1224, 1584);
        let highlight_rects =
            zoom_highlight_rects(PageIndex(0), &[descriptor()], &highlights, 1224, 1584);

        assert_eq!(link_rects.len(), 1);
        assert_eq!(highlight_rects.len(), 1);
        assert_eq!(link_rects[0].rect, highlight_rects[0]);
    }

    #[test]
    fn page_links_without_page_descriptor_are_not_mounted() {
        let page_links = PageLinkSet {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(0),
            links: vec![PageLink {
                id: PageLinkId(7),
                rect: page_rect(),
                target: NavigationTarget::Disabled(
                    domain::navigation::DisabledNavigationReason::UnsupportedAction,
                ),
            }],
            limit_status: NavigationLimitStatus::default(),
        };

        assert!(zoom_page_link_rects(PageIndex(0), &[], &page_links, 1224, 1584).is_empty());
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

    #[test]
    fn page_link_overlay_guard_checks_document_generation_and_page() {
        let links = PageLinkSet {
            document_id: DocumentId(1),
            generation: DocumentGeneration(2),
            page_index: PageIndex(3),
            links: Vec::new(),
            limit_status: NavigationLimitStatus::default(),
        };

        assert!(zoom_page_links_match_overlay(
            &links,
            DocumentId(1),
            DocumentGeneration(2),
            PageIndex(3),
        ));
        assert!(!zoom_page_links_match_overlay(
            &links,
            DocumentId(9),
            DocumentGeneration(2),
            PageIndex(3),
        ));
        assert!(!zoom_page_links_match_overlay(
            &links,
            DocumentId(1),
            DocumentGeneration(8),
            PageIndex(3),
        ));
        assert!(!zoom_page_links_match_overlay(
            &links,
            DocumentId(1),
            DocumentGeneration(2),
            PageIndex(4),
        ));
    }

    #[test]
    fn internal_link_target_at_returns_supported_internal_target_only() {
        let rects = vec![
            ZoomPageLinkRect {
                link: PageLink {
                    id: PageLinkId(1),
                    rect: page_rect(),
                    target: NavigationTarget::ExternalUri(domain::navigation::ExternalUriTarget {
                        raw_uri: "https://example.test".to_string(),
                    }),
                },
                rect: RectPx {
                    x: 0.0,
                    y: 0.0,
                    width: 20.0,
                    height: 20.0,
                },
            },
            ZoomPageLinkRect {
                link: PageLink {
                    id: PageLinkId(2),
                    rect: page_rect(),
                    target: NavigationTarget::InternalDestination(NavigationDestination {
                        page_index: PageIndex(3),
                        view: DestinationView::PageOnly,
                    }),
                },
                rect: RectPx {
                    x: 25.0,
                    y: 0.0,
                    width: 20.0,
                    height: 20.0,
                },
            },
        ];

        assert_eq!(
            internal_link_target_at(&rects, 30.0, 10.0, 5),
            Some(PageIndex(3))
        );
        assert_eq!(internal_link_target_at(&rects, 10.0, 10.0, 5), None);
        assert_eq!(internal_link_target_at(&rects, 30.0, 10.0, 3), None);
        assert_eq!(internal_link_target_at(&rects, 90.0, 10.0, 5), None);
    }

    #[test]
    fn client_point_relative_to_rect_uses_wrapper_origin_not_event_target_offset() {
        assert_eq!(
            client_point_relative_to_rect((150.0, 90.0), (120.0, 40.0)),
            (30.0, 50.0)
        );
    }

    #[test]
    fn movement_threshold_distinguishes_click_from_drag() {
        assert!(!movement_exceeds_click_threshold(
            (0.0, 0.0),
            (3.0, 0.0),
            4.0
        ));
        assert!(movement_exceeds_click_threshold(
            (0.0, 0.0),
            (5.0, 0.0),
            4.0
        ));
    }
}
