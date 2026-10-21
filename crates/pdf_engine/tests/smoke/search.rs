//! Engine smoke tests: search.

use super::*;
use domain::search::{SearchQuery, SearchRequest};

#[test]
fn search_finds_expected_pages_without_mutating_file() {
    let engine = require_engine!();
    let path = fixture("multi-page-search.pdf");
    let bytes_before = std::fs::read(&path).unwrap();

    let session = block_on(engine.open_document(path.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(session.pages.len(), 3);

    let request = SearchRequest {
        document_id: session.id,
        generation: session.generation,
        query: SearchQuery::plain("tile"),
    };
    let results = block_on(engine.search_document(request)).unwrap().unwrap();

    // Fixture spec (RFC 015): "tile" appears on pages 1 and 3 (display).
    let pages: Vec<usize> = results
        .pages
        .iter()
        .map(|p| p.page_index.display_number())
        .collect();
    assert_eq!(pages, vec![1, 3], "matched pages");
    assert!(results.total_matches >= 2, "match count");

    // RFC 010 §11: search never mutates the PDF.
    let bytes_after = std::fs::read(&path).unwrap();
    assert_eq!(bytes_before, bytes_after, "PDF file must not be modified");
}

#[test]
fn large_document_search_runs_without_error() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("fifty-pages-benchmark.pdf")))
        .unwrap()
        .unwrap();

    let request = domain::search::SearchRequest {
        document_id: session.id,
        generation: session.generation,
        query: domain::search::SearchQuery::plain("benchmark"),
    };
    let results = block_on(engine.search_document(request)).unwrap().unwrap();

    // "benchmark" appears on every page of the fixture.
    assert_eq!(results.pages.len(), 50, "all 50 pages should match");
    assert!(results.total_matches >= 50);
}
