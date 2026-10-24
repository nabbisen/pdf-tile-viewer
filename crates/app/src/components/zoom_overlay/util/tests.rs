use super::*;
use domain::geometry::PageRect;
use domain::navigation::{
    DestinationView, NavigationDestination, NavigationLimitStatus, PageLink, PageLinkId,
};
use domain::search::TextHighlight;
use domain::text::TextLayerSegment;

fn descriptor() -> PageDescriptor {
    PageDescriptor {
        page_index: PageIndex(0),
        display_width_points: 612.0,
        display_height_points: 792.0,
    }
}

fn page_rect() -> PageRect {
    PageRect {
        x: 100.0,
        y: 72.0, // display space (top-left origin): near the top of the page
        width: 120.0,
        height: 20.0,
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

    let link_rects = zoom_page_link_rects(PageIndex(0), &[descriptor()], &page_links, 1224, 1584);
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
fn link_activation_at_returns_supported_targets_only() {
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
        link_activation_at(&rects, 30.0, 10.0, 5),
        Some(ZoomLinkActivation::Internal(PageIndex(3)))
    );
    assert_eq!(
        link_activation_at(&rects, 10.0, 10.0, 5),
        Some(ZoomLinkActivation::ExternalUri(
            "https://example.test".to_string()
        ))
    );
    assert_eq!(link_activation_at(&rects, 30.0, 10.0, 3), None);
    assert_eq!(link_activation_at(&rects, 90.0, 10.0, 5), None);
}

#[test]
fn link_activation_at_returns_external_uri_without_authorizing_open() {
    let rects = vec![ZoomPageLinkRect {
        link: PageLink {
            id: PageLinkId(1),
            rect: page_rect(),
            target: NavigationTarget::ExternalUri(domain::navigation::ExternalUriTarget {
                raw_uri: "relative/path".to_string(),
            }),
        },
        rect: RectPx {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        },
    }];

    assert_eq!(
        link_activation_at(&rects, 10.0, 10.0, 5),
        Some(ZoomLinkActivation::ExternalUri("relative/path".to_string()))
    );
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

#[test]
fn keep_previous_when_only_the_scale_changes_on_the_same_page() {
    assert!(keep_previous(
        PageIndex(2),
        ScaleBucket(270),
        PageIndex(2),
        ScaleBucket(300)
    ));
    assert!(keep_previous(
        PageIndex(0),
        ScaleBucket(800),
        PageIndex(0),
        ScaleBucket(500)
    ));
}

#[test]
fn keep_previous_is_false_when_the_page_changes() {
    // Another page's image under the new page number would mislead, whatever the scale.
    assert!(!keep_previous(
        PageIndex(2),
        ScaleBucket(270),
        PageIndex(3),
        ScaleBucket(300)
    ));
    assert!(!keep_previous(
        PageIndex(2),
        ScaleBucket(270),
        PageIndex(3),
        ScaleBucket(270)
    ));
}

#[test]
fn keep_previous_is_false_for_the_same_page_and_bucket() {
    // Nothing to replace: the loader does not start a render for this case.
    assert!(!keep_previous(
        PageIndex(1),
        ScaleBucket(270),
        PageIndex(1),
        ScaleBucket(270)
    ));
}
