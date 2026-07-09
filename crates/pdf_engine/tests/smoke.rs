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

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, MutexGuard};

use domain::document::{DocumentError, DocumentPassword, PageIndex};
use domain::navigation::{
    DestinationView, DisabledNavigationReason, DocumentOutlineRequest, NavigationResourceLimits,
    NavigationTarget, OutlineTitle, PageLinksRequest,
};
use domain::render::{
    RenderFlags, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, ScaleBucket,
};
use domain::search::{SearchQuery, SearchRequest};
use domain::text::{TextLayerError, TextLayerRequest};
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

static ENGINE_LOCK: Mutex<()> = Mutex::new(());

struct LockedEngine {
    handle: &'static EngineHandle,
    _guard: MutexGuard<'static, ()>,
}

impl Deref for LockedEngine {
    type Target = EngineHandle;

    fn deref(&self) -> &Self::Target {
        self.handle
    }
}

macro_rules! require_engine {
    () => {{
        let guard = ENGINE_LOCK.lock().expect("engine smoke lock poisoned");
        match ENGINE.as_ref() {
            Some(h) => LockedEngine {
                handle: h,
                _guard: guard,
            },
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
    let RenderedImagePayload::Bytes(png) = &image.payload;
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

#[test]
fn text_layer_extracts_segments_for_single_page() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = TextLayerRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
    };
    let layer = block_on(engine.extract_page_text_layer(request))
        .unwrap()
        .unwrap();

    assert_eq!(layer.document_id, session.id);
    assert_eq!(layer.generation, session.generation);
    assert_eq!(layer.page_index, PageIndex(0));
    assert!(!layer.segments.is_empty(), "fixture should expose text");
    for (expected, segment) in layer.segments.iter().enumerate() {
        assert_eq!(
            segment.segment_index, expected as u32,
            "segment index should preserve extraction order"
        );
        assert!(
            !segment.text.is_empty(),
            "empty text segments should be filtered"
        );
        assert!(segment.rect.width > 0.0);
        assert!(segment.rect.height > 0.0);
    }
}

#[test]
fn text_layer_rejects_stale_generation() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("single-page-basic.pdf")))
        .unwrap()
        .unwrap();

    let request = TextLayerRequest {
        document_id: session.id,
        generation: domain::document::DocumentGeneration(session.generation.0 + 999),
        page_index: PageIndex(0),
    };
    let result = block_on(engine.extract_page_text_layer(request)).unwrap();

    assert_eq!(result, Err(TextLayerError::DocumentNotOpen));
}

#[test]
fn navigation_outline_extracts_roots_and_nested_entries() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let outline = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(outline.document_id, session.id);
    assert_eq!(outline.generation, session.generation);
    assert_eq!(outline.roots.len(), 3);
    assert_eq!(
        outline.roots[0].title,
        OutlineTitle::Present("Chapter 1".to_string())
    );
    assert_eq!(outline.roots[0].children.len(), 1);
    assert_eq!(outline.roots[0].children[0].title, OutlineTitle::Missing);
    assert_eq!(
        outline.roots[1].title,
        OutlineTitle::Present("Chapter 2".to_string())
    );
    assert_eq!(
        outline.roots[2].title,
        OutlineTitle::Present("Empty URI".to_string())
    );
    assert!(matches!(
        &outline.roots[2].target,
        NavigationTarget::Disabled(DisabledNavigationReason::InvalidExternalUri)
    ));

    match &outline.roots[1].target {
        NavigationTarget::InternalDestination(destination) => {
            assert_eq!(destination.page_index, PageIndex(2));
            assert!(matches!(
                destination.view,
                DestinationView::CoordinatesAndZoom { .. }
            ));
        }
        other => panic!("expected internal destination, got {other:?}"),
    }
}

#[test]
fn navigation_page_links_extract_internal_uri_and_disabled_actions() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let page_one_links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(0),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(page_one_links.links.len(), 2);
    assert!(
        page_one_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::InternalDestination(destination)
                if destination.page_index == PageIndex(2)
        )),
        "page 1 should include an internal link to page 3"
    );
    assert!(
        page_one_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri)
                if uri.raw_uri == "https://example.com/pdf-tile-viewer"
        )),
        "page 1 should include the https URI"
    );

    let page_two_links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(1),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap()
    .unwrap();

    assert_eq!(page_two_links.links.len(), 3);
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri)
                if uri.raw_uri == "file:///tmp/pdf-tile-viewer-blocked"
        )),
        "PDFium extraction stores raw file URI but does not authorize opening"
    );
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::ExternalUri(uri) if uri.raw_uri == "relative/path"
        )),
        "relative URI remains raw metadata for app_services policy rejection"
    );
    assert!(
        page_two_links.links.iter().any(|link| matches!(
            &link.target,
            NavigationTarget::Disabled(DisabledNavigationReason::LaunchAction)
        )),
        "launch action must not be executable"
    );
}

#[test]
fn navigation_rejects_stale_generation_and_enforces_page_link_cap() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let stale = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: domain::document::DocumentGeneration(session.generation.0 + 999),
        limits: NavigationResourceLimits::default(),
    }))
    .unwrap();
    assert_eq!(
        stale,
        Err(domain::navigation::NavigationError::DocumentNotOpen)
    );

    let links = block_on(engine.extract_page_links(PageLinksRequest {
        document_id: session.id,
        generation: session.generation,
        page_index: PageIndex(1),
        limits: NavigationResourceLimits {
            max_links_per_page: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();

    assert_eq!(links.links.len(), 1);
    assert!(links.limit_status.links_truncated);
}

#[test]
fn navigation_outline_enforces_node_depth_and_string_payload_caps() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("navigation-links-outline.pdf")))
        .unwrap()
        .unwrap();

    let node_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_nodes: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert_eq!(node_limited.roots.len(), 1);
    assert!(node_limited.limit_status.outline_truncated);

    let depth_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_depth: 1,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert!(
        depth_limited.roots[0].children.is_empty(),
        "nested outline entries should be omitted after depth cap"
    );
    assert!(depth_limited.limit_status.outline_truncated);

    let string_limited = block_on(engine.extract_document_outline(DocumentOutlineRequest {
        document_id: session.id,
        generation: session.generation,
        limits: NavigationResourceLimits {
            max_outline_string_bytes: 3,
            ..NavigationResourceLimits::default()
        },
    }))
    .unwrap()
    .unwrap();
    assert!(matches!(
        &string_limited.roots[0].title,
        OutlineTitle::Truncated(value) if value == "Cha"
    ));
    assert!(string_limited.limit_status.outline_truncated);
    assert!(string_limited.limit_status.strings_truncated);
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

#[test]
fn large_document_search_runs_without_error() {
    let engine = require_engine!();
    let session = block_on(engine.open_document(fixture("fifty-pages-benchmark.pdf")))
        .unwrap()
        .unwrap();

    let request = domain::search::SearchRequest {
        document_id: session.id,
        generation: session.generation,
        query: domain::search::SearchQuery::plain("benchmark"),
    };
    let results = block_on(engine.search_document(request)).unwrap().unwrap();

    // "benchmark" appears on every page of the fixture.
    assert_eq!(results.pages.len(), 50, "all 50 pages should match");
    assert!(results.total_matches >= 50);
}
