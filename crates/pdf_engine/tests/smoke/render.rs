//! Engine smoke tests: render.

use super::*;
use domain::document::PageIndex;
use domain::render::{
    RenderFlags, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, ScaleBucket,
};

#[test]
fn render_page_one_produces_valid_png() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    let image = block_on(engine.render_page(request)).unwrap().unwrap();

    assert_eq!(image.page_index, PageIndex(0));
    assert!(image.pixel_width > 0 && image.pixel_height > 0);
    // 612pt @ scale 1.0 → target_w 612px (clamp not hit).
    assert_eq!(image.pixel_width, 612, "pixel width");
    let RenderedImagePayload::Bytes(png) = &image.payload;
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG signature");
}

#[test]
fn render_with_stale_generation_is_rejected() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let mut request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    request.generation = domain::document::DocumentGeneration(request.generation.0 + 999);
    let result = block_on(engine.render_page(request)).unwrap();
    assert!(
        result.is_err(),
        "stale generation must be rejected (Appendix A §8)"
    );
}
