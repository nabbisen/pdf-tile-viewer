use std::sync::Arc;
use std::time::SystemTime;

use domain::document::{
    DocumentGeneration, DocumentId, DocumentMetadata, DocumentSession, DocumentSource,
    DocumentState, PageIndex,
};
use domain::navigation::{DocumentOutline, NavigationLimitStatus, PageLinkSet};
use domain::render::{RenderBackground, RenderCacheKey, RenderFlags, ScaleBucket};
use domain::text::{PageTextLayer, TextLayerCacheKey};
use pdf_engine::worker::EngineHandle;

use crate::navigation_service::{DocumentOutlineCacheKey, NavigationService};
use crate::render_service::{MINIMUM_CACHE_BUDGET_BYTES, RenderService};
use crate::session_service::SessionService;
use crate::text_service::{DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES, TextLayerService};

struct Fixture {
    service: SessionService,
    render: RenderService,
    text: TextLayerService,
    navigation: NavigationService,
}

fn fixture() -> Fixture {
    let engine = EngineHandle::unavailable();
    let render = RenderService::new(engine.clone(), MINIMUM_CACHE_BUDGET_BYTES);
    let text = TextLayerService::new(engine.clone(), DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES);
    let navigation = NavigationService::new(engine.clone());
    Fixture {
        service: SessionService::new(engine, render.clone(), text.clone(), navigation.clone()),
        render,
        text,
        navigation,
    }
}

fn session(id: u64, generation: u64) -> DocumentSession {
    DocumentSession {
        id: DocumentId(id),
        source: DocumentSource::LocalFile {
            path: "/tmp/session-service-test.pdf".into(),
            fingerprint: domain::document::FileFingerprint {
                size_bytes: 0,
                modified_at: None,
            },
        },
        display_name: "session-service-test.pdf".to_string(),
        metadata: DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            creator: None,
            producer: None,
            page_count: 0,
            encrypted: false,
        },
        pages: Vec::new(),
        opened_at: SystemTime::UNIX_EPOCH,
        generation: DocumentGeneration(generation),
        state: DocumentState::Ready,
    }
}

fn render_key(id: u64, generation: u64) -> RenderCacheKey {
    RenderCacheKey {
        document_id: DocumentId(id),
        document_generation: DocumentGeneration(generation),
        page_index: PageIndex(0),
        scale_bucket: ScaleBucket(100),
        render_flags: RenderFlags {
            background: RenderBackground::White,
        },
    }
}

fn text_key(id: u64, generation: u64) -> TextLayerCacheKey {
    TextLayerCacheKey {
        document_id: DocumentId(id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(0),
    }
}

fn text_layer(id: u64, generation: u64) -> Arc<PageTextLayer> {
    Arc::new(PageTextLayer {
        document_id: DocumentId(id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(0),
        segments: Vec::new(),
    })
}

fn outline(id: u64, generation: u64) -> Arc<DocumentOutline> {
    Arc::new(DocumentOutline {
        document_id: DocumentId(id),
        generation: DocumentGeneration(generation),
        roots: Vec::new(),
        limit_status: NavigationLimitStatus::default(),
    })
}

fn links(id: u64, generation: u64) -> Arc<PageLinkSet> {
    Arc::new(PageLinkSet {
        document_id: DocumentId(id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(0),
        links: Vec::new(),
        limit_status: NavigationLimitStatus::default(),
    })
}

fn fill_caches(f: &Fixture, id: u64, generation: u64) {
    f.render
        .cache()
        .lock()
        .unwrap()
        .insert(render_key(id, generation), vec![0u8; 64].into());
    f.text
        .cache()
        .lock()
        .unwrap()
        .insert(text_key(id, generation), text_layer(id, generation));
    let mut nav = f.navigation.cache().lock().unwrap();
    nav.insert_outline(
        DocumentOutlineCacheKey {
            document_id: DocumentId(id),
            generation: DocumentGeneration(generation),
        },
        outline(id, generation),
    );
    nav.insert_page_links(links(id, generation), 4);
}

fn cached_entries(f: &Fixture) -> (usize, usize, usize, usize) {
    (
        f.render.cache_entry_count(),
        f.text.cache_entry_count(),
        f.navigation.outline_count(),
        f.navigation.page_link_page_count(),
    )
}

#[test]
fn opened_registers_the_active_session_and_generation() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    assert_eq!(
        f.service.active_session(),
        Some((DocumentId(1), DocumentGeneration(7)))
    );
}

#[test]
fn close_evicts_every_cache_and_clears_the_active_session() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    fill_caches(&f, 1, 7);
    assert_eq!(cached_entries(&f), (1, 1, 1, 1));

    f.service.close(DocumentId(1), DocumentGeneration(7));

    assert_eq!(cached_entries(&f), (0, 0, 0, 0));
    assert_eq!(f.service.active_session(), None);
}

#[test]
fn late_results_for_a_closed_session_are_refused_by_every_cache() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    f.service.close(DocumentId(1), DocumentGeneration(7));

    fill_caches(&f, 1, 7); // what late async completions would do
    assert_eq!(cached_entries(&f), (0, 0, 0, 0));
}

#[test]
fn close_leaves_other_sessions_untouched() {
    let f = fixture();
    fill_caches(&f, 2, 8);
    f.service.close(DocumentId(1), DocumentGeneration(7));
    assert_eq!(cached_entries(&f), (1, 1, 1, 1));
}

#[test]
fn closing_twice_or_an_unknown_session_is_harmless() {
    let f = fixture();
    f.service.close(DocumentId(9), DocumentGeneration(9)); // never opened
    f.service.opened(&session(1, 7));
    fill_caches(&f, 1, 7);
    f.service.close(DocumentId(1), DocumentGeneration(7));
    f.service.close(DocumentId(1), DocumentGeneration(7));
    assert_eq!(cached_entries(&f), (0, 0, 0, 0));
    assert_eq!(f.service.active_session(), None);
}

#[test]
fn closing_an_old_session_keeps_the_new_one_active() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    f.service.close(DocumentId(1), DocumentGeneration(7));
    f.service.opened(&session(2, 8));
    f.service.close(DocumentId(1), DocumentGeneration(7)); // late duplicate close
    assert_eq!(
        f.service.active_session(),
        Some((DocumentId(2), DocumentGeneration(8)))
    );
}

#[test]
fn reopening_after_close_works_repeatedly() {
    let f = fixture();
    for n in 1..=5u64 {
        f.service.opened(&session(n, n + 10));
        fill_caches(&f, n, n + 10);
        f.service.close(DocumentId(n), DocumentGeneration(n + 10));
        assert_eq!(cached_entries(&f), (0, 0, 0, 0), "after cycle {n}");
    }
}

#[test]
fn the_same_session_registered_twice_is_not_a_second_session() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    f.service.opened(&session(1, 7));
    assert_eq!(
        f.service.active_session(),
        Some((DocumentId(1), DocumentGeneration(7)))
    );
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "another one is still active")]
fn opening_a_second_session_without_closing_trips_the_debug_assertion() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    f.service.opened(&session(2, 8));
}

#[cfg(not(debug_assertions))]
#[test]
fn release_builds_close_the_stale_session_when_another_is_opened() {
    let f = fixture();
    f.service.opened(&session(1, 7));
    fill_caches(&f, 1, 7);
    f.service.opened(&session(2, 8));
    assert_eq!(cached_entries(&f), (0, 0, 0, 0));
    assert_eq!(
        f.service.active_session(),
        Some((DocumentId(2), DocumentGeneration(8)))
    );
}
