//! Engine smoke tests: encryption.

use super::*;
use domain::document::{DocumentError, DocumentPassword, PageIndex};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};

#[test]
fn encrypted_pdf_reports_password_required_and_unlocks_with_correct_password() {
    let engine = require_engine!();
    let path = fixture("password-protected.pdf");
    let bytes_before = std::fs::read(&path).unwrap();
    let session_count_before = block_on(engine.debug_session_count()).expect("engine alive");

    let no_password = block_on(engine.open_document(path.clone())).expect("engine alive");
    assert_eq!(no_password, Err(DocumentError::PasswordRequired));
    assert_eq!(
        block_on(engine.debug_session_count()).expect("engine alive"),
        session_count_before,
        "password-required attempt must not create a session"
    );

    let wrong_password = block_on(engine.open_document_with_password(
        path.clone(),
        DocumentPassword::new("not-the-password".to_string()),
    ))
    .expect("engine alive");
    assert_eq!(wrong_password, Err(DocumentError::PasswordRequired));
    assert_eq!(
        block_on(engine.debug_session_count()).expect("engine alive"),
        session_count_before,
        "wrong-password attempt must not create a session"
    );

    let empty_password = block_on(
        engine.open_document_with_password(path.clone(), DocumentPassword::new(String::new())),
    )
    .expect("engine alive");
    assert_eq!(empty_password, Err(DocumentError::PasswordRequired));
    assert_eq!(
        block_on(engine.debug_session_count()).expect("engine alive"),
        session_count_before,
        "empty-password attempt must not create a session for this fixture"
    );

    let session = block_on(engine.open_document_with_password(
        path.clone(),
        DocumentPassword::new("pdf-tile-viewer-test".to_string()),
    ))
    .expect("engine alive")
    .expect("correct password opens");
    assert!(session.metadata.encrypted);
    assert_eq!(session.pages.len(), 1);

    let request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    let image = block_on(engine.render_page(request)).unwrap().unwrap();
    assert!(image.pixel_width > 0 && image.pixel_height > 0);
    assert!(block_on(engine.close_document(session.id)).unwrap());

    let bytes_after = std::fs::read(&path).unwrap();
    assert_eq!(bytes_before, bytes_after, "PDF file must not be modified");
}
