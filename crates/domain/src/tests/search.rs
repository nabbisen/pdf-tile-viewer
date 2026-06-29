use crate::document::PageIndex;
use crate::search::*;

#[test]
fn compact_page_formatting_matches_rfc_010_example() {
    // RFC 010 §10: [0, 3, 4, 5, 9] → "1, 4–6, 10"
    let pages: Vec<PageIndex> = [0, 3, 4, 5, 9].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 4\u{2013}6, 10");
}

#[test]
fn two_adjacent_pages_are_listed_not_ranged() {
    let pages: Vec<PageIndex> = [0, 1].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 2");
}

#[test]
fn unsorted_and_duplicate_input_is_normalized() {
    let pages: Vec<PageIndex> = [4, 0, 4, 2, 3].into_iter().map(PageIndex).collect();
    assert_eq!(format_matched_pages(&pages), "1, 3\u{2013}5");
}

#[test]
fn empty_input_formats_to_empty_string() {
    assert_eq!(format_matched_pages(&[]), "");
}

#[test]
fn minimum_query_policy_requires_two_visible_chars() {
    // RFC 010 §9.
    assert!(!SearchQuery::plain("").is_runnable());
    assert!(!SearchQuery::plain(" a ").is_runnable());
    assert!(SearchQuery::plain("ab").is_runnable());
    assert!(SearchQuery::plain("  ab  ").is_runnable());
}
