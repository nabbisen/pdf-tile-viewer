//! Search model (RFC 010) — non-mutating; PDF bytes are never modified.

use std::time::SystemTime;

use crate::document::{DocumentGeneration, DocumentId, PageIndex};

/// Minimum visible characters before a search runs (RFC 010 §9).
pub const MIN_QUERY_CHARS: usize = 2;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
}

impl SearchQuery {
    pub fn plain(text: impl Into<String>) -> Self {
        SearchQuery {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Whether this query is allowed to run (RFC 010 §9).
    pub fn is_runnable(&self) -> bool {
        self.text.trim().chars().count() >= MIN_QUERY_CHARS
    }
}

#[derive(Clone, Debug)]
pub struct SearchRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub query: SearchQuery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageSearchSummary {
    pub page_index: PageIndex,
    pub match_count: usize,
}

#[derive(Clone, Debug)]
pub struct SearchResultSet {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub query: SearchQuery,
    pub pages: Vec<PageSearchSummary>,
    pub total_matches: usize,
    pub searched_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchError {
    DocumentNotOpen,
    QueryTooShort,
    EngineUnavailable,
    SearchFailed(String),
}

/// Format matched zero-based page indices as a compact, one-based display
/// string (RFC 010 §10): `[0, 3, 4, 5, 9]` → `"1, 4–6, 10"`.
pub fn format_matched_pages(pages: &[PageIndex]) -> String {
    let mut sorted: Vec<usize> = pages.iter().map(|p| p.display_number()).collect();
    sorted.sort_unstable();
    sorted.dedup();

    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let start = sorted[i];
        let mut end = start;
        while i + 1 < sorted.len() && sorted[i + 1] == end + 1 {
            i += 1;
            end = sorted[i];
        }
        if end > start + 1 {
            parts.push(format!("{start}\u{2013}{end}"));
        } else if end == start + 1 {
            parts.push(start.to_string());
            parts.push(end.to_string());
        } else {
            parts.push(start.to_string());
        }
        i += 1;
    }
    parts.join(", ")
}
