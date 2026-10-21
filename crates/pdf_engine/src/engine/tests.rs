use super::*;

fn map_internal(error: PdfiumInternalError) -> DocumentError {
    map_pdfium_load_error(PdfiumError::PdfiumLibraryInternalError(error))
}

#[test]
fn password_error_maps_to_password_required() {
    assert_eq!(
        map_internal(PdfiumInternalError::PasswordError),
        DocumentError::PasswordRequired
    );
}

#[test]
fn format_error_maps_to_parse_failed() {
    assert_eq!(
        map_internal(PdfiumInternalError::FormatError),
        DocumentError::PdfParseFailed
    );
}

#[test]
fn security_error_maps_to_parse_failed() {
    assert_eq!(
        map_internal(PdfiumInternalError::SecurityError),
        DocumentError::PdfParseFailed
    );
}

#[test]
fn page_error_maps_to_parse_failed() {
    assert_eq!(
        map_internal(PdfiumInternalError::PageError),
        DocumentError::PdfParseFailed
    );
}

#[test]
fn file_error_maps_to_file_not_readable() {
    assert_eq!(
        map_internal(PdfiumInternalError::FileError),
        DocumentError::FileNotReadable
    );
}

#[test]
fn unknown_internal_error_stays_unknown() {
    assert!(matches!(
        map_internal(PdfiumInternalError::Unknown),
        DocumentError::Unknown(_)
    ));
}
