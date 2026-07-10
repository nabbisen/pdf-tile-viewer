//! Selectable page text layer model (RFC 023).
//!
//! This module exposes domain-owned text fragments for a single page. It is
//! intentionally separate from `search`: search answers "where did a query
//! match?", while text selection answers "what visible text can the user
//! select/copy on this page?"

use crate::document::{DocumentGeneration, DocumentId, PageIndex};
use crate::search::PageRect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayerRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
}

impl TextLayerRequest {
    pub fn cache_key(self) -> TextLayerCacheKey {
        TextLayerCacheKey {
            document_id: self.document_id,
            generation: self.generation,
            page_index: self.page_index,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TextLayerCacheKey {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PageTextLayer {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub segments: Vec<TextLayerSegment>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayerSegment {
    /// Stable extraction-order index for Dioxus keys and deterministic tests.
    pub segment_index: u32,
    pub text: String,
    pub rect: PageRect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextLayerError {
    DocumentNotOpen,
    EngineUnavailable,
    PageOutOfBounds,
    SegmentIndexOverflow,
    StaleResult,
    ExtractionFailed(String),
}

#[cfg(test)]
mod tests;
