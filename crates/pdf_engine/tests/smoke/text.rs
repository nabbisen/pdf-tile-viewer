//! Engine smoke tests: text.

use super::*;
use domain::document::PageIndex;
use domain::text::{TextLayerError, TextLayerRequest};

#[test]
fn text_layer_extracts_segments_for_single_page() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = TextLayerRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
    };
    let layer = block_on(engine.extract_page_text_layer(request))
        .unwrap()
        .unwrap();

    assert_eq!(layer.document_id, session.id);
    assert_eq!(layer.generation, session.generation);
    assert_eq!(layer.page_index, PageIndex(0));
    assert!(!layer.segments.is_empty(), "fixture should expose text");
    for (expected, segment) in layer.segments.iter().enumerate() {
        assert_eq!(
            segment.segment_index, expected as u32,
            "segment index should preserve extraction order"
        );
        assert!(
            !segment.text.is_empty(),
            "empty text segments should be filtered"
        );
        assert!(segment.rect.width > 0.0);
        assert!(segment.rect.height > 0.0);
    }
}

#[test]
fn text_layer_rejects_stale_generation() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = TextLayerRequest {
        document_id: session.id,
        generation: domain::document::DocumentGeneration(session.generation.0 + 999),
        page_index: PageIndex(0),
    };
    let result = block_on(engine.extract_page_text_layer(request)).unwrap();

    assert_eq!(result, Err(TextLayerError::DocumentNotOpen));
}
