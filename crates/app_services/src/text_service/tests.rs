use std::sync::Arc;

use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::search::{PageCoordinateSpace, PageRect};
use domain::text::{PageTextLayer, TextLayerCacheKey, TextLayerRequest, TextLayerSegment};

use crate::text_service::{TextLayerCache, estimate_layer_bytes, layer_matches_request};

fn key(page: usize, generation: u64) -> TextLayerCacheKey {
    TextLayerCacheKey {
        document_id: DocumentId(1),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(page),
    }
}

fn key_for(document_id: u64, page: usize, generation: u64) -> TextLayerCacheKey {
    TextLayerCacheKey {
        document_id: DocumentId(document_id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(page),
    }
}

fn layer(page: usize, generation: u64, text: &str) -> Arc<PageTextLayer> {
    layer_for(1, page, generation, text)
}

fn layer_for(document_id: u64, page: usize, generation: u64, text: &str) -> Arc<PageTextLayer> {
    Arc::new(PageTextLayer {
        document_id: DocumentId(document_id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(page),
        segments: vec![TextLayerSegment {
            segment_index: 0,
            text: text.to_string(),
            rect: PageRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 12.0,
                space: PageCoordinateSpace::PdfPointsBottomLeft,
            },
        }],
    })
}

#[test]
fn cache_miss_returns_none() {
    let cache = TextLayerCache::new(1024);
    assert!(cache.get(&key(0, 1)).is_none());
}

#[test]
fn insert_then_get_returns_text_layer() {
    let page = layer(0, 1, "Hello");
    let mut cache = TextLayerCache::new(estimate_layer_bytes(&page) * 2);

    assert!(cache.insert(key(0, 1), page.clone()));

    let got = cache.get(&key(0, 1)).unwrap();
    assert_eq!(got.segments[0].text, "Hello");
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn duplicate_insert_is_ignored() {
    let first = layer(0, 1, "First");
    let second = layer(0, 1, "Second");
    let mut cache = TextLayerCache::new(estimate_layer_bytes(&first) * 3);

    assert!(cache.insert(key(0, 1), first));
    assert!(!cache.insert(key(0, 1), second));

    let got = cache.get(&key(0, 1)).unwrap();
    assert_eq!(got.segments[0].text, "First");
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn eviction_occurs_when_over_budget() {
    let first = layer(0, 1, "Text");
    let second = layer(1, 1, "Text");
    let third = layer(2, 1, "Text");
    let budget = estimate_layer_bytes(&first) + estimate_layer_bytes(&second);
    let mut cache = TextLayerCache::new(budget);

    assert!(cache.insert(key(0, 1), first));
    assert!(cache.insert(key(1, 1), second));
    assert_eq!(cache.entry_count(), 2);

    assert!(cache.insert(key(2, 1), third));
    assert_eq!(cache.entry_count(), 2);
    assert!(cache.get(&key(0, 1)).is_none());
    assert!(cache.get(&key(2, 1)).is_some());
}

#[test]
fn oversized_page_is_not_cached() {
    let page = layer(0, 1, "A large extracted text layer");
    let mut cache = TextLayerCache::new(estimate_layer_bytes(&page) - 1);

    assert!(!cache.insert(key(0, 1), page));
    assert_eq!(cache.entry_count(), 0);
    assert_eq!(cache.used_bytes, 0);
}

#[test]
fn evict_session_removes_entries_and_suppresses_late_insert() {
    let page = layer(0, 1, "Hello");
    let mut cache = TextLayerCache::new(estimate_layer_bytes(&page) * 2);

    assert!(cache.insert(key(0, 1), page.clone()));
    cache.evict_session(DocumentId(1), DocumentGeneration(1));

    assert!(cache.get(&key(0, 1)).is_none());
    assert!(
        cache.is_session_suppressed(crate::text_service::TextLayerSessionKey {
            document_id: DocumentId(1),
            generation: DocumentGeneration(1),
        })
    );
    assert!(!cache.insert(key(0, 1), page));
}

#[test]
fn evict_session_does_not_suppress_same_generation_for_another_document() {
    let first_doc = layer_for(1, 0, 1, "First");
    let second_doc = layer_for(2, 0, 1, "Second");
    let mut cache = TextLayerCache::new(
        (estimate_layer_bytes(&first_doc) + estimate_layer_bytes(&second_doc)) * 2,
    );

    assert!(cache.insert(key_for(1, 0, 1), first_doc.clone()));
    cache.evict_session(DocumentId(1), DocumentGeneration(1));

    assert!(!cache.insert(key_for(1, 0, 1), first_doc));
    assert!(cache.insert(key_for(2, 0, 1), second_doc));
}

#[test]
fn layer_request_match_checks_document_generation_and_page() {
    let request = TextLayerRequest {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(3),
    };
    let matching = PageTextLayer {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(3),
        segments: Vec::new(),
    };
    let wrong_page = PageTextLayer {
        page_index: PageIndex(4),
        ..matching.clone()
    };

    assert!(layer_matches_request(&matching, &request));
    assert!(!layer_matches_request(&wrong_page, &request));
}
