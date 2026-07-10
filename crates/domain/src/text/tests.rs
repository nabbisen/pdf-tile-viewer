use crate::document::{DocumentGeneration, DocumentId, PageIndex};
use crate::search::{PageCoordinateSpace, PageRect};
use crate::text::{PageTextLayer, TextLayerCacheKey, TextLayerRequest, TextLayerSegment};

#[test]
fn text_layer_request_carries_document_generation_and_page() {
    let request = TextLayerRequest {
        document_id: DocumentId(7),
        generation: DocumentGeneration(11),
        page_index: PageIndex(2),
    };

    assert_eq!(request.document_id, DocumentId(7));
    assert_eq!(request.generation, DocumentGeneration(11));
    assert_eq!(request.page_index, PageIndex(2));
}

#[test]
fn text_layer_request_produces_cache_key() {
    let request = TextLayerRequest {
        document_id: DocumentId(7),
        generation: DocumentGeneration(11),
        page_index: PageIndex(2),
    };

    assert_eq!(
        request.cache_key(),
        TextLayerCacheKey {
            document_id: DocumentId(7),
            generation: DocumentGeneration(11),
            page_index: PageIndex(2),
        }
    );
}

#[test]
fn page_text_layer_segments_preserve_extraction_order() {
    let layer = PageTextLayer {
        document_id: DocumentId(1),
        generation: DocumentGeneration(3),
        page_index: PageIndex(0),
        segments: vec![TextLayerSegment {
            segment_index: 0,
            text: "Hello".to_string(),
            rect: PageRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 12.0,
                space: PageCoordinateSpace::PdfPointsBottomLeft,
            },
        }],
    };

    assert_eq!(layer.segments[0].segment_index, 0);
    assert_eq!(layer.segments[0].text, "Hello");
}
