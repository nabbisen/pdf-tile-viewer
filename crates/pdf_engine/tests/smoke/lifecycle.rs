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
