//! PDFium binding (RFC 003).
//!
//! Resolution policy (which directories may be searched) lives in
//! `packaging::pdfium_bundle`; this module performs the actual dynamic bind
//! and produces the load report used by diagnostics.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum PdfiumLoadError {
    LibraryNotFound { searched: Vec<PathBuf> },
    BindFailed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfiumLoadReport {
    pub resolved_dir: PathBuf,
    pub version: Option<String>,
}

/// Bind PDFium from a specific, already-resolved library directory.
///
/// A failed bind returns an error instead of panicking (RFC 003 §9).
pub fn bind_from_dir(library_dir: &Path) -> Result<(Pdfium, PdfiumLoadReport), PdfiumLoadError> {
    let library_path = Pdfium::pdfium_platform_library_name_at_path(library_dir);
    let bindings = Pdfium::bind_to_library(&library_path)
        .map_err(|e| PdfiumLoadError::BindFailed(e.to_string()))?;
    let pdfium = Pdfium::new(bindings);
    let report = PdfiumLoadReport {
        resolved_dir: library_dir.to_path_buf(),
        version: None,
    };
    Ok((pdfium, report))
}
