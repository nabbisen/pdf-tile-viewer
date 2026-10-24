//! Page text-layer extraction for single-page selection (RFC 023).
//!
//! PDFium handles stay inside this crate. The returned text layer is
//! domain-owned and generation-tagged so stale UI work can be discarded.

use domain::text::{PageTextLayer, TextLayerError, TextLayerRequest, TextLayerSegment};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;
use crate::user_space::UserRect;

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

    let geometry = engine.geometry(request.document_id, request.page_index.0);
    let mut segments = Vec::new();
    for (idx, segment) in text.segments().iter().enumerate() {
        let segment_index = u32::try_from(idx).map_err(|_| TextLayerError::SegmentIndexOverflow)?;
        let raw_text = segment.text();
        if raw_text.is_empty() {
            continue;
        }

        // Degenerate rects and pages with inconsistent geometry yield no segment.
        let bounds = segment.bounds();
        let Some(rect) = geometry.and_then(|g| {
            g.rect_to_display(UserRect {
                left: bounds.left().value,
                bottom: bounds.bottom().value,
                right: bounds.right().value,
                top: bounds.top().value,
            })
        }) else {
            continue;
        };
        segments.push(TextLayerSegment {
            segment_index,
            text: raw_text,
            rect,
        });
    }

    Ok(PageTextLayer {
        document_id: request.document_id,
        generation: request.generation,
        page_index: request.page_index,
        segments,
    })
}
