//! Engine smoke tests: document.

use super::*;
use domain::document::PageIndex;

#[test]
fn open_reports_correct_geometry_and_page_count() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .expect("engine alive")
        .expect("open ok");

    assert_eq!(session.pages.len(), 1);
    assert_eq!(session.metadata.page_count, 1);
    assert!(!session.metadata.encrypted);
    let page = session.pages[0];
    assert_eq!(page.page_index, PageIndex(0));
    // US Letter: 612 x 792 points.
    assert!(
        (page.width_points - 612.0).abs() < 0.5,
        "width {}",
        page.width_points
    );
    assert!(
        (page.height_points - 792.0).abs() < 0.5,
        "height {}",
        page.height_points
    );
}

#[test]
fn large_document_opens_and_reports_correct_page_count() {
    // RFC 015 M9 extension: opening a 50-page document must not panic, must
    // return the correct page count, and must close cleanly.
    let engine = require_engine!();
    let path = fixture("fifty-pages-benchmark.pdf");
    let session = block_on(engine.open_document(path)).unwrap().unwrap();

    assert_eq!(session.pages.len(), 50, "page count");
    assert_eq!(session.metadata.page_count, 50);
    // All pages should be US Letter with 0° rotation.
    for (i, page) in session.pages.iter().enumerate() {
        assert!((page.width_points - 612.0).abs() < 1.0, "page {i} width");
        assert!((page.height_points - 792.0).abs() < 1.0, "page {i} height");
        assert_eq!(page.rotation_degrees, 0, "page {i} rotation");
    }

    let closed = block_on(engine.close_document(session.id)).unwrap();
    assert!(closed, "close large document");
}
