use std::io::Write;

use domain::document::DocumentError;

use crate::document_service::{
    IntakeRejection, OpenDocumentOutcome, outcome_from_engine_result, validate_candidate,
};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ptv-doc-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn missing_file_is_rejected_first() {
    let path = temp_dir().join("nope.pdf");
    assert_eq!(validate_candidate(&path), Err(IntakeRejection::NotFound));
}

#[test]
fn directory_is_not_a_file() {
    let dir = temp_dir();
    let sub = dir.join("folder.pdf");
    std::fs::create_dir_all(&sub).unwrap();
    assert_eq!(validate_candidate(&sub), Err(IntakeRejection::NotAFile));
}

#[test]
fn wrong_extension_rejected_before_content_is_read() {
    // RFC 002 §7: extension check precedes the magic check, so even a file
    // with valid PDF bytes is rejected when named *.txt.
    let path = temp_dir().join("real-pdf-bytes.txt");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"%PDF-1.7\n")
        .unwrap();
    assert_eq!(
        validate_candidate(&path),
        Err(IntakeRejection::WrongExtension)
    );
}

#[test]
fn extension_check_is_case_insensitive() {
    let path = temp_dir().join("UPPER.PDF");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"%PDF-1.4 rest")
        .unwrap();
    assert_eq!(validate_candidate(&path), Ok(()));
}

#[test]
fn pdf_extension_with_non_pdf_bytes_is_rejected() {
    let path = temp_dir().join("fake.pdf");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"GIF89a not a pdf")
        .unwrap();
    assert_eq!(validate_candidate(&path), Err(IntakeRejection::NotAPdf));
}

#[test]
fn too_short_file_is_not_a_pdf() {
    let path = temp_dir().join("tiny.pdf");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"%P")
        .unwrap();
    assert_eq!(validate_candidate(&path), Err(IntakeRejection::NotAPdf));
}

#[test]
fn password_required_becomes_recoverable_outcome() {
    let path = temp_dir().join("protected.pdf");
    let outcome = outcome_from_engine_result(&path, Err(DocumentError::PasswordRequired)).unwrap();

    assert_eq!(
        outcome,
        OpenDocumentOutcome::PasswordRequired(crate::document_service::PasswordRequiredContext {
            path,
        })
    );
}

#[test]
fn every_other_engine_error_becomes_a_displayable_failure() {
    use domain::document::OpenFailure;
    let path = temp_dir().join("other.pdf");
    for (error, expected) in [
        (DocumentError::FileNotFound, OpenFailure::FileNotFound),
        (DocumentError::FileNotReadable, OpenFailure::FileNotReadable),
        (DocumentError::UnsupportedFile, OpenFailure::UnsupportedFile),
        (
            DocumentError::PdfiumUnavailable,
            OpenFailure::PdfiumUnavailable,
        ),
        (DocumentError::PdfParseFailed, OpenFailure::PdfParseFailed),
        (
            DocumentError::TooLargeForPolicy,
            OpenFailure::TooLargeForPolicy,
        ),
        (
            DocumentError::Unknown("x".into()),
            OpenFailure::Unknown("x".into()),
        ),
    ] {
        let result = outcome_from_engine_result(&path, Err(error));
        assert_eq!(
            result,
            Err(crate::document_service::OpenError::Engine(expected))
        );
    }
}

#[test]
fn a_password_request_has_no_displayable_form() {
    use domain::document::{OpenFailure, PasswordIsRequired};
    assert_eq!(
        OpenFailure::try_from(DocumentError::PasswordRequired),
        Err(PasswordIsRequired)
    );
    // and it never comes out of the service as an error
    let path = temp_dir().join("locked.pdf");
    assert!(outcome_from_engine_result(&path, Err(DocumentError::PasswordRequired)).is_ok());
}
