//! File intake validation and document opening (RFC 002 §7).
//!
//! Validation is layered: cheap checks (existence, extension) run here on
//! the caller's thread; the `%PDF-` magic check and the actual parse run
//! inside the engine worker, which re-validates the header itself.

use std::path::Path;

use domain::document::{DocumentError, DocumentSession, PDF_MAGIC, header_is_pdf};
use pdf_engine::worker::EngineHandle;

/// Why a candidate file was rejected before reaching the engine.
#[derive(Clone, Debug, PartialEq)]
pub enum IntakeRejection {
    NotFound,
    NotAFile,
    /// Extension is not `.pdf` (case-insensitive). RFC 002 §7 step 2.
    WrongExtension,
    /// Readable, but the first bytes are not `%PDF-`. RFC 002 §7 step 3.
    NotAPdf,
    Unreadable,
}

/// Validate a dropped/picked path before handing it to the engine.
///
/// Order mirrors RFC 002 §7: existence → regular file → extension →
/// magic header. Each step fails fast with a user-presentable reason.
pub fn validate_candidate(path: &Path) -> Result<(), IntakeRejection> {
    if !path.exists() {
        return Err(IntakeRejection::NotFound);
    }
    if !path.is_file() {
        return Err(IntakeRejection::NotAFile);
    }
    let extension_ok = path
        .extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("pdf"))
        .unwrap_or(false);
    if !extension_ok {
        return Err(IntakeRejection::WrongExtension);
    }

    let mut prefix = [0u8; PDF_MAGIC.len()];
    let read = std::fs::File::open(path).and_then(|mut f| {
        use std::io::Read;
        f.read_exact(&mut prefix)
    });
    match read {
        Ok(()) if header_is_pdf(&prefix) => Ok(()),
        Ok(()) => Err(IntakeRejection::NotAPdf),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(IntakeRejection::NotAPdf),
        Err(_) => Err(IntakeRejection::Unreadable),
    }
}

/// Combined error for the full open path (validation + engine).
#[derive(Clone, Debug, PartialEq)]
pub enum OpenError {
    Rejected(IntakeRejection),
    Engine(DocumentError),
    EngineUnavailable,
}

/// Validate, then open the document on the engine worker.
pub async fn open_document(
    engine: &EngineHandle,
    path: &Path,
) -> Result<DocumentSession, OpenError> {
    validate_candidate(path).map_err(OpenError::Rejected)?;
    match engine.open_document(path.to_path_buf()).await {
        Ok(Ok(session)) => Ok(session),
        Ok(Err(e)) => Err(OpenError::Engine(e)),
        Err(_) => Err(OpenError::EngineUnavailable),
    }
}

#[cfg(test)]
mod tests;
