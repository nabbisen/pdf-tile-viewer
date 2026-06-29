//! Text search (RFC 010): non-mutating, page-level match summaries.
//!
//! PDF bytes are never modified; results are an app-side model.

use std::time::SystemTime;

use domain::document::PageIndex;
use domain::search::{PageSearchSummary, SearchError, SearchRequest, SearchResultSet};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;

pub fn search_document(
    engine: &PdfEngine,
    request: &SearchRequest,
) -> Result<SearchResultSet, SearchError> {
    if !request.query.is_runnable() {
        return Err(SearchError::QueryTooShort);
    }
    let session = engine
        .session(request.document_id)
        .ok_or(SearchError::DocumentNotOpen)?;
    if session.generation != request.generation {
        return Err(SearchError::DocumentNotOpen);
    }
    let document = engine
        .native(request.document_id)
        .ok_or(SearchError::DocumentNotOpen)?;

    let mut options = PdfSearchOptions::new();
    if request.query.case_sensitive {
        options = options.match_case(true);
    }
    if request.query.whole_word {
        options = options.match_whole_word(true);
    }

    let mut pages_out: Vec<PageSearchSummary> = Vec::new();
    let mut total = 0usize;

    for (index, page) in document.pages().iter().enumerate() {
        let text = match page.text() {
            Ok(text) => text,
            Err(e) => return Err(SearchError::SearchFailed(e.to_string())),
        };
        let search = match text.search(&request.query.text, &options) {
            Ok(search) => search,
            Err(e) => return Err(SearchError::SearchFailed(e.to_string())),
        };
        let count = search.iter(PdfSearchDirection::SearchForward).count();
        if count > 0 {
            pages_out.push(PageSearchSummary {
                page_index: PageIndex(index),
                match_count: count,
            });
            total += count;
        }
    }

    Ok(SearchResultSet {
        document_id: request.document_id,
        generation: request.generation,
        query: request.query.clone(),
        pages: pages_out,
        total_matches: total,
        searched_at: SystemTime::now(),
    })
}
