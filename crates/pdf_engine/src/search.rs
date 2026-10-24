//! Text search (RFC 010): non-mutating, page-level match summaries.
//!
//! PDF bytes are never modified; results are an app-side model.

use std::time::SystemTime;

use domain::document::PageIndex;
use domain::search::{PageSearchSummary, SearchError, SearchRequest, SearchResultSet};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;
use crate::user_space::UserRect;

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

/// Search with per-match rectangle extraction (RFC 011).
///
/// Returns both page-level summaries (RFC 010) and highlight coordinates
/// (RFC 011) in one PDFium pass to avoid redundant page iteration.
pub fn search_document_with_highlights(
    engine: &PdfEngine,
    request: &SearchRequest,
) -> Result<(SearchResultSet, domain::search::SearchHighlightSet), SearchError> {
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

    let mut summary_pages: Vec<PageSearchSummary> = Vec::new();
    let mut highlight_pages: Vec<domain::search::PageHighlightSet> = Vec::new();
    let mut total = 0usize;

    for (index, page) in document.pages().iter().enumerate() {
        let text = match page.text() {
            Ok(t) => t,
            Err(e) => return Err(SearchError::SearchFailed(e.to_string())),
        };
        let search = match text.search(&request.query.text, &options) {
            Ok(s) => s,
            Err(e) => return Err(SearchError::SearchFailed(e.to_string())),
        };

        let geometry = engine.geometry(request.document_id, index);
        let mut highlights: Vec<domain::search::TextHighlight> = Vec::new();
        let mut match_count = 0usize;
        for (match_idx, segments) in search.iter(PdfSearchDirection::SearchForward).enumerate() {
            match_count += 1;
            // Rects that cannot be converted (inconsistent page, degenerate) are dropped;
            // the match itself still counts, only its overlay is lost.
            let rects: Vec<domain::geometry::PageRect> = segments
                .iter()
                .filter_map(|seg| {
                    let b = seg.bounds();
                    geometry?.rect_to_display(UserRect {
                        left: b.left().value,
                        bottom: b.bottom().value,
                        right: b.right().value,
                        top: b.top().value,
                    })
                })
                .collect();
            if !rects.is_empty() {
                highlights.push(domain::search::TextHighlight {
                    match_index: match_idx,
                    page_rects: rects,
                });
            }
        }

        if match_count > 0 {
            summary_pages.push(PageSearchSummary {
                page_index: PageIndex(index),
                match_count,
            });
            if !highlights.is_empty() {
                highlight_pages.push(domain::search::PageHighlightSet {
                    page_index: PageIndex(index),
                    highlights,
                });
            }
            total += match_count;
        }
    }

    let result_set = SearchResultSet {
        document_id: request.document_id,
        generation: request.generation,
        query: request.query.clone(),
        pages: summary_pages,
        total_matches: total,
        searched_at: std::time::SystemTime::now(),
    };

    let highlight_set = domain::search::SearchHighlightSet {
        document_id: request.document_id,
        generation: request.generation,
        query: request.query.clone(),
        pages: highlight_pages,
    };

    Ok((result_set, highlight_set))
}
