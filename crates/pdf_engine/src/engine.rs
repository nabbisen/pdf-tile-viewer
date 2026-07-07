//! Engine core (RFC 004): owns the Pdfium instance and the registry of
//! native documents keyed by `DocumentId`. Only `domain` types cross out.

use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use domain::document::{
    DocumentError, DocumentGeneration, DocumentId, DocumentMetadata, DocumentSession,
    DocumentSource, DocumentState, FileFingerprint, PageDescriptor, PageIndex, header_is_pdf,
};
use pdfium_render::prelude::*;

pub struct PdfEngine {
    // Field order is load-bearing: Rust drops fields in declaration order,
    // and every `PdfDocument` in `sessions` borrows from `pdfium`'s boxed
    // bindings. `sessions` MUST be declared (and therefore dropped) before
    // `pdfium`. Do not reorder.
    sessions: HashMap<DocumentId, PdfiumBackedSession>,
    pdfium: Pdfium,
    next_id: u64,
    generation_counter: u64,
}

struct PdfiumBackedSession {
    public: DocumentSession,
    // Native handle; lifetime '_ ties documents to the engine-owned Pdfium.
    native: PdfDocument<'static>,
}

impl PdfEngine {
    /// The engine takes ownership of the bound Pdfium instance.
    ///
    /// SAFETY of the `'static` document lifetime used in `open_document`:
    /// documents borrow `pdfium`'s heap-allocated bindings, whose address is
    /// stable across moves of `PdfEngine`. The struct's field declaration
    /// order guarantees `sessions` drops before `pdfium`, so no document
    /// outlives the bindings it borrows.
    pub fn new(pdfium: Pdfium) -> Self {
        PdfEngine {
            sessions: HashMap::new(),
            pdfium,
            next_id: 1,
            generation_counter: 1,
        }
    }

    pub fn open_document(&mut self, path: &Path) -> Result<DocumentSession, DocumentError> {
        let bytes_prefix = std::fs::File::open(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => DocumentError::FileNotFound,
                _ => DocumentError::FileNotReadable,
            })
            .and_then(|mut f| {
                use std::io::Read;
                let mut buf = [0u8; 5];
                f.read_exact(&mut buf)
                    .map(|_| buf)
                    .map_err(|_| DocumentError::UnsupportedFile)
            })?;
        if !header_is_pdf(&bytes_prefix) {
            return Err(DocumentError::UnsupportedFile);
        }

        let meta = std::fs::metadata(path).map_err(|_| DocumentError::FileNotReadable)?;
        let fingerprint = FileFingerprint {
            size_bytes: meta.len(),
            modified_at: meta.modified().ok(),
        };

        let native = self
            .pdfium
            .load_pdf_from_file(path, None)
            .map_err(map_pdfium_load_error)?;
        // SAFETY: see `new` — the document never outlives `self.pdfium`.
        let native: PdfDocument<'static> = unsafe { std::mem::transmute(native) };

        let pages = collect_page_descriptors(&native)?;
        let metadata = collect_metadata(&native, pages.len());

        let id = DocumentId(self.next_id);
        self.next_id += 1;
        self.generation_counter += 1;

        let display_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "document.pdf".to_string());

        let session = DocumentSession {
            id,
            source: DocumentSource::LocalFile {
                path: path.to_path_buf(),
                fingerprint,
            },
            display_name,
            metadata,
            pages,
            opened_at: SystemTime::now(),
            generation: DocumentGeneration(self.generation_counter),
            state: DocumentState::Ready,
        };

        self.sessions.insert(
            id,
            PdfiumBackedSession {
                public: session.clone(),
                native,
            },
        );
        Ok(session)
    }

    /// Close a document and release native PDFium resources (RFC 004 §12).
    pub fn close_document(&mut self, id: DocumentId) -> bool {
        self.sessions.remove(&id).is_some()
    }

    pub fn session(&self, id: DocumentId) -> Option<&DocumentSession> {
        self.sessions.get(&id).map(|s| &s.public)
    }

    pub(crate) fn native(&self, id: DocumentId) -> Option<&PdfDocument<'static>> {
        self.sessions.get(&id).map(|s| &s.native)
    }
}

fn map_pdfium_load_error(error: PdfiumError) -> DocumentError {
    match error {
        PdfiumError::PdfiumLibraryInternalError(internal) => match internal {
            PdfiumInternalError::FileError => DocumentError::FileNotReadable,
            PdfiumInternalError::FormatError
            | PdfiumInternalError::SecurityError
            | PdfiumInternalError::PageError => DocumentError::PdfParseFailed,
            PdfiumInternalError::PasswordError => DocumentError::EncryptedUnsupported,
            PdfiumInternalError::Unknown => {
                DocumentError::Unknown("Unknown PDFium internal load error".to_string())
            }
        },
        other => DocumentError::Unknown(other.to_string()),
    }
}

fn collect_page_descriptors(
    document: &PdfDocument<'_>,
) -> Result<Vec<PageDescriptor>, DocumentError> {
    let pages = document.pages();
    let mut out = Vec::with_capacity(pages.len() as usize);
    for (index, page) in pages.iter().enumerate() {
        let rotation = page.rotation().map(|r| r.as_degrees() as i32).unwrap_or(0);
        out.push(PageDescriptor {
            page_index: PageIndex(index),
            width_points: page.width().value,
            height_points: page.height().value,
            rotation_degrees: rotation,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_internal(error: PdfiumInternalError) -> DocumentError {
        map_pdfium_load_error(PdfiumError::PdfiumLibraryInternalError(error))
    }

    #[test]
    fn password_error_maps_to_encrypted_unsupported() {
        assert_eq!(
            map_internal(PdfiumInternalError::PasswordError),
            DocumentError::EncryptedUnsupported
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
}

fn collect_metadata(document: &PdfDocument<'_>, page_count: usize) -> DocumentMetadata {
    let meta = document.metadata();
    let get = |tag: PdfDocumentMetadataTagType| {
        meta.get(tag)
            .map(|item| item.value().to_string())
            .filter(|s| !s.is_empty())
    };
    DocumentMetadata {
        title: get(PdfDocumentMetadataTagType::Title),
        author: get(PdfDocumentMetadataTagType::Author),
        subject: get(PdfDocumentMetadataTagType::Subject),
        creator: get(PdfDocumentMetadataTagType::Creator),
        producer: get(PdfDocumentMetadataTagType::Producer),
        page_count,
        encrypted: false,
    }
}
