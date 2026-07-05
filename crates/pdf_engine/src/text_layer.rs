//! Page text-layer extraction for single-page selection (RFC 023).
//!
//! PDFium handles stay inside this crate. The returned text layer is
//! domain-owned and generation-tagged so stale UI work can be discarded.

use domain::text::{PageTextLayer, TextLayerError, TextLayerRequest, TextLayerSegment};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;

pub fn extract_page_text_layer(
    engine: &PdfEngine,
    request: &TextLayerRequest,
) -> Result<PageTextLayer, TextLayerError> {
    let session = engine
        .session(request.document_id)
        .ok_or(TextLayerError::DocumentNotOpen)?;
    if session.generation != request.generation {
        return Err(TextLayerError::DocumentNotOpen);
    }
    if request.page_index.0 >= session.pages.len() {
        return Err(TextLayerError::PageOutOfBounds);
    }

    let document = engine
        .native(request.document_id)
        .ok_or(TextLayerError::DocumentNotOpen)?;
    let page = document
        .pages()
        .get(request.page_index.0 as PdfPageIndex)
        .map_err(|_| TextLayerError::PageOutOfBounds)?;
    let text = page
        .text()
        .map_err(|e| TextLayerError::ExtractionFailed(e.to_string()))?;

    let mut segments = Vec::new();
    for (idx, segment) in text.segments().iter().enumerate() {
        let segment_index = u32::try_from(idx).map_err(|_| TextLayerError::SegmentIndexOverflow)?;
        let raw_text = segment.text();
        if raw_text.is_empty() {
            continue;
        }

        let bounds = segment.bounds();
        if bounds.width().value <= 0.0 || bounds.height().value <= 0.0 {
            continue;
        }
        segments.push(TextLayerSegment {
            segment_index,
            text: raw_text,
            rect: domain::search::PageRect {
                x: bounds.left().value,
                y: bounds.bottom().value,
                width: bounds.width().value,
                height: bounds.height().value,
                space: domain::search::PageCoordinateSpace::PdfPointsBottomLeft,
            },
        });
    }

    Ok(PageTextLayer {
        document_id: request.document_id,
        generation: request.generation,
        page_index: request.page_index,
        segments,
    })
}
