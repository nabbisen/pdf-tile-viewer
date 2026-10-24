//! Engine smoke tests: display-space geometry (RFC 034).
//!
//! Expected values come from the fixture table in the docstring of
//! `fixtures/generate_fixtures.py`, computed by hand from the §5.3 formulas — not from the
//! engine's own conversion code. Tolerance 0.5 pt.

use super::*;
use domain::document::{DocumentSession, PageIndex};
use domain::geometry::PageRect;
use domain::navigation::{
    DestinationView, NavigationResourceLimits, NavigationTarget, PageLinksRequest,
};
use domain::render::{
    RenderFlags, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, ScaleBucket,
};
use domain::search::{SearchQuery, SearchRequest};
use domain::text::TextLayerRequest;

const TOLERANCE: f32 = 0.5;

fn open(engine: &EngineHandle, name: &str) -> DocumentSession {
    block_on(engine.open_document(fixture(name)))
        .unwrap()
        .unwrap()
}

fn assert_rect(actual: PageRect, expected: (f32, f32, f32, f32), what: &str) {
    let (x, y, w, h) = expected;
    assert!(
        (actual.x - x).abs() <= TOLERANCE
            && (actual.y - y).abs() <= TOLERANCE
            && (actual.width - w).abs() <= TOLERANCE
            && (actual.height - h).abs() <= TOLERANCE,
        "{what}: got ({}, {}, {}, {}), expected ({x}, {y}, {w}, {h})",
        actual.x,
        actual.y,
        actual.width,
        actual.height
    );
}

fn links(
    engine: &EngineHandle,
    session: &DocumentSession,
    page: usize,
) -> Vec<domain::navigation::PageLink> {
    block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(page),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap()
    .links
}

fn text_rect(engine: &EngineHandle, session: &DocumentSession, page: usize) -> PageRect {
    let layer = block_on(engine.extract_page_text_layer(TextLayerRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(page),
    }))
    .unwrap()
    .unwrap();
    assert_eq!(layer.segments.len(), 1, "one text run per fixture page");
    assert_eq!(layer.segments[0].text, "Hello tile viewer");
    layer.segments[0].rect
}

/// Link rect `[72 650 240 680]` on a 612x792 page, by rotation (docstring table).
const LINK_RECT: [(i32, (f32, f32, f32, f32)); 4] = [
    (0, (72.0, 112.0, 168.0, 30.0)),
    (90, (650.0, 72.0, 30.0, 168.0)),
    (180, (372.0, 650.0, 168.0, 30.0)),
    (270, (112.0, 372.0, 30.0, 168.0)),
];

/// Destination (100, 500) as (display x, display y), by rotation.
const DESTINATION: [(i32, (f32, f32)); 4] = [
    (0, (100.0, 292.0)),
    (90, (500.0, 100.0)),
    (180, (512.0, 500.0)),
    (270, (292.0, 512.0)),
];

/// Hand-written rotation of a user-space rect `(left, bottom, right, top)` on a 612x792
/// page into display `(x, y, w, h)`, independent of `user_space`.
fn expected_display(rotation: i32, l: f32, b: f32, r: f32, t: f32) -> (f32, f32, f32, f32) {
    match rotation {
        0 => (l, 792.0 - t, r - l, t - b),
        90 => (b, l, t - b, r - l),
        180 => (612.0 - r, b, r - l, t - b),
        270 => (792.0 - t, 612.0 - r, t - b, r - l),
        other => panic!("unexpected rotation {other}"),
    }
}

fn expect_coordinates(view: &DestinationView, expected: (f32, f32), what: &str) {
    match view {
        DestinationView::CoordinatesAndZoom {
            x_points: Some(x),
            y_points: Some(y),
            ..
        } => assert!(
            (x - expected.0).abs() <= TOLERANCE && (y - expected.1).abs() <= TOLERANCE,
            "{what}: got ({x}, {y}), expected {expected:?}"
        ),
        other => panic!("{what}: expected coordinates, got {other:?}"),
    }
}

#[test]
fn descriptors_report_display_size_for_every_rotation_and_crop() {
    let engine = require_engine!();
    for (rotation, (w, h)) in [
        (0, (612.0, 792.0)),
        (90, (792.0, 612.0)),
        (180, (612.0, 792.0)),
        (270, (792.0, 612.0)),
    ] {
        let session = open(&engine, &format!("rotate-{rotation}.pdf"));
        let page = session.pages[0];
        assert!(
            (page.display_width_points - w).abs() <= TOLERANCE
                && (page.display_height_points - h).abs() <= TOLERANCE,
            "rotate-{rotation}: {}x{}",
            page.display_width_points,
            page.display_height_points
        );
    }

    let crop = open(&engine, "crop-offset.pdf");
    assert!((crop.pages[0].display_width_points - 512.0).abs() <= TOLERANCE);
    assert!((crop.pages[0].display_height_points - 592.0).abs() <= TOLERANCE);

    let mixed = open(&engine, "mixed-rotation.pdf");
    let sizes: Vec<(f32, f32)> = mixed
        .pages
        .iter()
        .map(|p| {
            (
                p.display_width_points.round(),
                p.display_height_points.round(),
            )
        })
        .collect();
    assert_eq!(
        sizes,
        vec![
            (612.0, 792.0),
            (792.0, 612.0),
            (612.0, 792.0),
            (792.0, 612.0)
        ]
    );
}

#[test]
fn link_rects_and_destinations_are_in_display_space_for_every_rotation() {
    let engine = require_engine!();
    for ((rotation, rect), (_, destination)) in LINK_RECT.into_iter().zip(DESTINATION) {
        let session = open(&engine, &format!("rotate-{rotation}.pdf"));
        let links = links(&engine, &session, 0);
        assert_eq!(links.len(), 1, "rotate-{rotation}");
        assert_rect(links[0].rect, rect, &format!("rotate-{rotation} link rect"));
        match &links[0].target {
            NavigationTarget::InternalDestination(d) => {
                assert_eq!(d.page_index, PageIndex(0));
                expect_coordinates(
                    &d.view,
                    destination,
                    &format!("rotate-{rotation} destination"),
                );
            }
            other => panic!("rotate-{rotation}: expected internal destination, got {other:?}"),
        }
    }
}

#[test]
fn crop_offset_page_subtracts_the_crop_origin() {
    let engine = require_engine!();
    let session = open(&engine, "crop-offset.pdf");
    let links = links(&engine, &session, 0);
    assert_eq!(links.len(), 1);
    assert_rect(links[0].rect, (22.0, 252.0, 168.0, 40.0), "crop link rect");
    match &links[0].target {
        NavigationTarget::InternalDestination(d) => {
            expect_coordinates(&d.view, (50.0, 192.0), "crop destination");
        }
        other => panic!("expected internal destination, got {other:?}"),
    }

    // The text run (user space 72,600) lies inside the crop box and the displayed page.
    let text = text_rect(&engine, &session, 0);
    assert!(
        text.x >= 0.0 && text.right() <= 512.0,
        "text x within crop: {text:?}"
    );
    assert!(
        text.y >= 0.0 && text.bottom() <= 592.0,
        "text y within crop: {text:?}"
    );
    // The glyph box starts 1.8 pt right of the text origin (73.8 in user space, the value the
    // RFC 034 experiment measured on an uncropped page) → display x = 73.8 - 50.
    assert!((text.x - 23.8).abs() <= TOLERANCE, "text x {}", text.x);
}

#[test]
fn text_layer_and_search_agree_with_hand_computed_rects_for_every_rotation() {
    let engine = require_engine!();

    // The unrotated page is the reference for the glyph box in user space.
    let base_session = open(&engine, "rotate-0.pdf");
    let base = text_rect(&engine, &base_session, 0);
    assert!(
        (base.x - 73.8).abs() <= TOLERANCE,
        "glyph box starts at user x 73.8: {base:?}"
    );
    assert!(
        base.height > 10.0 && base.height < 40.0,
        "plausible 24 pt glyph box: {base:?}"
    );
    let (left, right) = (base.x, base.x + base.width);
    let top = 792.0 - base.y;
    let bottom = top - base.height;
    assert!(
        bottom < 700.0 && top > 700.0,
        "baseline origin y=700 inside the box: {base:?}"
    );

    for rotation in [0, 90, 180, 270] {
        let session = open(&engine, &format!("rotate-{rotation}.pdf"));
        let expected = expected_display(rotation, left, bottom, right, top);

        let text = text_rect(&engine, &session, 0);
        assert_rect(text, expected, &format!("rotate-{rotation} text layer"));

        let (_, highlights) = block_on(engine.search_document_with_highlights(SearchRequest {
            document_id: session.id,
            generation: session.generation,
            query: SearchQuery::plain("Hello tile viewer"),
        }))
        .unwrap()
        .unwrap();
        assert_eq!(highlights.pages.len(), 1, "rotate-{rotation}");
        let rects = &highlights.pages[0].highlights[0].page_rects;
        assert!(!rects.is_empty());
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for r in rects {
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.right());
            max_y = max_y.max(r.bottom());
        }
        let union = PageRect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        };
        assert_rect(
            union,
            expected,
            &format!("rotate-{rotation} search highlight"),
        );
    }
}

#[test]
fn destination_uses_the_geometry_of_the_destination_page() {
    let engine = require_engine!();
    let session = open(&engine, "mixed-rotation.pdf");
    // Page N links to page (N+1) % 4; the destination (100, 500) is converted with the
    // rotation of the *target* page, not of the page holding the link.
    for (page, rotation) in [(0usize, 90), (1, 180), (2, 270), (3, 0)] {
        let links = links(&engine, &session, page);
        assert_eq!(links.len(), 1, "page {page}");
        let source_rotation = LINK_RECT[page].0;
        assert_rect(
            links[0].rect,
            LINK_RECT[page].1,
            &format!("page {page} (rotate {source_rotation}) link rect"),
        );
        let destination = DESTINATION.iter().find(|(r, _)| *r == rotation).unwrap().1;
        match &links[0].target {
            NavigationTarget::InternalDestination(d) => {
                assert_eq!(d.page_index, PageIndex((page + 1) % 4));
                expect_coordinates(&d.view, destination, &format!("page {page} destination"));
            }
            other => panic!("page {page}: expected internal destination, got {other:?}"),
        }
    }
}

#[test]
fn rendered_bitmap_aspect_ratio_matches_the_descriptor_for_quarter_turns() {
    let engine = require_engine!();
    for rotation in [0, 90, 180, 270] {
        let session = open(&engine, &format!("rotate-{rotation}.pdf"));
        let page = session.pages[0];
        let image = block_on(engine.render_page(RenderPageRequest {
            document_id: session.id,
            generation: session.generation,
            page_index: PageIndex(0),
            scale_bucket: ScaleBucket::from_scale(1.0),
            flags: RenderFlags::default(),
            format: RenderOutputFormat::Png,
        }))
        .unwrap()
        .unwrap();
        let RenderedImagePayload::Bytes(png) = &image.payload;
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");

        // The tile is laid out at display size × scale, so the bitmap must have exactly that
        // size (scale 1.0) and the descriptor's aspect ratio.
        assert_eq!(
            image.pixel_width,
            page.display_width_points.round() as u32,
            "rotate-{rotation}"
        );
        assert_eq!(
            image.pixel_height,
            page.display_height_points.round() as u32,
            "rotate-{rotation}"
        );
        let descriptor_ratio = page.display_width_points / page.display_height_points;
        let bitmap_ratio = image.pixel_width as f32 / image.pixel_height as f32;
        assert!(
            (descriptor_ratio - bitmap_ratio).abs() < 0.002,
            "rotate-{rotation}"
        );
    }
}
