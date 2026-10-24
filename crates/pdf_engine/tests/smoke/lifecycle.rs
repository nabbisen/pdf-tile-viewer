//! Engine smoke tests: lifecycle.

use super::*;
use domain::document::PageIndex;
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};

#[test]
fn close_document_invalidates_session() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();
    assert!(block_on(engine.close_document(session.id)).unwrap());

    let request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    let result = block_on(engine.render_page(request)).unwrap();
    assert!(result.is_err(), "closed document must not render");
}

#[test]
fn closed_session_is_released_and_every_request_reports_document_not_open() {
    use domain::navigation::{
        DocumentOutlineRequest, NavigationError, NavigationResourceLimits, PageLinksRequest,
    };
    use domain::render::RenderError;
    use domain::search::{SearchError, SearchQuery, SearchRequest};
    use domain::text::{TextLayerError, TextLayerRequest};

    let engine = require_engine!();
    // Other smoke tests leave their sessions open in the shared engine; the tests are
    // serialized, so this test only compares against the count it started with.
    let baseline = block_on(engine.debug_session_count()).unwrap();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();
    assert_eq!(
        block_on(engine.debug_session_count()).unwrap(),
        baseline + 1
    );

    // What `SessionService::close` does for the engine: send the command, drop the future.
    drop(engine.close_document(session.id));
    // The worker executes commands in order, so this reply is produced after the close.
    assert_eq!(block_on(engine.debug_session_count()).unwrap(), baseline);

    let render = block_on(engine.render_page(RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    }))
    .unwrap();
    assert_eq!(render.err(), Some(RenderError::DocumentNotOpen));

    let search = block_on(engine.search_document(SearchRequest {
        document_id: session.id,
        generation: session.generation,
        query: SearchQuery::plain("Navigation"),
    }))
    .unwrap();
    assert_eq!(search.err(), Some(SearchError::DocumentNotOpen));

    let text = block_on(engine.extract_page_text_layer(TextLayerRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
    }))
    .unwrap();
    assert_eq!(text.err(), Some(TextLayerError::DocumentNotOpen));

    let links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap();
    assert_eq!(links.err(), Some(NavigationError::DocumentNotOpen));

    let outline = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap();
    assert_eq!(outline.err(), Some(NavigationError::DocumentNotOpen));

    // Closing again is a harmless no-op that reports "nothing was open".
    assert!(!block_on(engine.close_document(session.id)).unwrap());
}
