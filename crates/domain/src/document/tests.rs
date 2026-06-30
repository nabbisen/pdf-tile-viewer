use crate::document::*;

#[test]
fn page_numbers_are_one_based_for_display_and_zero_based_internally() {
    // Appendix A §2 rules.
    assert_eq!(PageIndex(0).display_number(), 1);
    assert_eq!(PageIndex::from_display_number(1, 10), Some(PageIndex(0)));
    assert_eq!(PageIndex::from_display_number(10, 10), Some(PageIndex(9)));
    assert_eq!(PageIndex::from_display_number(0, 10), None);
    assert_eq!(PageIndex::from_display_number(11, 10), None);
}

#[test]
fn pdf_header_validation_follows_rfc_002_rules() {
    assert!(header_is_pdf(b"%PDF-1.7\n"));
    assert!(!header_is_pdf(b"%PDF"));
    assert!(!header_is_pdf(b"PK\x03\x04"));
    assert!(!header_is_pdf(b""));
}

#[test]
fn stale_generation_results_are_rejectable() {
    // RFC 004 §7: results carry document id + generation; mismatch is stale.
    let g = Generated {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        value: 42,
    };
    assert!(g.matches(DocumentId(1), DocumentGeneration(2)));
    assert!(!g.matches(DocumentId(1), DocumentGeneration(3)));
    assert!(!g.matches(DocumentId(2), DocumentGeneration(2)));
}
