//! Engine smoke tests (RFC 015 §5): open → geometry → render → search,
//! end-to-end through the serialized worker, against generated fixtures.
//!
//! Requires a real PDFium dynamic library. Set `PDF_TILE_VIEWER_PDFIUM_DIR`
//! to its directory (e.g. `ci/.pdfium`, populated by `ci/fetch-pdfium.sh`);
//! without it the tests skip with a notice instead of failing, so plain
//! `cargo test` stays green on machines without PDFium (RFC 015 §7).
//!
//! Implementation note: PDFium calls `FPDF_InitLibraryWithConfig` once per
//! process. All tests share a single `EngineHandle` via a `LazyLock`, and
//! the associated `EngineThread` is forgotten so `FPDF_DestroyLibrary` is
//! never called mid-process.

use std::path::PathBuf;
use std::sync::LazyLock;

use domain::document::PageIndex;
use domain::render::{
    RenderFlags, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, ScaleBucket,
};
use domain::search::{SearchQuery, SearchRequest};
use pdf_engine::worker::EngineHandle;

const PDFIUM_DIR_ENV: &str = "PDF_TILE_VIEWER_PDFIUM_DIR";

fn pdfium_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(PDFIUM_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    let repo_default = workspace_root().join("ci").join(".pdfium");
    repo_default.is_dir().then_some(repo_default)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/pdf_engine")
        .to_path_buf()
}

fn fixture(name: &str) -> PathBuf {
    workspace_root().join("fixtures").join(name)
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    futures_executor::block_on(future)
}

/// Shared PDFium engine for the whole test process.
///
/// PDFium may only be initialized once per OS process; the EngineThread is
/// forgotten so FPDF_DestroyLibrary is never called between tests.
static ENGINE: LazyLock<Option<EngineHandle>> = LazyLock::new(|| {
    let dir = pdfium_dir()?;
    let (handle, thread) = EngineHandle::spawn(dir).expect("PDFium bind failed");
    std::mem::forget(thread);
    Some(handle)
});

macro_rules! require_engine {
    () => {{
        match ENGINE.as_ref() {
            Some(h) => h,
            None => {
                eprintln!(
                    "SKIP: set {PDFIUM_DIR_ENV} (or run ci/fetch-pdfium.sh) \
                     to run engine smoke tests"
                );
                return;
            }
        }
    }};
}

#[test]
fn open_reports_correct_geometry_and_page_count() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .expect("engine alive")
        .expect("open ok");

    assert_eq!(session.pages.len(), 1);
    assert_eq!(session.metadata.page_count, 1);
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
fn render_page_one_produces_valid_png() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    let image = block_on(engine.render_page(request)).unwrap().unwrap();

    assert_eq!(image.page_index, PageIndex(0));
    assert!(image.pixel_width > 0 && image.pixel_height > 0);
    // 612pt @ scale 1.0 → target_w 612px (clamp not hit).
    assert_eq!(image.pixel_width, 612, "pixel width");
    let RenderedImagePayload::Bytes(png) = &image.payload else {
        panic!("expected PNG bytes payload");
    };
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG signature");
}

#[test]
fn render_with_stale_generation_is_rejected() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let mut request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    request.generation = domain::document::DocumentGeneration(request.generation.0 + 999);
    let result = block_on(engine.render_page(request)).unwrap();
    assert!(
        result.is_err(),
        "stale generation must be rejected (Appendix A §8)"
    );
}

#[test]
fn search_finds_expected_pages_without_mutating_file() {
    let engine = require_engine!();
    let path = fixture("multi-page-search.pdf");
    let bytes_before = std::fs::read(&path).unwrap();

    let session = block_on(engine.open_document(path.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(session.pages.len(), 3);

    let request = SearchRequest {
        document_id: session.id,
        generation: session.generation,
        query: SearchQuery::plain("tile"),
    };
    let results = block_on(engine.search_document(request)).unwrap().unwrap();

    // Fixture spec (RFC 015): "tile" appears on pages 1 and 3 (display).
    let pages: Vec<usize> = results
        .pages
        .iter()
        .map(|p| p.page_index.display_number())
        .collect();
    assert_eq!(pages, vec![1, 3], "matched pages");
    assert!(results.total_matches >= 2, "match count");

    // RFC 010 §11: search never mutates the PDF.
    let bytes_after = std::fs::read(&path).unwrap();
    assert_eq!(bytes_before, bytes_after, "PDF file must not be modified");
}

#[test]
fn close_document_invalidates_session() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();
    assert!(block_on(engine.close_document(session.id)).unwrap());

    let request = RenderPageRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket::from_scale(1.0),
        flags: RenderFlags::default(),
        format: RenderOutputFormat::Png,
    };
    let result = block_on(engine.render_page(request)).unwrap();
    assert!(result.is_err(), "closed document must not render");
}
