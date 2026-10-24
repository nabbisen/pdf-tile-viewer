//! Engine core (RFC 004): owns the Pdfium instance and the registry of
//! native documents keyed by `DocumentId`. Only `domain` types cross out.

use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use domain::document::{
    DocumentError, DocumentGeneration, DocumentId, DocumentMetadata, DocumentPassword,
    DocumentSession, DocumentSource, DocumentState, FileFingerprint, PageDescriptor, PageIndex,
    header_is_pdf,
};
use pdfium_render::prelude::*;

use crate::user_space::{PageBox, PageGeometry, Rotation};

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
    // Per-page conversion geometry (RFC 034), computed once at open; index = page index.
    geometry: Vec<PageGeometry>,
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
        self.open_document_with_password(path, None)
    }

    pub fn open_document_with_password(
        &mut self,
        path: &Path,
        password: Option<&DocumentPassword>,
    ) -> Result<DocumentSession, DocumentError> {
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
            .load_pdf_from_file(path, password.map(DocumentPassword::as_str))
            .map_err(map_pdfium_load_error)?;
        // SAFETY: see `new` — the document never outlives `self.pdfium`.
        let native: PdfDocument<'static> = unsafe { std::mem::transmute(native) };

        let (pages, geometry) = collect_page_descriptors(&native)?;
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
                geometry,
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

    /// Conversion geometry of one page (RFC 034); `None` for an unknown document or page.
    pub(crate) fn geometry(&self, id: DocumentId, page_index: usize) -> Option<PageGeometry> {
        self.sessions.get(&id)?.geometry.get(page_index).copied()
    }

    /// Conversion geometry of every page of a document (empty for an unknown document).
    pub(crate) fn geometries(&self, id: DocumentId) -> &[PageGeometry] {
        self.sessions
            .get(&id)
            .map_or(&[], |s| s.geometry.as_slice())
    }

    /// Pages whose computed display size disagrees with PDFium's (RFC 034 §5.3): they are
    /// rendered normally but produce no overlay geometry.
    pub fn inconsistent_pages(&self, id: DocumentId) -> Vec<PageIndex> {
        self.sessions.get(&id).map_or_else(Vec::new, |s| {
            s.geometry
                .iter()
                .enumerate()
                .filter(|(_, g)| !g.is_consistent())
                .map(|(i, _)| PageIndex(i))
                .collect()
        })
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

fn map_pdfium_load_error(error: PdfiumError) -> DocumentError {
    match error {
        PdfiumError::PdfiumLibraryInternalError(internal) => match internal {
            PdfiumInternalError::FileError => DocumentError::FileNotReadable,
            PdfiumInternalError::FormatError
            | PdfiumInternalError::SecurityError
            | PdfiumInternalError::PageError => DocumentError::PdfParseFailed,
            PdfiumInternalError::PasswordError => DocumentError::PasswordRequired,
            PdfiumInternalError::Unknown => {
                DocumentError::Unknown("Unknown PDFium internal load error".to_string())
            }
        },
        other => DocumentError::Unknown(other.to_string()),
    }
}

fn collect_page_descriptors(
    document: &PdfDocument<'_>,
) -> Result<(Vec<PageDescriptor>, Vec<PageGeometry>), DocumentError> {
    let pages = document.pages();
    let mut descriptors = Vec::with_capacity(pages.len() as usize);
    let mut geometry = Vec::with_capacity(pages.len() as usize);
    for (index, page) in pages.iter().enumerate() {
        let width = page.width().value;
        let height = page.height().value;
        descriptors.push(PageDescriptor {
            page_index: PageIndex(index),
            display_width_points: width,
            display_height_points: height,
        });
        geometry.push(page_geometry(&page, (width, height)));
    }
    Ok((descriptors, geometry))
}

/// Reads the page box and rotation once and cross-checks them against the size PDFium reports.
fn page_geometry(page: &PdfPage<'_>, reported: (f32, f32)) -> PageGeometry {
    let page_box = page.boundaries().bounding().ok().map(|b| PageBox {
        x0: b.bounds.left().value,
        y0: b.bounds.bottom().value,
        x1: b.bounds.right().value,
        y1: b.bounds.top().value,
    });
    let rotation = page
        .rotation()
        .ok()
        .map(|r| Rotation::from_degrees(r.as_degrees() as i32));
    match (page_box, rotation) {
        (Some(page_box), Some(rotation)) => PageGeometry::new(page_box, rotation, reported),
        _ => PageGeometry::unavailable(),
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
        encrypted: is_encrypted(document),
    }
}

fn is_encrypted(document: &PdfDocument<'_>) -> bool {
    !matches!(
        document.permissions().security_handler_revision(),
        Ok(PdfSecurityHandlerRevision::Unprotected)
    )
}

#[cfg(test)]
mod tests;
