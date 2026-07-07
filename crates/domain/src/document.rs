//! Document session model (RFC 004).
//!
//! A `DocumentSession` is the app-level representation of an opened PDF.
//! Native PDFium handles never appear here; the PDF engine keeps them private
//! and exposes only these types across the service boundary.

use std::path::PathBuf;
use std::time::SystemTime;

/// Opaque, process-unique document identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DocumentId(pub u64);

/// Zero-based page index. User-facing page numbers are one-based.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PageIndex(pub usize);

impl PageIndex {
    /// One-based page number for display.
    pub fn display_number(self) -> usize {
        self.0 + 1
    }

    /// Parse a one-based user input into a zero-based index, bounded by `page_count`.
    pub fn from_display_number(n: usize, page_count: usize) -> Option<Self> {
        if n >= 1 && n <= page_count {
            Some(PageIndex(n - 1))
        } else {
            None
        }
    }
}

/// Monotonic counter; any operation invalidating render/search results bumps it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Default)]
pub struct DocumentGeneration(pub u64);

impl DocumentGeneration {
    pub fn next(self) -> Self {
        DocumentGeneration(self.0 + 1)
    }
}

/// A value tagged with the document and generation it was computed against.
/// Consumers must discard values whose tag no longer matches current state.
#[derive(Clone, Debug, PartialEq)]
pub struct Generated<T: PartialEq> {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub value: T,
}

impl<T: PartialEq> Generated<T> {
    pub fn matches(&self, id: DocumentId, generation: DocumentGeneration) -> bool {
        self.document_id == id && self.generation == generation
    }
}

/// Where a document came from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentSource {
    LocalFile {
        path: PathBuf,
        fingerprint: FileFingerprint,
    },
}

/// Cheap identity check for "did the file change on disk".
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct FileFingerprint {
    pub size_bytes: u64,
    pub modified_at: Option<SystemTime>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub page_count: usize,
    pub encrypted: bool,
}

/// Transient user-supplied PDF password.
///
/// This type intentionally does not implement `Display`, `Debug`, or `Clone`.
/// It reduces accidental exposure in logs and diagnostics, but it is not a
/// hard memory-erasure guarantee.
#[derive(Eq, PartialEq)]
pub struct DocumentPassword(String);

impl DocumentPassword {
    pub fn new(value: String) -> Self {
        DocumentPassword(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Canonical page geometry. Width/height are PDF points (1/72 inch).
/// Layout and coordinate-transform code must use these descriptors and must
/// not query the PDF engine for page size (RFC 004 §11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageDescriptor {
    pub page_index: PageIndex,
    pub width_points: f32,
    pub height_points: f32,
    pub rotation_degrees: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DocumentState {
    Opening,
    Ready,
    Failed(DocumentError),
    Closing,
    Closed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DocumentError {
    FileNotFound,
    FileNotReadable,
    UnsupportedFile,
    PdfiumUnavailable,
    PdfParseFailed,
    PasswordRequired,
    EncryptedUnsupported,
    TooLargeForPolicy,
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentSession {
    pub id: DocumentId,
    pub source: DocumentSource,
    pub display_name: String,
    pub metadata: DocumentMetadata,
    pub pages: Vec<PageDescriptor>,
    pub opened_at: SystemTime,
    pub generation: DocumentGeneration,
    pub state: DocumentState,
}

/// PDF magic header required by file intake validation (RFC 002 §7).
pub const PDF_MAGIC: &[u8; 5] = b"%PDF-";

/// Returns true when the given header prefix marks a PDF file.
pub fn header_is_pdf(prefix: &[u8]) -> bool {
    prefix.len() >= PDF_MAGIC.len() && &prefix[..PDF_MAGIC.len()] == PDF_MAGIC
}

#[cfg(test)]
mod tests;
